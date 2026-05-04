//! Trait abstracting over VCS backends (Git, Jujutsu, …).
//!
//! Downstream crates only depend on the concrete [`Repository`] type and the
//! public types in [`crate::types`].  This trait is `pub(crate)` and is an
//! implementation detail.

use std::time::SystemTime;

use anyhow::Result;

use crate::types::{BranchInfo, CommitId, CommitInfo, DiffSummary, SortOrder, StatusDigest, TagInfo};

/// Common interface implemented by every VCS backend.
///
/// All methods take `&self` and may be called from multiple threads.
/// Implementations must be `Send + Sync`.
pub(crate) trait VcsBackend: Send + Sync {
    // ── Status ─────────────────────────────────────────────────────────── //
    fn status_digest(&self) -> Result<StatusDigest>;

    // ── Branches ───────────────────────────────────────────────────────── //
    fn local_branches(&self) -> Result<Vec<BranchInfo>>;
    fn remote_branches(&self) -> Result<Vec<BranchInfo>>;

    // ── Commits ────────────────────────────────────────────────────────── //
    fn list_commits(&self) -> Result<Vec<CommitInfo>>;
    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>>;
    fn log_since(&self, since: SystemTime, until: SystemTime) -> Result<Vec<CommitInfo>>;
    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo>;

    // ── Tags ───────────────────────────────────────────────────────────── //
    fn list_tags(&self) -> Result<Vec<TagInfo>>;
    fn list_tags_sorted(&self, order: SortOrder) -> Result<Vec<TagInfo>>;
    fn create_tag(&self, name: &str) -> Result<()>;
    fn create_annotated_tag(&self, name: &str, message: &str) -> Result<()>;
    fn delete_tag(&self, name: &str) -> Result<()>;

    // ── Diff ───────────────────────────────────────────────────────────── //
    fn diff(&self, from: &CommitId, to: &CommitId) -> Result<DiffSummary>;

    // ── Remotes ────────────────────────────────────────────────────────── //
    fn remote_url(&self, name: &str) -> Option<String>;
}
