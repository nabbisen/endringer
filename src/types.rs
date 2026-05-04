use std::time::SystemTime;

/// Opaque commit identifier (SHA-1, 40 hex characters).
///
/// This type intentionally hides the underlying VCS library type so that
/// callers have no compile-time dependency on `gix`.  Use
/// [`Display`][std::fmt::Display] to obtain the full 40-character hex string,
/// or [`CommitId::short`] for the conventional 7-character abbreviation.
///
/// # Example
///
/// ```
/// # use endringer::types::CommitId;
/// // Full hex representation is available via the Display trait.
/// // Seven-character abbreviation:
/// // let short = commit_id.short();   // e.g. "a1b2c3d"
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommitId(pub(crate) gix::ObjectId);

impl CommitId {
    /// Returns the first 7 hex characters of the commit ID — the conventional
    /// "short" form used in log output and tag descriptions.
    pub fn short(&self) -> String {
        // Format only the first 4 raw bytes (= 8 hex chars) then truncate to 7.
        // This avoids allocating the full 40-character hex string just to take
        // a prefix.
        let bytes = self.0.as_slice();
        let mut out = String::with_capacity(7);
        for &b in bytes.iter().take(4) {
            out.push(char::from_digit((b >> 4) as u32, 16).unwrap_or('?'));
            out.push(char::from_digit((b & 0xf) as u32, 16).unwrap_or('?'));
        }
        out.truncate(7);
        out
    }
}

impl std::fmt::Display for CommitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information about a branch (local or remote).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchInfo {
    /// Short branch name, e.g. `main`.
    pub name: String,
    /// Full ref name, e.g. `refs/heads/main` or `refs/remotes/origin/main`.
    pub full_name: String,
    /// Commit ID at the tip of the branch.
    pub last_commit_id: CommitId,
    /// First line of the most recent commit message.
    pub last_commit_summary: String,
    /// Committer timestamp of the most recent commit.
    pub last_commit_timestamp: SystemTime,
}

/// Lightweight summary of the repository's current state.
///
/// Returned by [`Repository::status_digest`][crate::repository::Repository::status_digest].
/// Useful as a quick health-check or change-detection signal without walking
/// the full commit history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusDigest {
    /// Directory name of the repository's working tree (derived from the
    /// `workdir` path; falls back to `"unknown"` for bare repositories).
    pub repo_name: String,
    /// Name of the currently checked-out branch, or `"(detached)"` when HEAD
    /// is in detached-HEAD state.
    pub current_branch: String,
    /// Commit ID of the current HEAD.
    pub last_commit_id: CommitId,
    /// First line of HEAD's commit message.
    pub last_commit_summary: String,
    /// Committer timestamp of HEAD.
    pub last_commit_timestamp: SystemTime,
}

/// Information about a single commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitInfo {
    /// Full SHA-1 commit identifier.
    pub commit_id: CommitId,
    /// Author name.
    pub author: String,
    /// First line of the commit message (subject line).
    pub summary: String,
    /// Author timestamp (matches the [`author`][Self::author] field).
    /// Saturated to [`UNIX_EPOCH`][std::time::UNIX_EPOCH] for pre-1970 values.
    pub timestamp: SystemTime,
}

/// Information about a lightweight Git tag.
///
/// Returned as elements of the [`Vec`] from
/// [`Repository::list_tags`][crate::repository::Repository::list_tags].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagInfo {
    /// Short tag name, e.g. `v1.0.0`.
    pub name: String,
    /// Full ref name, e.g. `refs/tags/v1.0.0`.
    pub full_name: String,
    /// Commit ID the tag points to (after peeling any tag objects).
    pub commit_id: CommitId,
    /// First line of the tagged commit's message.
    pub commit_summary: String,
    /// Committer timestamp of the tagged commit.
    pub commit_timestamp: SystemTime,
}
