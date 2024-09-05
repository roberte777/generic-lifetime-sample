use std::{cmp::Reverse, collections::BinaryHeap, marker::PhantomData, sync::Arc};

use tokio::sync::{mpsc, Notify, RwLock};

use crate::{error::ToolboxError, time::Clock};

use super::event::{Event, EventNotification};

pub enum SchedulerCommand<E> {
    Schedule(E),
    Cancel { name: String },
    Stop,
}

pub struct SchedulerHandle<T: Clock, E: Event<T>> {
    command_sender: mpsc::UnboundedSender<SchedulerCommand<E>>,
    clock: Arc<RwLock<T>>,
    _phantom: PhantomData<E>,
}

impl<T: Clock + Sync + Send, E: Event<T> + 'static> SchedulerHandle<T, E> {
    pub fn new(
        clock: Arc<RwLock<T>>,
        event_sender: mpsc::UnboundedSender<EventNotification<T>>,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let clock_clone = Arc::clone(&clock);
        let mut actor = Scheduler::new(receiver, event_sender, clock_clone);

        tokio::spawn(async move {
            _ = actor.run().await;
        });

        Self {
            command_sender: sender,
            clock,
            _phantom: PhantomData,
        }
    }

    pub fn schedule(&self, event: E) -> Result<(), mpsc::error::SendError<SchedulerCommand<E>>> {
        self.command_sender.send(SchedulerCommand::Schedule(event))
    }

    pub async fn now(&self) -> T::Time {
        self.clock.read().await.now()
    }

    pub fn cancel_scheduled_event(
        &self,
        name: &str,
    ) -> Result<(), mpsc::error::SendError<SchedulerCommand<E>>> {
        self.command_sender.send(SchedulerCommand::Cancel {
            name: name.to_string(),
        })
    }

    pub async fn stop(&self) -> Result<(), mpsc::error::SendError<SchedulerCommand<E>>> {
        self.command_sender.send(SchedulerCommand::Stop)
    }
}

pub struct Scheduler<T: Clock, E: Event<T>> {
    clock: Arc<RwLock<T>>,
    command_receiver: mpsc::UnboundedReceiver<SchedulerCommand<E>>,
    event_sender: mpsc::UnboundedSender<EventNotification<T>>,
}

impl<T: Clock, E: Event<T>> Scheduler<T, E> {
    pub fn new(
        command_receiver: mpsc::UnboundedReceiver<SchedulerCommand<E>>,
        event_sender: mpsc::UnboundedSender<EventNotification<T>>,
        clock: Arc<RwLock<T>>,
    ) -> Self {
        Self {
            clock,
            command_receiver,
            event_sender,
        }
    }

    pub async fn run(&mut self) -> Result<(), ToolboxError> {
        let mut events = BinaryHeap::new();
        let notify = Notify::new();
        let mut sleep_time: Option<tokio::time::Duration> = None;

        loop {
            tokio::select! {
                Some(task) = self.command_receiver.recv() => {
                    match task {
                        SchedulerCommand::Schedule(evt) => {
                            events.push(Reverse(evt));
                            notify.notify_one();
                        }
                        SchedulerCommand::Cancel { name } => {
                            events.retain(|Reverse(evt)| evt.name() != name);
                        }
                        SchedulerCommand::Stop => {
                            events.clear();
                            self.command_receiver.close();
                            break;
                        }
                    }
                },
                _ = notify.notified() => {
                    let now = self.clock.read().await.now();
                    while let Some(task) = events.peek() {
                        if task.0.execution_time() <= now {
                            if let Some(Reverse(task)) = events.pop() {
                                let next_time = task.next_time();
                                if next_time.execution_time() > now {
                                    events.push(Reverse(next_time));
                                }
                                self.event_sender.send(EventNotification {
                                    name: task.name().to_string(),
                                    time: task.execution_time(),
                                })?;
                            }
                        } else {
                            let time_diff = self.clock.read().await.delay_time(task.0.execution_time());
                            let duration = tokio::time::Duration::from_millis(i64::from(time_diff) as u64);
                            sleep_time = Some(duration);
                            break;
                        }
                    }

                    if events.is_empty() {
                        sleep_time = None;
                    }
                },
                _ = async {
                    if let Some(duration) = sleep_time {
                        tokio::time::sleep(duration).await;
                    } else {
                        tokio::time::sleep(tokio::time::Duration::MAX).await;
                    }
                } => {
                    notify.notify_one();
                }
            }
        }
        Ok(())
    }
}
