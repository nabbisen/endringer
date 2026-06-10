//! gix-specific conversion helpers.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use endringer_core::types::{CommitId, ObjectId};

/// Converts a signed Unix timestamp (seconds) to [`SystemTime`].
/// Negative values (pre-1970) are saturated to `UNIX_EPOCH`.
pub(crate) fn seconds_to_systemtime(seconds: i64) -> SystemTime {
    if seconds >= 0 {
        UNIX_EPOCH + Duration::from_secs(seconds as u64)
    } else {
        UNIX_EPOCH
    }
}

/// Converts a `gix::ObjectId` to a backend-agnostic [`CommitId`].
pub(crate) fn gix_id_to_commit_id(id: gix::ObjectId) -> CommitId {
    CommitId::from_bytes(id.as_slice().to_vec())
}

/// Converts a `gix::ObjectId` to a backend-agnostic [`ObjectId`].
///
/// Used by tree enumeration, ref-target resolution, and conflict-state reads
/// (RFCs 010, 011, 008). Defined here ready for those consumers.
#[allow(dead_code)]
pub(crate) fn gix_id_to_object_id(id: gix::ObjectId) -> ObjectId {
    ObjectId::from_bytes(id.as_slice().to_vec())
}
