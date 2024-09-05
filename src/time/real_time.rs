use crate::error::ToolboxError;
use crate::time::SimTime;

/// This type represents a [`WallTime`] time stamp as a microsecond offset as
/// [`WallTime`] is not serializable.
#[derive(serde::Serialize, serde::Deserialize, Debug, Copy, Clone)]
pub struct TimeStamp(i64);

/// Wrapper type around the underlying duration type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeDuration(chrono::Duration);

impl TimeDuration {
    /// Returns a `TimeDuration` with 0 duration
    pub fn zero() -> Self {
        Self(chrono::Duration::zero())
    }

    /// Returns a `TimeDuration` with millisecond duration
    ///
    /// # Arguments
    ///
    /// * `millis` - The number of millisecond duration
    pub fn milliseconds(millis: i64) -> Self {
        Self(chrono::Duration::milliseconds(millis))
    }

    pub(crate) fn as_duration(&self) -> chrono::Duration {
        self.0
    }
}

impl std::ops::AddAssign<TimeDuration> for TimeDuration {
    fn add_assign(&mut self, rhs: TimeDuration) {
        self.0 += rhs.0;
    }
}

impl From<chrono::NaiveDateTime> for TimeStamp {
    fn from(value: chrono::NaiveDateTime) -> Self {
        TimeStamp(value.and_utc().timestamp_micros())
    }
}

impl From<SimTime> for TimeStamp {
    fn from(value: SimTime) -> Self {
        TimeStamp(value.as_micros() as i64)
    }
}

impl TryFrom<TimeStamp> for WallTime {
    type Error = ToolboxError;

    fn try_from(value: TimeStamp) -> Result<Self, Self::Error> {
        Ok(WallTime(
            chrono::DateTime::from_timestamp_micros(value.0)
                .ok_or(ToolboxError::Conversion(
                    "Could not convert TimeStamp to WallTime".to_string(),
                ))?
                .naive_utc(),
        ))
    }
}

impl From<i64> for TimeDuration {
    fn from(value: i64) -> Self {
        TimeDuration(chrono::Duration::milliseconds(value))
    }
}

impl From<TimeDuration> for i64 {
    fn from(value: TimeDuration) -> i64 {
        value.0.num_milliseconds()
    }
}

impl From<TimeDuration> for chrono::Duration {
    fn from(val: TimeDuration) -> Self {
        val.0
    }
}

/// A simulation time.
#[derive(Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct WallTime(chrono::NaiveDateTime);

impl WallTime {
    /// Create a new instance with the current wall clock time.
    pub fn now() -> WallTime {
        Self(chrono::Utc::now().naive_utc())
    }

    /// Makes a new `WallTime` from the number of non-leap milliseconds
    /// since January 1, 1970 0:00:00.000 UTC (aka "UNIX timestamp").
    ///
    /// # Errors
    ///
    /// # Example
    /// Returns `None` on out-of-range number of milliseconds, otherwise returns `Some(WallTime)`.
    ///
    /// ```
    /// use vct_utils::time::WallTime;
    ///
    /// let dt = WallTime::from_timestamp_millis(947638923004).expect("invalid timestamp");
    /// ```
    pub fn from_timestamp_millis(millis: i64) -> Option<WallTime> {
        let t = chrono::DateTime::from_timestamp_millis(millis)?;
        Some(WallTime(t.naive_utc()))
    }

    /// Return time stamp as a [`chrono::NaiveDateTime`]
    pub fn as_date_time(&self) -> chrono::NaiveDateTime {
        self.0
    }

    /// Return as a msec offset
    pub fn timestamp_millis(&self) -> i64 {
        self.0.and_utc().timestamp_millis()
    }
}

impl Default for WallTime {
    fn default() -> Self {
        Self::now()
    }
}

impl From<WallTime> for TimeStamp {
    fn from(value: WallTime) -> Self {
        TimeStamp(value.0.and_utc().timestamp_micros())
    }
}

impl std::ops::AddAssign<TimeDuration> for WallTime {
    fn add_assign(&mut self, rhs: TimeDuration) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub for WallTime {
    type Output = TimeDuration;

    fn sub(self, rhs: Self) -> Self::Output {
        TimeDuration(self.0 - rhs.0)
    }
}

impl std::ops::Sub for TimeDuration {
    type Output = TimeDuration;

    fn sub(self, rhs: TimeDuration) -> Self::Output {
        TimeDuration(self.0 - rhs.0)
    }
}

impl std::ops::Mul<f64> for TimeDuration {
    type Output = TimeDuration;

    fn mul(self, rhs: f64) -> Self::Output {
        TimeDuration(chrono::Duration::microseconds(
            (self.0.num_microseconds().unwrap() as f64 * rhs).floor() as i64,
        ))
    }
}
