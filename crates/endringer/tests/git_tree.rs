//! Integration tests for point-in-time reads and tree snapshots (RFC 010).

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;
use endringer::TreeEntryKind;

// ── Helpers ──────────────────────────────────────────────────────────────── //

fn rev_parse(f: &Fixture, rev: &str) -> String {
    let out = std::process::Command::new("git")
        .args(["rev-parse", rev])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output().unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

// ── tree_at_commit: root listing ──────────────────────────────────────────── //

#[test]
fn tree_at_commit_root_contains_readme() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let entries = repo.tree_at_commit(&commit_id).unwrap();
    assert!(
        entries.iter().any(|e| e.name == "README.md"),
        "root tree should contain README.md; entries: {entries:?}"
    );
}

#[test]
fn tree_at_commit_entries_are_sorted_ascending() {
    let f = Fixture::new();
    // Add a directory so we have mixed types.
    std::fs::create_dir(f.path.join("alpha")).unwrap();
    std::fs::write(f.path.join("alpha").join("z.rs"), "// z\n").unwrap();
    std::fs::write(f.path.join("00_first.txt"), "first\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "add directory and file"]);

    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let entries = repo.tree_at_commit(&commit_id).unwrap();
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    let sorted = {
        let mut v = names.clone();
        v.sort();
        v
    };
    assert_eq!(names, sorted, "tree_at_commit entries must be sorted ascending by name");
}

#[test]
fn tree_at_commit_entry_kinds() {
    let f = Fixture::new();
    // Add a directory and an executable file.
    std::fs::create_dir(f.path.join("subdir")).unwrap();
    std::fs::write(f.path.join("subdir").join("inner.txt"), "inner\n").unwrap();
    std::fs::write(f.path.join("script.sh"), "#!/bin/sh\n").unwrap();
    f.git(&["add", "."]);
    // Mark script.sh as executable (chmod on Unix only).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(f.path.join("script.sh")).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(f.path.join("script.sh"), perms).unwrap();
        f.git(&["update-index", "--chmod=+x", "script.sh"]);
    }
    f.git(&["commit", "-m", "add subdir and script"]);

    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let entries = repo.tree_at_commit(&commit_id).unwrap();

    let subdir_entry = entries.iter().find(|e| e.name == "subdir");
    assert!(subdir_entry.is_some(), "subdir should appear as a tree entry");
    assert_eq!(
        subdir_entry.unwrap().kind, TreeEntryKind::Directory,
        "subdir should have kind Directory"
    );

    let readme = entries.iter().find(|e| e.name == "README.md");
    assert!(readme.is_some(), "README.md should appear");
    assert_eq!(readme.unwrap().kind, TreeEntryKind::File);
    assert!(!readme.unwrap().executable, "README.md should not be executable");
}

#[test]
fn tree_at_commit_file_has_size() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let entries = repo.tree_at_commit(&commit_id).unwrap();
    let readme = entries.iter().find(|e| e.name == "README.md").unwrap();
    // File entries should have a size.
    assert!(
        readme.size.is_some(),
        "README.md should have a known size"
    );
    assert!(readme.size.unwrap() > 0, "README.md size should be > 0");
}

// ── tree_at_path: sub-directory listing ───────────────────────────────────── //

#[test]
fn tree_at_path_lists_subdirectory() {
    let f = Fixture::new();
    std::fs::create_dir(f.path.join("src")).unwrap();
    std::fs::write(f.path.join("src").join("lib.rs"),  "// lib\n").unwrap();
    std::fs::write(f.path.join("src").join("main.rs"), "// main\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "add src/"]);

    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let entries = repo
        .tree_at_path(&commit_id, std::path::Path::new("src"))
        .unwrap();

    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"lib.rs"),  "src/ should contain lib.rs");
    assert!(names.contains(&"main.rs"), "src/ should contain main.rs");
}

#[test]
fn tree_at_path_missing_directory_returns_err() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let err = repo
        .tree_at_path(&commit_id, std::path::Path::new("nonexistent"))
        .unwrap_err();
    // Should be PathNotFound or a Backend error mentioning the missing path.
    let msg = err.to_string();
    assert!(
        matches!(err, endringer::Error::PathNotFound { .. })
            || msg.contains("not found"),
        "missing path should return PathNotFound or a descriptive error; got: {msg}"
    );
}

#[test]
fn tree_at_path_root_equals_tree_at_commit() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let from_root  = repo.tree_at_commit(&commit_id).unwrap();
    let from_path  = repo.tree_at_path(&commit_id, std::path::Path::new("")).unwrap();
    // Both should return the same entries.
    assert_eq!(
        from_root.iter().map(|e| &e.name).collect::<Vec<_>>(),
        from_path.iter().map(|e| &e.name).collect::<Vec<_>>(),
        "tree_at_path(\"\") should match tree_at_commit"
    );
}

#[test]
fn tree_at_path_nested_directory() {
    let f = Fixture::new();
    std::fs::create_dir_all(f.path.join("a").join("b")).unwrap();
    std::fs::write(f.path.join("a").join("b").join("deep.rs"), "// deep\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "add a/b/deep.rs"]);

    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let entries = repo
        .tree_at_path(&commit_id, std::path::Path::new("a/b"))
        .unwrap();
    assert!(
        entries.iter().any(|e| e.name == "deep.rs"),
        "a/b/ should contain deep.rs"
    );
}

// ── tree_at_commit on historical commit ───────────────────────────────────── //

#[test]
fn tree_at_commit_historical_differs_from_head() {
    let f = Fixture::new();
    // Fixture starts with initial commit (README.md only) and HEAD (README.md + src.rs).
    let initial_hex = rev_parse(&f, "HEAD^");
    let head_hex    = rev_parse(&f, "HEAD");
    let initial = endringer::CommitId::from_hex(&initial_hex).unwrap();
    let head    = endringer::CommitId::from_hex(&head_hex).unwrap();

    let repo = repository(f.path()).unwrap();

    let initial_entries = repo.tree_at_commit(&initial).unwrap();
    let head_entries    = repo.tree_at_commit(&head).unwrap();

    // Initial commit has only README.md; HEAD also has src.rs.
    assert!(
        !initial_entries.iter().any(|e| e.name == "src.rs"),
        "initial commit should not contain src.rs"
    );
    assert!(
        head_entries.iter().any(|e| e.name == "src.rs"),
        "HEAD should contain src.rs"
    );
}

// ── blame_at ──────────────────────────────────────────────────────────────── //

#[test]
fn blame_at_head_matches_blame() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let blame_head    = repo.blame(std::path::Path::new("README.md")).unwrap();
    let blame_at_head = repo.blame_at(std::path::Path::new("README.md"), &commit_id).unwrap();

    // Both should produce the same entries (same commit, same file).
    assert_eq!(
        blame_head.len(), blame_at_head.len(),
        "blame and blame_at(HEAD) should return the same number of entries"
    );
}

#[test]
fn blame_at_old_commit_differs_from_head() {
    let f = Fixture::new();
    // Add a second version of README.md.
    std::fs::write(f.path.join("README.md"), "# fixture\nv2\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "update README"]);

    let repo = repository(f.path()).unwrap();
    let head_hex    = rev_parse(&f, "HEAD");
    let before_hex  = rev_parse(&f, "HEAD^");
    let head   = endringer::CommitId::from_hex(&head_hex).unwrap();
    let before = endringer::CommitId::from_hex(&before_hex).unwrap();

    let at_head   = repo.blame_at(std::path::Path::new("README.md"), &head).unwrap();
    let at_before = repo.blame_at(std::path::Path::new("README.md"), &before).unwrap();

    // The head version has more lines; they will have different blame entries.
    assert_ne!(
        at_head.len(), at_before.len(),
        "blame_at HEAD should differ from blame_at HEAD^: \
         head={} entries, before={} entries",
        at_head.len(), at_before.len()
    );
}

#[test]
fn blame_at_missing_file_returns_err() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let head_hex = rev_parse(&f, "HEAD");
    let commit_id = endringer::CommitId::from_hex(&head_hex).unwrap();

    let err = repo
        .blame_at(std::path::Path::new("no_such_file.rs"), &commit_id)
        .unwrap_err();
    // Should be some form of not-found error.
    assert!(
        !err.to_string().is_empty(),
        "missing file should return a non-empty error"
    );
}
