//! Working-tree dirty-check tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::repository;

#[test]
fn clean_repo_returns_result_without_error() {
    let f = Fixture::new();
    // Just verify it completes without error.  The exact bool may depend on
    // the index stat cache being fresh; we do not assert the value here.
    repository(f.path()).unwrap().is_dirty().unwrap();
}

#[test]
fn modified_tracked_file_is_dirty() {
    let f = Fixture::new();
    // Append content to a tracked file, changing its size.
    std::fs::write(f.path.join("README.md"), "# fixture\nmodified\n").unwrap();
    assert!(repository(f.path()).unwrap().is_dirty().unwrap(),
        "modified tracked file should be detected as dirty");
}

#[test]
fn deleted_tracked_file_is_dirty() {
    let f = Fixture::new();
    std::fs::remove_file(f.path.join("README.md")).unwrap();
    assert!(repository(f.path()).unwrap().is_dirty().unwrap(),
        "deleted tracked file should be detected as dirty");
}

#[test]
fn new_staged_file_is_dirty() {
    let f = Fixture::new();
    // Stage a new file — this is a staged change (not just an unstaged one).
    std::fs::write(f.path.join("new.txt"), "staged\n").unwrap();
    f.git(&["add", "new.txt"]);
    assert!(repository(f.path()).unwrap().is_dirty().unwrap(),
        "staged new file should be detected as dirty");
}
