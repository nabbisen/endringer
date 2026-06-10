//! RFC 003 acceptance test: VcsBackend default implementations.
//!
//! Verifies that a backend implementing only the required core methods
//! compiles and that all optional-empty defaults return sensible values,
//! and that write-side exception defaults return errors.

use std::time::SystemTime;

use anyhow::Result;
use endringer::VcsBackend;
use endringer::{
    AheadBehind, BlameEntry, BranchInfo, CommitId, CommitInfo, DiffSummary,
    SortOrder, StatusDigest, WorktreeStatus,
};

/// A minimal backend that implements only the required core methods.
/// Optional-empty methods are left to their defaults.
struct MinimalBackend;

impl VcsBackend for MinimalBackend {
    fn status_digest(&self) -> Result<StatusDigest> {
        anyhow::bail!("not implemented")
    }
    fn local_branches(&self) -> Result<Vec<BranchInfo>> {
        anyhow::bail!("not implemented")
    }
    fn remote_branches(&self) -> Result<Vec<BranchInfo>> {
        anyhow::bail!("not implemented")
    }
    fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        anyhow::bail!("not implemented")
    }
    fn list_commits_sorted(&self, _order: SortOrder) -> Result<Vec<CommitInfo>> {
        anyhow::bail!("not implemented")
    }
    fn log_since(&self, _since: SystemTime, _until: SystemTime) -> Result<Vec<CommitInfo>> {
        anyhow::bail!("not implemented")
    }
    fn find_commit(&self, _id: &CommitId) -> Result<CommitInfo> {
        anyhow::bail!("not implemented")
    }
    fn list_tags(&self) -> Result<Vec<endringer::TagInfo>> {
        anyhow::bail!("not implemented")
    }
    fn list_tags_sorted(&self, _order: SortOrder) -> Result<Vec<endringer::TagInfo>> {
        anyhow::bail!("not implemented")
    }
    fn diff(&self, _from: &CommitId, _to: &CommitId) -> Result<DiffSummary> {
        anyhow::bail!("not implemented")
    }
    fn is_dirty(&self) -> Result<bool> {
        anyhow::bail!("not implemented")
    }
    fn merge_base(&self, _a: &CommitId, _b: &CommitId) -> Result<Option<CommitId>> {
        anyhow::bail!("not implemented")
    }
    fn is_ancestor(&self, _candidate: &CommitId, _descendant: &CommitId) -> Result<bool> {
        anyhow::bail!("not implemented")
    }
    fn blame(&self, _path: &std::path::Path) -> Result<Vec<BlameEntry>> {
        anyhow::bail!("not implemented")
    }
    fn worktree_status(&self) -> Result<WorktreeStatus> {
        anyhow::bail!("not implemented")
    }
    fn file_at_commit(
        &self,
        _path: &std::path::Path,
        _commit_id: &CommitId,
    ) -> Result<Vec<u8>> {
        anyhow::bail!("not implemented")
    }
    fn ahead_behind(&self, _local: &CommitId, _upstream: &CommitId) -> Result<AheadBehind> {
        anyhow::bail!("not implemented")
    }
    fn repository_info(&self) -> Result<endringer::RepositoryInfo> {
        anyhow::bail!("not implemented")
    }
    // All optional methods left at their defaults.
}

// ── Optional-empty defaults return empty / None ───────────────────────────── //

#[test]
fn default_remote_url_is_none() {
    let b = MinimalBackend;
    assert_eq!(b.remote_url("origin"), None);
}

#[test]
fn default_submodules_is_empty() {
    let b = MinimalBackend;
    assert_eq!(b.submodules().unwrap(), vec![]);
}

#[test]
fn default_stash_entries_is_empty() {
    let b = MinimalBackend;
    assert_eq!(b.stash_entries().unwrap(), vec![]);
}

#[test]
fn default_worktrees_is_empty() {
    let b = MinimalBackend;
    assert_eq!(b.worktrees().unwrap(), vec![]);
}

// ── Write-side exception defaults return errors ───────────────────────────── //

#[test]
fn default_create_tag_returns_error() {
    let b = MinimalBackend;
    let err = b.create_tag("v0.1.0").unwrap_err();
    assert!(
        err.to_string().contains("does not support"),
        "error should mention 'does not support', got: {err}"
    );
}

#[test]
fn default_create_annotated_tag_returns_error() {
    let b = MinimalBackend;
    let err = b.create_annotated_tag("v0.1.0", "release").unwrap_err();
    assert!(err.to_string().contains("does not support"));
}

#[test]
fn default_delete_tag_returns_error() {
    let b = MinimalBackend;
    let err = b.delete_tag("v0.1.0").unwrap_err();
    assert!(err.to_string().contains("does not support"));
}

#[test]
fn default_branch_ahead_behind_returns_error() {
    let b = MinimalBackend;
    let err = b.branch_ahead_behind("main").unwrap_err();
    assert!(err.to_string().contains("does not support"));
}

#[test]
fn default_branch_tracking_returns_error() {
    let b = MinimalBackend;
    let err = b.branch_tracking("main").unwrap_err();
    assert!(err.to_string().contains("does not support"));
}

#[test]
fn default_local_branch_tracking_returns_error() {
    let b = MinimalBackend;
    let err = b.local_branch_tracking().unwrap_err();
    assert!(err.to_string().contains("does not support"));
}

#[test]
fn default_is_merged_into_returns_error() {
    let b = MinimalBackend;
    let err = b.is_merged_into("feature", "main").unwrap_err();
    assert!(err.to_string().contains("does not support"));
}

// ── Repository::with_backend works with minimal backend ───────────────────── //

#[test]
fn repository_with_minimal_backend_constructs() {
    use endringer::repository::Repository;
    use endringer::BackendKind;
    // Just verify it compiles and constructs without requiring any git state.
    let _repo = Repository::with_backend(
        Box::new(MinimalBackend),
        BackendKind::Git,
    );
}
