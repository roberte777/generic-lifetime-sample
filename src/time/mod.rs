/*!
The time module handles timing and scheduling of events based on simulation time as provided by a simulation clock.
The simulation clock and either by time step based or real (wall) clock time. The simulation clock operates at
millisecond resolution as an offset from the Unix timestamp, i.e. January 1st 1970 at midnight.
 */
mod real_time;
mod real_time_sim_clock;
mod sim_time;

pub use crate::time::real_time::{TimeDuration, TimeStamp, WallTime};
pub use crate::time::sim_time::{SimDuration, SimTime};

/// SimClock trait that extends Clock
pub trait SimClock: Clock<Time = SimTime> {
    /// Start the clock.
    ///  
    /// # Arguments
    ///  
    /// * `simulation_start_time` - The wall clock time when the simulation was started. Used to sync
    ///     clocks across services.
    /// * `relative_start_time` - The virtual time of the simulation. Use to have the simulation occur
    ///     on a particular date/time.
    /// * `elapsed_pause_time` - The amount of pause time accumulated
    /// * `time_dilation` - Time dilation factor. Greater than 1 is faster than real time and less
    ///     than 1 is slower than real time.
    fn start(
        &mut self,
        simulation_start_time: WallTime,
        relative_start_time: SimTime,
        elapsed_pause_time: TimeDuration,
        time_dilation: f64,
    );
    /// Adjust the time of the underlying clock either forwards or backwards to correct for clock
    /// drift or for time steps.
    ///  
    /// # Arguments
    ///  
    /// * `by` - The amount to adjust the clock.    
    fn offset_by(&mut self, by: TimeDuration);
    /// Pause the simulation clock.
    fn pause(&mut self);
    /// Resume the simulation clock. Clock must currently be paused when resume is called.
    fn resume(&mut self);
    /// Stop the simulation clock.
    fn stop(&mut self);
    /// Return a boolean indicating if the clock is currently paused.
    fn is_paused(&self) -> bool;
    /// Return a boolean indicating if the clock is currently running.
    fn is_running(&self) -> bool;
    /// Return a boolean indicating if the clock is currently stopped.
    fn is_stopped(&self) -> bool;
    /// Return the amount of time elapsed in the simulation excluding paused time.
    fn elapsed(&self) -> SimDuration;
}

/// The `Clock` trait defines a read only interface to the underlying clock that allows its state
/// to be read.
pub trait Clock {
    type Time: Ord + Copy + Send + Sync;

    /// Return the current simulation time.
    fn now(&self) -> Self::Time;

    /// Computes the time between *now* and *then*.
    ///
    /// This is used to estimate the amount of wall clock time the scheduler
    /// can delay before the target time arrives.     
    ///
    /// # Arguments
    ///
    /// * `then` - The time to wait until.
    fn delay_time(&self, then: Self::Time) -> TimeDuration;
}

/// The states that the clock may be in.
#[derive(PartialEq, Debug)]
pub enum ClockState {
    /// The clock is currently moving forward in time
    Running,
    /// The clock is not running
    Stopped,
    /// Time is paused and may resume from the current clock time
    Paused,
}
