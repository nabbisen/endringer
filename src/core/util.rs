use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn seconds_to_systemtime(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}
