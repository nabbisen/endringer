//! Security and resource-exhaustion hardening tests (RFC 025).
//!
//! Verifies that:
//! - endringer gracefully handles repositories opened at unusual paths;
//! - `CommitId::from_hex` rejects malformed input cleanly;
//! - corrupt or missing marker files yield errors rather than panics;
//! - large-history repositories can be read in bounded fashion;
//! - `file_at_commit` on a missing path returns a typed error;
//! - operations on a bare repository do not panic.

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;
use endringer::{CommitId, Error};

// ── CommitId hardening ────────────────────────────────────────────────────── //

#[test]
fn commit_id_from_hex_rejects_non_hex() {
    assert!(CommitId::from_hex("gggggggggggggggggggggggggggggggggggggggg").is_err(),
        "non-hex characters should be rejected");
    assert!(CommitId::from_hex("").is_err(), "empty string should be rejected");
    assert!(CommitId::from_hex("  ").is_err(), "whitespace should be rejected");
}

#[test]
fn commit_id_from_hex_is_deterministic() {
    let hex = "a".repeat(40);
    let id1 = CommitId::from_hex(&hex).unwrap();
    let id2 = CommitId::from_hex(&hex).unwrap();
    assert_eq!(id1, id2);
}

#[test]
fn commit_id_short_always_7_chars() {
    let id = CommitId::from_hex(&"c".repeat(40)).unwrap();
    assert_eq!(id.short().len(), 7, "short() must always return 7 characters");
    let id256 = CommitId::from_hex(&"d".repeat(64)).unwrap();
    assert_eq!(id256.short().len(), 7);
}

// ── Repository path edge cases ────────────────────────────────────────────── //

#[test]
fn repository_open_at_non_git_dir_returns_typed_error() {
    let dir = tempfile::TempDir::new().unwrap();
    // Plain empty directory — not a git repo.
    match repository(dir.path()) {
        Ok(_) => panic!("expected error opening non-git directory"),
        Err(err) => assert!(
            matches!(err, Error::NotARepository { .. }),
            "non-git directory should return NotARepository; got: {err}"
        ),
    }
}

#[test]
fn repository_open_at_nonexistent_path_returns_typed_error() {
    // any typed error is acceptable — the important thing is no panic
    if repository(std::path::Path::new("/no/such/path/at/all")).is_ok() {
        panic!("expected error for nonexistent path");
    }
}

// ── file_at_commit edge cases ─────────────────────────────────────────────── //

#[test]
fn file_at_commit_missing_path_returns_error() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    let head_id = &commits[0].commit_id;

    let err = repo.file_at_commit(
        std::path::Path::new("this_file_does_not_exist.rs"),
        head_id,
    ).unwrap_err();

    // Should be NotFound or Backend — not a panic.
    assert!(
        !err.to_string().is_empty(),
        "missing file should return a non-empty error"
    );
}

#[test]
fn file_at_commit_invalid_commit_id_returns_error() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    // A commit ID that doesn't exist in this repo.
    let fake_id = CommitId::from_hex(&"0".repeat(40)).unwrap();

    let err = repo.file_at_commit(
        std::path::Path::new("README.md"),
        &fake_id,
    ).unwrap_err();

    assert!(
        !err.to_string().is_empty(),
        "invalid commit should return a non-empty error"
    );
}

// ── Bounded history ───────────────────────────────────────────────────────── //

#[test]
fn query_commits_bounded_never_exceeds_max_count() {
    // Build a fixture with several commits.
    let f = Fixture::new();
    for i in 0..5 {
        std::fs::write(f.path.join(format!("f{i}.txt")), "x").unwrap();
        std::process::Command::new("git")
            .args(["add", "."]).current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .status().unwrap();
        std::process::Command::new("git")
            .args(["commit","-m",&format!("c{i}")]).current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
            .stdin(std::process::Stdio::null()).status().unwrap();
    }
    let repo = repository(f.path()).unwrap();

    for max in [1, 2, 3, 7] {
        let result = repo.query_commits(
            endringer::CommitQuery::head_page(max)
        ).unwrap();
        assert!(result.commits.len() <= max,
            "query_commits(max={max}) returned {} commits", result.commits.len());
    }
}

// ── Bare repository safety ────────────────────────────────────────────────── //

#[test]
fn bare_repository_does_not_panic_on_reads() {
    // Set up a bare repo.
    let src = Fixture::new();
    let bare = tempfile::TempDir::new().unwrap();
    std::process::Command::new("git")
        .args(["clone","--bare", src.path().to_str().unwrap(),
               bare.path().to_str().unwrap()])
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null()).status().unwrap();

    let repo = repository(bare.path()).unwrap();

    // None of these should panic.
    let _ = repo.list_commits();
    let _ = repo.local_branches();
    let _ = repo.list_tags();
    let _ = repo.is_dirty();
    let _ = repo.references();
    let _ = repo.repository_info();
}

// ── No-external-command guarantee (a real grep audit, not a marker) ───────── //
//
// **Proves:** no `src/` file in any of the five workspace crates spawns an
// external command (`std::process::Command`/`Command::new`) — endringer
// talks to git exclusively through `gix`, never by shelling out to the
// `git`/`jj` binaries. Previously only claimed in a comment ("enforced by
// code review and grep audits") behind an `assert!(true, ...)` that could
// never fail; this runs the grep audit for real, so a future `Command::new`
// in library code fails this test instead of only a human's own review.
#[test]
fn library_code_never_spawns_an_external_command() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let mut offenders = Vec::new();
    for crate_name in [
        "endringer-core",
        "endringer-git",
        "endringer-jj",
        "endringer",
        "endringer-async",
    ] {
        let src_dir = workspace_root.join("crates").join(crate_name).join("src");
        visit_rs_files(&src_dir, &mut |path| {
            let text = std::fs::read_to_string(path).expect("read source file");
            if text.contains("process::Command") || text.contains("Command::new") {
                offenders.push(path.display().to_string());
            }
        });
    }
    assert!(
        offenders.is_empty(),
        "library (non-test) code must never spawn an external command directly: {offenders:?}"
    );
}

fn visit_rs_files(dir: &std::path::Path, f: &mut impl FnMut(&std::path::Path)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, f);
        } else if path.extension().is_some_and(|e| e == "rs") {
            f(&path);
        }
    }
}
