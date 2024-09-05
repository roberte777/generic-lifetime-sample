//! This module contains a simulation clock that operates on a multiple of real time.
//! By default, it operates at a 1 to 1 scale, but can be sped up or slowed down.
//!
use crate::time::{Clock, ClockState, SimClock, SimDuration, SimTime, TimeDuration, WallTime};

/// `RealTimeSimClock` is a simulation clock that operates at a multiple of real time.
pub struct RealTimeSimClock {
    /// start time of the simulation
    simulation_start_time: WallTime,
    /// Reported start time of the simulation. Used to offset the reported time to some arbitrary start time.
    relative_start_time: SimTime,
    /// Total paused time in ms.
    paused_time: TimeDuration,
    /// Used to run at a non-real-time speed. Values > 1 indicate faster than real-time factor and < 1 indicate slow down
    /// factor. Value must be > 0.
    time_dilation: f64,
    /// Current state of the clock.
    state: ClockState,
    /// The wall clock time that pause began, or None if the clock is not paused.
    pause_start_time: Option<WallTime>,
}

impl RealTimeSimClock {
    /// Return the simulation start value passed from the controller
    pub fn simulation_start_time(&self) -> WallTime {
        self.simulation_start_time
    }

    /// Return the accumulated time spent paused
    pub fn pause_time(&self) -> TimeDuration {
        self.paused_time
    }
}

impl Default for RealTimeSimClock {
    fn default() -> Self {
        let now = WallTime::now();
        Self {
            simulation_start_time: now,
            relative_start_time: SimTime::from_seconds(0),
            paused_time: TimeDuration::zero(),
            time_dilation: 1.0,
            state: ClockState::Stopped,
            pause_start_time: None,
        }
    }
}

impl Clock for RealTimeSimClock {
    type Time = SimTime;
    fn now(&self) -> Self::Time {
        let rt_now = WallTime::now();
        let current_pause = if let Some(pause) = self.pause_start_time {
            rt_now - pause
        } else {
            TimeDuration::milliseconds(0)
        };
        let execution_time = rt_now - self.simulation_start_time - self.paused_time - current_pause;
        self.relative_start_time + execution_time * self.time_dilation
    }

    /// Calculates the delay time for a future event, taking into account the current time dilation factor.
    /// This method is useful for determining how long to wait in real-time for an event that is scheduled
    /// in simulation time.
    ///
    /// # Arguments
    /// * `then` - The future simulation time at which an event is scheduled to occur.
    ///
    /// # Returns
    /// The real-time duration that corresponds to the delay until the event, adjusted for the current time dilation.
    fn delay_time(&self, then: Self::Time) -> TimeDuration {
        let delta = (then - self.now()) / self.time_dilation;
        if delta > 0 {
            TimeDuration::milliseconds(delta.num_milliseconds())
        } else {
            TimeDuration::zero()
        }
    }
}

impl SimClock for RealTimeSimClock {
    /// Initializes or resets the clock with specified start times and default settings.
    /// This clock is created paused (time is not ticking).
    ///
    /// # Arguments
    /// * `simulation_start_time` - The absolute start time of the simulation.
    /// * `relative_start_time` - The relative start time to which the simulation's reported time is offset.
    fn start(
        &mut self,
        simulation_start_time: WallTime,
        relative_start_time: SimTime,
        elapsed_pause_time: TimeDuration,
        time_dilation: f64,
    ) {
        self.simulation_start_time = simulation_start_time;
        self.relative_start_time = relative_start_time;
        self.paused_time = elapsed_pause_time;
        self.time_dilation = time_dilation;
        self.state = ClockState::Paused;
        self.pause_start_time = Some(WallTime::now());
    }

    /// Adjusts the simulation start time by a specified duration. This can be used to move the simulation's
    /// start time forward, effectively shifting when the simulated events occur.
    ///
    /// # Arguments
    /// * `by` - The duration to offset the simulation start time.
    fn offset_by(&mut self, by: TimeDuration) {
        self.simulation_start_time += by;
    }

    /// Pauses the simulation clock. This method records the current time as the pause start time, effectively
    /// stopping the advancement of the simulation time until `resume` is called.
    fn pause(&mut self) {
        self.state = ClockState::Paused;
        self.pause_start_time = Some(WallTime::now());
    }

    /// Resumes the simulation clock from a paused state. This method calculates the total duration of the pause
    /// and adds it to the total paused time, allowing the simulation to continue from where it left off.
    fn resume(&mut self) {
        self.paused_time += WallTime::now() - self.pause_start_time.unwrap_or(WallTime::now());
        self.state = ClockState::Running;
        self.pause_start_time = None;
    }

    /// Stops the simulation clock. This is similar to pausing but intended to signal a more permanent halt.
    /// The clock records the current time as the stop time, and the simulation's state is set to `Stopped`.
    fn stop(&mut self) {
        self.pause_start_time = Some(WallTime::now());
        self.state = ClockState::Stopped;
    }

    /// Calculates the elapsed time since the simulation started, accounting for any paused duration.
    ///
    /// # Returns
    /// The total elapsed time as `Duration`, excluding any periods during which the clock was paused.
    fn elapsed(&self) -> SimDuration {
        self.now() - self.relative_start_time
    }

    /// Checks if the simulation clock is currently paused.
    ///
    /// # Returns
    /// `true` if the clock is in the `Paused` state, `false` otherwise.
    fn is_paused(&self) -> bool {
        self.state == ClockState::Paused
    }

    /// Checks if the simulation clock is currently running.
    ///
    /// # Returns
    /// `true` if the clock is in the `Running` state, `false` otherwise.
    fn is_running(&self) -> bool {
        self.state == ClockState::Running
    }

    /// Checks if the simulation clock has been stopped.
    ///
    /// # Returns
    /// `true` if the clock is in the `Stopped` state, `false` otherwise.
    fn is_stopped(&self) -> bool {
        self.state == ClockState::Stopped
    }
}

#[cfg(test)]
mod rt_clock_tests {
    use crate::time::real_time_sim_clock::RealTimeSimClock;
    use crate::time::{Clock, SimClock};
    use std::thread::sleep;

    #[test]
    pub fn clock_starts_in_stopped_state() {
        let clock = RealTimeSimClock::default();
        assert!(clock.is_stopped());
    }

    #[test]
    pub fn stopped_clock_returns_same_time() {
        let mut clock = RealTimeSimClock::default();
        clock.stop();
        assert!(clock.is_stopped());
        let before = clock.now();
        sleep(core::time::Duration::from_millis(2));
        assert_eq!(before, clock.now());
    }

    #[test]
    pub fn paused_clock_returns_same_time() {
        let mut clock = RealTimeSimClock::default();
        clock.pause();
        assert!(clock.is_paused());
        let before = clock.now();
        sleep(core::time::Duration::from_millis(2));
        assert_eq!(before, clock.now());
    }
}
