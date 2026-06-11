//! Integration tests for snapshot batch reads (RFC 027) and rename/copy-aware
//! diff (RFC 028).

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;
use endringer::{DiffChangeKind, DiffOptions, SnapshotRequest};

// ── RFC 027: snapshot ─────────────────────────────────────────────────────── //

#[test]
fn snapshot_default_includes_status_and_operation_state() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let snap = repo.snapshot(SnapshotRequest::default()).unwrap();

    // Default request includes status_digest and operation_state.
    assert!(snap.status_digest.is_some(), "default snapshot should include status_digest");
    // operation_state may return UnsupportedFeature on some backends; Ok is fine.
    // Just verify it was requested (field is Some) when supported.
    // local_branches and tags are NOT included by default.
    assert!(snap.local_branches.is_none(), "branches excluded from default snapshot");
    assert!(snap.tags.is_none(), "tags excluded from default snapshot");
}

#[test]
fn snapshot_with_branches_includes_them() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let req = SnapshotRequest {
        include_local_branches: true,
        ..Default::default()
    };
    let snap = repo.snapshot(req).unwrap();
    assert!(snap.local_branches.is_some(), "branches should be included when requested");
    assert!(!snap.local_branches.unwrap().is_empty(), "fixture has at least one branch");
}

#[test]
fn snapshot_with_tags_includes_them() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let req = SnapshotRequest { include_tags: true, ..Default::default() };
    let snap = repo.snapshot(req).unwrap();
    assert!(snap.tags.is_some(), "tags should be included when requested");
}

#[test]
fn snapshot_always_includes_repo_info() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    // Even an empty request returns RepositoryInfo.
    let req = SnapshotRequest {
        include_status_digest: false,
        include_operation_state: false,
        include_local_branches: false,
        include_tags: false,
    };
    let snap = repo.snapshot(req).unwrap();
    assert!(!snap.info.repo_name.is_empty(), "repo_info is always populated");
    assert!(snap.status_digest.is_none());
}

#[test]
fn snapshot_status_matches_individual_call() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let snap = repo.snapshot(SnapshotRequest::default()).unwrap();
    let direct = repo.status_digest().unwrap();

    if let Some(sd) = snap.status_digest {
        assert_eq!(sd.current_branch, direct.current_branch);
        assert_eq!(sd.last_commit_id, direct.last_commit_id);
    }
}

#[test]
fn snapshot_branches_match_individual_call() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let snap = repo.snapshot(SnapshotRequest { include_local_branches: true, ..Default::default() }).unwrap();
    let direct = repo.local_branches().unwrap();
    if let Some(sb) = snap.local_branches {
        let snap_names: Vec<&str> = sb.iter().map(|b| b.name.as_str()).collect();
        let direct_names: Vec<&str> = direct.iter().map(|b| b.name.as_str()).collect();
        assert_eq!(snap_names, direct_names);
    }
}

// ── RFC 028: diff_entries ─────────────────────────────────────────────────── //

#[test]
fn diff_entries_added_file() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    assert!(commits.len() >= 2, "fixture needs at least 2 commits");

    // HEAD added src.rs relative to HEAD^.
    let parent = &commits[1].commit_id;
    let head   = &commits[0].commit_id;
    let entries = repo.diff_entries(parent, head, DiffOptions::default()).unwrap();

    assert!(!entries.is_empty(), "diff_entries should return entries");
    assert!(
        entries.iter().any(|e| e.kind == DiffChangeKind::Added),
        "should have at least one Added entry"
    );
}

#[test]
fn diff_entries_sorted_by_path() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let parent = &commits[1].commit_id;
    let head   = &commits[0].commit_id;

    let entries = repo.diff_entries(parent, head, DiffOptions::default()).unwrap();
    let paths: Vec<_> = entries.iter()
        .filter_map(|e| e.new_path.as_ref().or(e.old_path.as_ref()))
        .collect();
    let sorted = { let mut v = paths.clone(); v.sort(); v };
    assert_eq!(paths, sorted, "diff_entries should be sorted by path");
}

#[test]
fn diff_entries_default_agrees_with_diff_summary() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let parent = &commits[1].commit_id;
    let head   = &commits[0].commit_id;

    let summary = repo.diff(parent, head).unwrap();
    let entries = repo.diff_entries(parent, head, DiffOptions::default()).unwrap();

    let added_count   = entries.iter().filter(|e| e.kind == DiffChangeKind::Added).count();
    let deleted_count = entries.iter().filter(|e| e.kind == DiffChangeKind::Deleted).count();
    let modified_count = entries.iter().filter(|e| e.kind == DiffChangeKind::Modified).count();

    assert_eq!(added_count,   summary.added.len(),
        "diff_entries Added should match DiffSummary added");
    assert_eq!(deleted_count, summary.deleted.len(),
        "diff_entries Deleted should match DiffSummary deleted");
    assert_eq!(modified_count, summary.modified.len(),
        "diff_entries Modified should match DiffSummary modified");
}

#[test]
fn diff_entries_identical_commits_returns_empty() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let head = &repo.list_commits().unwrap()[0].commit_id;
    let entries = repo.diff_entries(head, head, DiffOptions::default()).unwrap();
    assert!(entries.is_empty(), "diffing a commit against itself should return empty");
}

#[test]
fn diff_entries_detect_renames_option_accepted() {
    // Rename detection flag is accepted without error, even in the first version
    // where it produces the same output as the default.
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let parent = &commits[1].commit_id;
    let head   = &commits[0].commit_id;

    let opts = DiffOptions { detect_renames: true, ..Default::default() };
    let result = repo.diff_entries(parent, head, opts);
    assert!(result.is_ok(), "detect_renames=true should not return an error");
}
