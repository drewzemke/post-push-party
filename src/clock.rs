/// time context for bonus calculations
#[derive(Debug, Default, Clone, Copy)]
pub struct Clock {
    now: u64,
    tz_offset_secs: i32,
}

impl Clock {
    pub const SECONDS_PER_DAY: i64 = 86400;

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
    pub fn day_id_of(&self, timestamp: u64) -> i64 {
        (timestamp as i64 + self.tz_offset_secs as i64) / Self::SECONDS_PER_DAY
    }

    /// local day number for `now`
    pub fn today_id(&self) -> i64 {
        self.day_id_of(self.now)
    }

    /// the timestamp at the start of a given day
    pub fn day_start(&self, day_id: i64) -> u64 {
        (Self::SECONDS_PER_DAY * day_id - self.tz_offset_secs as i64) as u64
    }

    /// the timestamp at the start of the current day
    pub fn today_start(&self) -> u64 {
        let today_id = (self.now as i64 + self.tz_offset_secs as i64) / Self::SECONDS_PER_DAY;
        self.day_start(today_id)
    }

    /// day of the week for `now`
    /// Thursday is 0, Friday is 1, etc
    pub fn day_of_week(&self) -> i64 {
        self.day_id_of(self.now).rem_euclid(7)
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

    pub fn tz_offset_secs(&self) -> i32 {
        self.tz_offset_secs
    }
}
