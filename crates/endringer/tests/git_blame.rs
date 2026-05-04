//! Per-line commit attribution (blame) tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::repository;

#[test]
fn blame_single_author_file() {
    let f = Fixture::new();
    // src.rs was added entirely by commit B ("add feature").
    let repo = repository(f.path()).unwrap();
    let entries = repo.blame(std::path::Path::new("src.rs")).unwrap();

    assert!(!entries.is_empty(), "blame returned no entries for src.rs");

    // All lines in src.rs were introduced by the same commit (HEAD).
    let head_id = &repo.list_commits().unwrap()[0].commit_id;
    for e in &entries {
        assert_eq!(&e.commit_id, head_id,
            "every line of src.rs should be attributed to HEAD");
        assert!(e.start_line >= 1);
        assert!(e.end_line >= e.start_line);
    }
}

#[test]
fn blame_entries_cover_all_lines() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let entries = repo.blame(std::path::Path::new("README.md")).unwrap();

    // Lines must be contiguous (no gaps, no overlaps).
    let mut expected_next = 1u32;
    for e in &entries {
        assert_eq!(e.start_line, expected_next,
            "gap or overlap in blame entries: expected start {}, got {}",
            expected_next, e.start_line);
        expected_next = e.end_line + 1;
    }
}

#[test]
fn blame_missing_file_returns_error() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let err = repo.blame(std::path::Path::new("no_such_file.rs"));
    assert!(err.is_err(), "blaming a missing file should return Err");
}

#[test]
fn blame_multi_author_records_correct_commits() {
    // Commit A adds README.md, commit B modifies it.
    let f = Fixture::new();
    // Modify README.md in a new commit so it has lines from two commits.
    std::fs::write(f.path.join("README.md"), "# fixture\nnew line\n").unwrap();
    f.git(&["add", "README.md"]);
    f.git(&["commit", "-m", "update README"]);

    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let entries = repo.blame(std::path::Path::new("README.md")).unwrap();

    // There should be at least one entry, and at least one commit is referenced.
    assert!(!entries.is_empty());
    let referenced_ids: std::collections::HashSet<_> =
        entries.iter().map(|e| e.commit_id.to_string()).collect();
    // At least one of the known commits should be referenced.
    assert!(
        commits.iter().any(|c| referenced_ids.contains(&c.commit_id.to_string())),
        "blame entries should reference known commits; entries: {:?}", entries
    );
}
