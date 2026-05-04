use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::CommitId;

/// Converts a [`CommitId`] to its conventional 7-character hex abbreviation.
///
/// This is a convenience free function.  You can also call
/// [`CommitId::short`] directly.
///
/// # Example
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
/// Git stores timestamps as `i64`.  For the rare case of a negative value
/// (commits authored or committed before 1970-01-01) the result is clamped to
/// `UNIX_EPOCH` rather than silently wrapping.
pub(crate) fn seconds_to_systemtime(seconds: i64) -> SystemTime {
    if seconds >= 0 {
        UNIX_EPOCH + Duration::from_secs(seconds as u64)
    } else {
        // Timestamps before 1970 are vanishingly rare in real repositories.
        // Saturate to UNIX_EPOCH rather than wrap to a date far in the future.
        UNIX_EPOCH
    }
}
