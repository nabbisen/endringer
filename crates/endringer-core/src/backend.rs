//! The [`VcsBackend`] trait that all VCS backends implement.
//!
//! This trait is `pub` so that downstream crates can implement custom
//! backends and inject them via [`endringer::repository::Repository::with_backend`].
//! The trait signature may change before v1.0.

use std::time::SystemTime;

use anyhow::Result;

use crate::types::{BranchInfo, CommitId, CommitInfo, DiffSummary, SortOrder, StatusDigest, TagInfo};

/// Common interface implemented by every VCS backend.
///
/// All methods take `&self` and must be safe to call from multiple threads
/// (`Send + Sync` bound).
pub trait VcsBackend: Send + Sync {
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
