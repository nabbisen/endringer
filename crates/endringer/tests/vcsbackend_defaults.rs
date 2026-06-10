//! RFC 003 + RFC 006 acceptance test: VcsBackend default implementations.
//!
//! Verifies that:
//! - A backend implementing only the required core methods compiles.
//! - All optional-empty defaults return sensible values.
//! - Write-side exception defaults return typed `Error::UnsupportedBackendFeature`.

use std::time::SystemTime;

use endringer::{Error, NotFoundKind, Result, VcsBackend};
use endringer::{
    AheadBehind, BlameEntry, BranchInfo, BranchTrackingInfo, CommitId, CommitInfo,
    DiffSummary, RepositoryInfo, SortOrder, StatusDigest, WorktreeStatus,
};

/// A minimal backend implementing only the required core methods.
/// All optional methods use their trait defaults.
struct MinimalBackend;

impl VcsBackend for MinimalBackend {
    fn status_digest(&self) -> Result<StatusDigest> { Err(Error::EmptyRepository) }
    fn local_branches(&self) -> Result<Vec<BranchInfo>> { Ok(vec![]) }
    fn remote_branches(&self) -> Result<Vec<BranchInfo>> { Ok(vec![]) }
    fn list_commits(&self) -> Result<Vec<CommitInfo>> { Ok(vec![]) }
    fn list_commits_sorted(&self, _: SortOrder) -> Result<Vec<CommitInfo>> { Ok(vec![]) }
    fn log_since(&self, _: SystemTime, _: SystemTime) -> Result<Vec<CommitInfo>> { Ok(vec![]) }
    fn find_commit(&self, id: &CommitId) -> Result<CommitInfo> {
        Err(Error::NotFound { kind: NotFoundKind::Commit, name: id.to_string() })
    }
    fn list_tags(&self) -> Result<Vec<endringer::TagInfo>> { Ok(vec![]) }
    fn list_tags_sorted(&self, _: SortOrder) -> Result<Vec<endringer::TagInfo>> { Ok(vec![]) }
    fn diff(&self, _: &CommitId, _: &CommitId) -> Result<DiffSummary> { Ok(DiffSummary::default()) }
    fn is_dirty(&self) -> Result<bool> { Ok(false) }
    fn merge_base(&self, _: &CommitId, _: &CommitId) -> Result<Option<CommitId>> { Ok(None) }
    fn is_ancestor(&self, _: &CommitId, _: &CommitId) -> Result<bool> { Ok(false) }
    fn blame(&self, _: &std::path::Path) -> Result<Vec<BlameEntry>> { Ok(vec![]) }
    fn worktree_status(&self) -> Result<WorktreeStatus> { Ok(WorktreeStatus::default()) }
    fn file_at_commit(&self, path: &std::path::Path, commit: &CommitId) -> Result<Vec<u8>> {
        Err(Error::PathNotFound { path: path.to_path_buf(), commit: Some(commit.clone()) })
    }
    fn ahead_behind(&self, _: &CommitId, _: &CommitId) -> Result<AheadBehind> {
        Err(Error::UnsupportedBackendFeature { backend: None, feature: "ahead_behind" })
    }
    fn repository_info(&self) -> Result<RepositoryInfo> {
        Err(Error::UnsupportedBackendFeature { backend: None, feature: "repository_info" })
    }
    // All optional methods use their defaults.
}

// ── Optional-empty defaults ───────────────────────────────────────────────── //

#[test]
fn default_remote_url_is_ok_none() {
    let b = MinimalBackend;
    assert_eq!(b.remote_url("origin").unwrap(), None);
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

// ── Write-side exception defaults return typed errors ─────────────────────── //

#[test]
fn default_create_tag_returns_unsupported() {
    let b = MinimalBackend;
    let err = b.create_tag("v0.1.0").unwrap_err();
    assert!(matches!(err, Error::UnsupportedBackendFeature { backend: None, feature: "create_tag" }));
}

#[test]
fn default_create_annotated_tag_returns_unsupported() {
    let b = MinimalBackend;
    let err = b.create_annotated_tag("v0.1.0", "release").unwrap_err();
    assert!(matches!(err, Error::UnsupportedBackendFeature { .. }));
}

#[test]
fn default_delete_tag_returns_unsupported() {
    let b = MinimalBackend;
    let err = b.delete_tag("v0.1.0").unwrap_err();
    assert!(matches!(err, Error::UnsupportedBackendFeature { .. }));
}

#[test]
fn default_branch_ahead_behind_returns_unsupported() {
    let b = MinimalBackend;
    let err = b.branch_ahead_behind("main").unwrap_err();
    assert!(matches!(err, Error::UnsupportedBackendFeature { .. }));
}

#[test]
fn default_branch_tracking_returns_unsupported() {
    let b = MinimalBackend;
    let err = b.branch_tracking("main").unwrap_err();
    assert!(matches!(err, Error::UnsupportedBackendFeature { .. }));
}

#[test]
fn default_local_branch_tracking_returns_unsupported() {
    let b = MinimalBackend;
    let err = b.local_branch_tracking().unwrap_err();
    assert!(matches!(err, Error::UnsupportedBackendFeature { .. }));
}

#[test]
fn default_is_merged_into_returns_unsupported() {
    let b = MinimalBackend;
    let err = b.is_merged_into("feature", "main").unwrap_err();
    assert!(matches!(err, Error::UnsupportedBackendFeature { .. }));
}

// ── Repository::with_backend works with MinimalBackend ───────────────────── //

#[test]
fn repository_with_minimal_backend_constructs() {
    use endringer::repository::Repository;
    use endringer::BackendKind;
    let _repo = Repository::with_backend(Box::new(MinimalBackend), BackendKind::Git);
}

// ── Error enum matches variants (not strings) ─────────────────────────────── //

#[test]
fn error_display_is_useful() {
    let e = Error::NotFound { kind: NotFoundKind::Commit, name: "abc1234".into() };
    let s = e.to_string();
    assert!(s.contains("commit") && s.contains("abc1234"), "Display: {s}");
}

#[test]
fn error_not_a_repository_display() {
    let e = Error::NotARepository { path: "/tmp/nope".into() };
    assert!(e.to_string().contains("not a repository"), "{}", e);
}

#[test]
fn error_unsupported_feature_jj() {
    let e = Error::UnsupportedBackendFeature {
        backend: Some(endringer::BackendKind::Jj),
        feature: "create_annotated_tag",
    };
    assert!(matches!(
        e,
        Error::UnsupportedBackendFeature {
            backend: Some(endringer::BackendKind::Jj),
            feature: "create_annotated_tag"
        }
    ));
}
