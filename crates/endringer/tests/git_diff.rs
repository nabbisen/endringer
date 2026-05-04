//! Diff summary tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::repository;

#[test]
fn diff_between_commits_adds_file() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap(); // [B, A]

    let d = repo.diff(&commits[1].commit_id, &commits[0].commit_id).unwrap();
    assert!(d.added.iter().any(|p| p.ends_with("src.rs")),
        "src.rs should appear as added; got {:?}", d.added);
    assert!(d.modified.is_empty());
    assert!(d.deleted.is_empty());
}

#[test]
fn diff_same_commit_is_empty() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let id = &repo.list_commits().unwrap()[0].commit_id;
    let d = repo.diff(id, id).unwrap();
    assert!(d.added.is_empty() && d.modified.is_empty() && d.deleted.is_empty());
}

#[test]
fn diff_paths_are_sorted() {
    let f = Fixture::new();
    // Add several files in one commit so we get multiple diff entries.
    std::fs::write(f.path.join("z.rs"), "").unwrap();
    std::fs::write(f.path.join("a.rs"), "").unwrap();
    std::fs::write(f.path.join("m.rs"), "").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "multi-file"]);

    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let d = repo.diff(&commits[1].commit_id, &commits[0].commit_id).unwrap();

    for w in d.added.windows(2) {
        assert!(w[0] <= w[1], "added paths not sorted: {:?}", d.added);
    }
}
