//! Commit listing, lookup, sorting, ancestry, and commit-graph tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use std::time::{Duration, SystemTime};

use endringer::repository::repository;
use endringer::SortOrder;

#[test]
fn list_commits_newest_first() {
    let f = Fixture::new();
    let commits = repository(f.path()).unwrap().list_commits().unwrap();
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].summary, "add feature");
    assert_eq!(commits[1].summary, "initial commit");
    assert!(commits[0].timestamp >= commits[1].timestamp);
    for c in &commits {
        assert_eq!(c.author, "Fixture");
        assert_eq!(c.committer, "Fixture");
    }
}

#[test]
fn list_commits_sorted_newest_first() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let newest = repo.list_commits_sorted(SortOrder::NewestFirst).unwrap();
    assert!(newest[0].timestamp >= newest[1].timestamp);
}

#[test]
fn list_commits_sorted_oldest_first() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let oldest = repo.list_commits_sorted(SortOrder::OldestFirst).unwrap();
    assert!(oldest[0].timestamp <= oldest[1].timestamp);
}

#[test]
fn list_commits_sorted_same_ids_both_orders() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let newest = repo.list_commits_sorted(SortOrder::NewestFirst).unwrap();
    let oldest = repo.list_commits_sorted(SortOrder::OldestFirst).unwrap();
    let mut n: Vec<_> = newest.iter().map(|c| c.commit_id.to_string()).collect();
    let mut o: Vec<_> = oldest.iter().map(|c| c.commit_id.to_string()).collect();
    n.sort(); o.sort();
    assert_eq!(n, o);
}

#[test]
fn log_since_recent_commits() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let until = SystemTime::now();
    let since = until - Duration::from_secs(3600);
    let commits = repo.log_since(since, until).unwrap();
    assert!(!commits.is_empty());
    for c in &commits {
        assert!(c.timestamp >= since && c.timestamp <= until);
    }
}

#[test]
fn log_since_ancient_range_is_empty() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let ancient = SystemTime::UNIX_EPOCH + Duration::from_secs(86400);
    let empty = repo.log_since(SystemTime::UNIX_EPOCH, ancient).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn find_commit_roundtrip() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let list = repo.list_commits().unwrap();
    let found = repo.find_commit(&list[0].commit_id).unwrap();
    assert_eq!(found.commit_id, list[0].commit_id);
    assert_eq!(found.summary, list[0].summary);
    assert_eq!(found.author, "Fixture");
}

#[test]
fn commit_info_parents() {
    let f = Fixture::new();
    let commits = repository(f.path()).unwrap().list_commits().unwrap();
    // HEAD (B) has A as parent
    assert_eq!(commits[0].parents.len(), 1);
    assert_eq!(commits[0].parents[0], commits[1].commit_id);
    // Initial commit has no parents
    assert!(commits[1].parents.is_empty());
}

#[test]
fn commit_id_ord_is_deterministic() {
    let f = Fixture::new();
    let commits = repository(f.path()).unwrap().list_commits().unwrap();
    let mut ids: Vec<_> = commits.iter().map(|c| c.commit_id.clone()).collect();
    let snap = ids.clone();
    ids.sort();
    let mut snap2 = snap;
    snap2.sort();
    assert_eq!(ids, snap2);
}

// ── Commit-graph helpers ─────────────────────────────────────────────────── //

#[test]
fn merge_base_linear_history() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let b = &commits[0]; // HEAD
    let a = &commits[1]; // initial

    // merge_base(B, A) should be A (A is ancestor of B)
    let base = repo.merge_base(&b.commit_id, &a.commit_id).unwrap();
    assert_eq!(base, Some(a.commit_id.clone()));
}

#[test]
fn merge_base_same_commit() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let id = &commits[0].commit_id;
    let base = repo.merge_base(id, id).unwrap();
    assert_eq!(base, Some(id.clone()));
}

#[test]
fn is_ancestor_direct() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let b = &commits[0].commit_id; // HEAD
    let a = &commits[1].commit_id; // initial

    assert!(repo.is_ancestor(a, b).unwrap()); // A is ancestor of B
    assert!(!repo.is_ancestor(b, a).unwrap()); // B is NOT ancestor of A
}

#[test]
fn is_ancestor_self() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let id = &repo.list_commits().unwrap()[0].commit_id;
    assert!(repo.is_ancestor(id, id).unwrap());
}
