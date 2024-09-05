use std::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};

use crate::time::Clock;

/// event to be scheduled
pub trait Event<T: Clock>: Ord + Send + Sync {
    /// name of event
    fn name(&self) -> &str;
    /// time to execute event
    fn execution_time(&self) -> T::Time;
    /// next time to execute event
    fn next_time(&self) -> Self;
    /// number of times to execute event
    fn count(&self, new_count: u64) -> Self;
}

/// Notification event occured
#[derive(Clone, Debug)]
pub struct EventNotification<T: Clock> {
    pub name: String,
    pub time: T::Time,
}

impl<T: Clock> EventNotification<T> {
    /// name of event that occured
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// time the event occured
    pub fn time(&self) -> T::Time {
        self.time
    }
}

impl<T: Clock> Display for EventNotification<T>
where
    T::Time: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.time)
    }
}
