//! Branch listing tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::repository;

#[test]
fn local_branches_single_main() {
    let f = Fixture::new();
    let branches = repository(f.path()).unwrap().local_branches().unwrap();
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
    assert!(branches[0].full_name.starts_with("refs/heads/"));
    assert_eq!(branches[0].last_commit_summary, "add feature");
}

#[test]
fn remote_branches_empty_when_no_remotes() {
    let f = Fixture::new();
    let branches = repository(f.path()).unwrap().remote_branches().unwrap();
    assert!(branches.is_empty());
}
