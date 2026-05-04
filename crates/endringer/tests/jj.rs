//! Jujutsu backend tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::jj_repository;

#[test]
fn rejects_plain_git_repo() {
    let f = Fixture::new();
    assert!(jj_repository(f.path()).is_err(),
        "plain git repo must be rejected by jj_repository");
}

#[test]
fn rejects_nonexistent_path() {
    assert!(jj_repository(std::path::Path::new("/no/such/path")).is_err());
}

#[test]
fn rejects_empty_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    assert!(jj_repository(dir.path()).is_err());
}
