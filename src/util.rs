use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::CommitId;

/// Converts a [`CommitId`] to its conventional 7-character hex abbreviation.
///
/// This is a convenience free function; see also [`CommitId::short`].
///
/// ```no_run
/// use endringer::{repository::repository, commit_id_to_short_id};
///
/// let repo = repository(std::path::Path::new(".")).expect("open repo");
/// let digest = repo.status_digest().expect("status digest");
/// let short = commit_id_to_short_id(digest.last_commit_id);
/// assert_eq!(short.len(), 7);
/// ```
pub fn commit_id_to_short_id(commit_id: CommitId) -> String {
    commit_id.short()
}

/// Converts a signed Unix timestamp (seconds since epoch, as returned by gix)
/// to a [`SystemTime`].
///
/// Negative values (pre-1970 commits) are saturated to `UNIX_EPOCH`.
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
