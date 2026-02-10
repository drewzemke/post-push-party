/// time context for bonus calculations
#[derive(Debug, Default, Clone, Copy)]
pub struct Clock {
    now: u64,
    tz_offset_secs: i32,
}

impl Clock {
    #[cfg(test)]
    pub fn at(now: u64) -> Self {
        Self {
            now,
            tz_offset_secs: 0,
        }
    }

    pub fn with_offset(now: u64, tz_offset_secs: i32) -> Self {
        Self {
            now,
            tz_offset_secs,
        }
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    /// convert a utc timestamp to local day number
    pub fn day_of(&self, timestamp: u64) -> i64 {
        const SECONDS_PER_DAY: i64 = 86400;
        (timestamp as i64 + self.tz_offset_secs as i64) / SECONDS_PER_DAY
    }

    /// local day number for `now`
    pub fn today(&self) -> i64 {
        self.day_of(self.now)
    }

    /// day of the week for `now`
    /// Thursday is 0, Friday is 1, etc
    pub fn day_of_week(&self) -> i64 {
        self.day_of(self.now).rem_euclid(7)
    }

    /// seconds elapsed since midnight in local time
    pub fn local_seconds_since_midnight(&self) -> i64 {
        let local_seconds = self.now as i64 + self.tz_offset_secs as i64;
        local_seconds.rem_euclid(86400)
    }

    /// Creates a Clock for the current moment.
    pub fn from_now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // get local timezone offset
        let tz_offset_secs = chrono::Local::now().offset().local_minus_utc();

        Self::with_offset(now, tz_offset_secs)
    }

    pub fn is_today(&self, timestamp: u64) -> bool {
        self.day_of(timestamp) == self.today()
    }
}
