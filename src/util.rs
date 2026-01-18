use gix::ObjectId;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn commit_id_to_short_id(commit_id: ObjectId) -> String {
    commit_id.to_string()[..7].to_owned()
}

pub fn seconds_to_systemtime(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}
