use crate::time::real_time::TimeDuration;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

/// A simulation time duration. i.e. the amount of time elapsed between two simulation time
/// measurements.
#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub struct SimDuration(chrono::Duration);

impl SimDuration {
    /// Creates an instance with 0 duration
    pub fn zero() -> Self {
        Self(chrono::Duration::zero())
    }

    /// Creates an instance with the specified number of seconds
    pub fn seconds(seconds: i64) -> Self {
        Self(chrono::Duration::seconds(seconds))
    }

    /// Returns a `SimDuration` with number of milliseconds
    pub fn milliseconds(millis: i64) -> Self {
        Self(chrono::Duration::milliseconds(millis))
    }

    /// Returns a `SimDuration` with micros number of microseconds
    ///
    /// # Arguments
    ///
    /// * `micros` - The number of microseconds duration
    pub fn microseconds(micros: i64) -> Self {
        Self(chrono::Duration::microseconds(micros))
    }

    /// Returns the number of milliseconds since the simulation clock started
    pub fn num_milliseconds(&self) -> i64 {
        self.0.num_milliseconds()
    }
}

impl std::ops::Div<f64> for SimDuration {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        SimDuration::milliseconds((self.0.num_milliseconds() as f64 / rhs) as i64)
    }
}

impl PartialEq<i64> for SimDuration {
    fn eq(&self, other: &i64) -> bool {
        self.0.num_milliseconds() == *other
    }
}

impl PartialOrd<i64> for SimDuration {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        Some(self.0.num_milliseconds().cmp(other))
    }
}

/// A time measurement for internal simulation time.
///
/// This time is represented internal as a zero based microsecond offset
/// from the simulation start. It accounts for pauses and stops in the
/// simulation.
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct SimTime(
    //microseconds
    u64,
);

impl SimTime {
    /// Create from microseconds
    pub fn from_micros(microseconds: u64) -> Self {
        Self(microseconds)
    }
    /// Create from milliseconds
    pub fn from_millis(milliseconds: u64) -> Self {
        Self(milliseconds * 1_000)
    }
    /// Create from seconds
    pub fn from_seconds(seconds: u64) -> Self {
        Self(seconds * 1_000_000)
    }

    /// Zero value.
    pub fn zero() -> Self {
        Self(0)
    }
}

impl SimTime {
    /// Return as millisecond offset.
    pub fn as_millis(&self) -> u64 {
        self.0 / 1_000
    }
    /// Return as microsecond offset.
    pub fn as_micros(&self) -> u64 {
        self.0
    }
    /// Return as second offset.
    pub fn as_seconds(&self) -> u64 {
        self.0 / 1_000_000
    }
}

impl std::ops::Sub for SimTime {
    type Output = SimDuration;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.0 >= rhs.0 {
            return SimDuration(chrono::Duration::microseconds((self.0 - rhs.0) as i64));
        }

        SimDuration::zero()
    }
}

impl std::ops::Add<SimDuration> for SimTime {
    type Output = SimTime;

    fn add(self, rhs: SimDuration) -> Self::Output {
        let duration = rhs
            .0
            .num_microseconds()
            .expect("Duration should not overflow");
        SimTime::from_micros(self.0 + duration as u64)
    }
}

impl std::ops::Add<TimeDuration> for SimTime {
    type Output = SimTime;

    fn add(self, rhs: TimeDuration) -> Self::Output {
        SimTime(
            self.0
                + rhs
                    .as_duration()
                    .num_microseconds()
                    .expect("TimeDuration should be valid microseconds") as u64,
        )
    }
}

impl Display for SimTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
