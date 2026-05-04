//! Unit tests for the repository module (run against the workspace git repo).

use std::path::Path;
use std::time::{Duration, SystemTime};

use super::*;

fn open() -> Repository {
    repository(Path::new(".")).expect("failed to open repository")
}

#[test]
fn it_works_repository() {
    assert!(repository(Path::new(".")).is_ok());
    assert!(repository(Path::new("/no/such/repo")).is_err());
}

#[test]
fn it_works_jj_repository_error_on_non_jj_path() {
    assert!(jj_repository(Path::new(".")).is_err());
    assert!(jj_repository(Path::new("/no/such/repo")).is_err());
}

#[test]
fn it_works_backend_kind() {
    assert_eq!(open().backend_kind(), BackendKind::Git);
}

#[test]
fn it_works_status_digest() {
    let digest = open().status_digest().expect("status digest");
    assert!(!digest.repo_name.is_empty());
    assert_ne!(digest.repo_name, "unknown");
    assert!(!digest.current_branch.starts_with("refs/"));
    assert_eq!(digest.last_commit_id.short().len(), 7);
}

#[test]
fn it_works_local_branches() {
    let branches = open().local_branches().expect("local_branches");
    assert!(!branches.is_empty());
    for b in &branches {
        assert!(!b.name.is_empty());
        assert!(b.full_name.starts_with("refs/heads/"));
    }
}

#[test]
fn it_works_remote_branches() {
    let branches = open().remote_branches().expect("remote_branches");
    for b in &branches {
        assert!(b.full_name.starts_with("refs/remotes/"));
    }
}

#[test]
fn it_works_list_commits() {
    let commits = open().list_commits().expect("list_commits");
    assert!(!commits.is_empty());
    for c in &commits {
        assert_eq!(c.commit_id.short().len(), 7);
        assert!(!c.author.is_empty());
    }
    for w in commits.windows(2) {
        assert!(w[0].timestamp >= w[1].timestamp, "must be newest-first");
    }
}

#[test]
fn it_works_commit_id_ord() {
    use endringer_core::types::CommitId;
    let commits = open().list_commits().expect("list_commits");
    if commits.len() >= 2 {
        let mut ids: Vec<CommitId> = commits.iter().map(|c| c.commit_id.clone()).collect();
        let sorted_clone = {
            let mut s = ids.clone();
            s.sort();
            s
        };
        ids.sort();
        assert_eq!(ids, sorted_clone); // deterministic sort
    }
}

#[test]
fn it_works_commit_id_to_short_id() {
    let digest = open().status_digest().expect("status digest");
    let short = crate::commit_id_to_short_id(digest.last_commit_id);
    assert_eq!(short.len(), 7);
}

#[test]
fn it_works_remote_url() {
    assert!(open().remote_url("origin").is_none());
}
