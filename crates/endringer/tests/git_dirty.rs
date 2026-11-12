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

// ── is_dirty: a stat mismatch is not a verdict ──────────────────────────── //

/// `touch`ing a tracked file (mtime only, content untouched) must not read
/// as dirty — the single-file analogue of
/// `copied_repository_with_rewritten_mtimes_is_clean` (`git_status.rs`).
#[test]
fn touched_tracked_file_is_not_dirty() {
    let f = Fixture::new();
    let later = f.mtime_secs("README.md") + 3600;
    f.set_mtime("README.md", later);
    assert!(!repository(f.path()).unwrap().is_dirty().unwrap(),
        "a touched, content-identical file should not be dirty");
}

/// A regression guard for the content-hash fallback this project already
/// had working: a same-size edit, with the file's mtime pinned back to
/// exactly what the index cached (simulating an edit that lands in the
/// same clock second the index already recorded — not left to wall-clock
/// timing luck). Stat alone says clean; content does not, and must still
/// win.
#[test]
fn same_size_same_second_edit_is_still_dirty() {
    let f = Fixture::new();
    let cached = f.mtime_secs("README.md");
    std::fs::write(f.path.join("README.md"), "# fixturd\n").unwrap(); // same length as "# fixture\n"
    f.set_mtime("README.md", cached);
    assert!(repository(f.path()).unwrap().is_dirty().unwrap(),
        "a same-size, same-cached-mtime edit must still be detected as dirty");
}
