/// time context for bonus calculations
#[derive(Debug, Default, Clone, Copy)]
pub struct Clock {
    pub now: u64,
    pub tz_offset_secs: i32,
}

impl Clock {
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
}
