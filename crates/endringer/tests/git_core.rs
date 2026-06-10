//! Constructor and status tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::{jj_repository, repository};

#[test]
fn opens_valid_repo() {
    let f = Fixture::new();
    assert!(repository(f.path()).is_ok());
}

#[test]
fn rejects_missing_path() {
    assert!(repository(std::path::Path::new("/no/such/repo")).is_err());
}

#[test]
fn backend_kind_is_git() {
    use endringer::BackendKind;
    let f = Fixture::new();
    assert_eq!(repository(f.path()).unwrap().backend_kind(), BackendKind::Git);
}

#[test]
fn status_digest_fields() {
    let f = Fixture::new();
    let d = repository(f.path()).unwrap().status_digest().unwrap();
    assert_eq!(d.current_branch, "main");
    assert_eq!(d.last_commit_summary, "add feature");
    assert_eq!(d.last_commit_id.short().len(), 7);
}

#[test]
fn remote_url_none_when_no_remote() {
    let f = Fixture::new();
    assert!(repository(f.path()).unwrap().remote_url("origin").unwrap().is_none());
}

#[test]
fn jj_repository_rejects_plain_git() {
    let f = Fixture::new();
    assert!(jj_repository(f.path()).is_err());
}
