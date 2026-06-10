//! Integration tests for the rich status model (RFC 013).

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;
use endringer::{FileStatusKind, StatusOptions};

// ── clean repository ──────────────────────────────────────────────────────── //

#[test]
fn rich_status_clean_repo_has_no_entries() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    assert!(status.entries.is_empty(), "clean repo should have no entries");
}

// ── default options ────────────────────────────────────────────────────────── //

#[test]
fn rich_status_default_options_include_untracked() {
    let f = Fixture::new();
    std::fs::write(f.path.join("untracked.txt"), "hello").unwrap();

    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    assert!(
        status.entries.iter().any(|e|
            e.path.to_str().unwrap_or("").contains("untracked") &&
            e.worktree == Some(FileStatusKind::Untracked)
        ),
        "untracked file should appear with default options"
    );
}

#[test]
fn rich_status_no_untracked_when_disabled() {
    let f = Fixture::new();
    std::fs::write(f.path.join("untracked2.txt"), "hello").unwrap();

    let repo = repository(f.path()).unwrap();
    let opts = StatusOptions { include_untracked: false, include_ignored: false };
    let status = repo.rich_worktree_status(opts).unwrap();
    assert!(
        !status.entries.iter().any(|e|
            e.worktree == Some(FileStatusKind::Untracked)
        ),
        "untracked files should be excluded when include_untracked=false"
    );
}

// ── staged changes ────────────────────────────────────────────────────────── //

#[test]
fn rich_status_staged_new_file() {
    let f = Fixture::new();
    std::fs::write(f.path.join("new.rs"), "fn main() {}").unwrap();
    std::process::Command::new("git")
        .args(["add", "new.rs"]).current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .status().unwrap();

    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    let entry = status.entries.iter().find(|e| e.path.to_str() == Some("new.rs"));
    assert!(entry.is_some(), "new.rs should appear in rich status");
    assert_eq!(entry.unwrap().index, Some(FileStatusKind::Added),
        "staged new file should have index=Added");
}

#[test]
fn rich_status_staged_modification() {
    let f = Fixture::new();
    std::fs::write(f.path.join("README.md"), "modified\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "README.md"]).current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .status().unwrap();

    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    let entry = status.entries.iter().find(|e| e.path.to_str() == Some("README.md"));
    assert!(entry.is_some(), "README.md should appear in rich status");
    assert_eq!(entry.unwrap().index, Some(FileStatusKind::Modified),
        "staged modification should have index=Modified");
}

// ── unstaged changes ──────────────────────────────────────────────────────── //

#[test]
fn rich_status_unstaged_modification() {
    let f = Fixture::new();
    // Modify a tracked file without staging it.
    std::fs::write(f.path.join("README.md"), "unstaged change\n").unwrap();

    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    let entry = status.entries.iter().find(|e| e.path.to_str() == Some("README.md"));
    assert!(entry.is_some(), "README.md should appear in rich status");
    assert_eq!(entry.unwrap().worktree, Some(FileStatusKind::Modified),
        "unstaged modification should have worktree=Modified"
    );
}

// ── entries sorted by path ────────────────────────────────────────────────── //

#[test]
fn rich_status_entries_sorted_ascending() {
    let f = Fixture::new();
    // Create multiple untracked files to check sort order.
    for name in &["zzz.txt", "aaa.txt", "mmm.txt"] {
        std::fs::write(f.path.join(name), "x").unwrap();
    }

    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    let paths: Vec<&str> = status.entries.iter()
        .filter_map(|e| e.path.to_str())
        .collect();
    let sorted: Vec<&str> = { let mut v = paths.clone(); v.sort(); v };
    assert_eq!(paths, sorted, "rich status entries must be sorted ascending by path");
}

// ── both index and worktree changes ───────────────────────────────────────── //

#[test]
fn rich_status_both_staged_and_unstaged() {
    let f = Fixture::new();
    // Stage a new file, then modify it again without staging.
    std::fs::write(f.path.join("dual.txt"), "v1\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "dual.txt"]).current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .status().unwrap();
    std::fs::write(f.path.join("dual.txt"), "v2\n").unwrap();

    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    let entry = status.entries.iter().find(|e| e.path.to_str() == Some("dual.txt"));
    assert!(entry.is_some(), "dual.txt should appear");
    let e = entry.unwrap();
    assert_eq!(e.index, Some(FileStatusKind::Added),
        "dual.txt staged part should be Added");
    assert_eq!(e.worktree, Some(FileStatusKind::Modified),
        "dual.txt unstaged part should be Modified");
}

// ── conflict entries ──────────────────────────────────────────────────────── //

#[test]
fn rich_status_conflict_has_stages() {
    use std::process::Command;
    let f = Fixture::new();

    // Create a merge conflict.
    f.git(&["checkout", "-b", "branch-b"]);
    std::fs::write(f.path.join("conflict.txt"), "branch-b\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "branch-b change"]);

    let parent = String::from_utf8(
        Command::new("git").args(["rev-parse", "HEAD^"])
            .current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .output().unwrap().stdout
    ).unwrap().trim().to_owned();

    f.git(&["checkout", "main"]);
    std::fs::write(f.path.join("conflict.txt"), "main\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "main change"]);

    let _ = Command::new("git").args(["merge", "--no-ff", "branch-b"])
        .current_dir(f.path())
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null()).status();

    let _ = parent; // suppress warning

    if !f.path().join(".git").join("MERGE_HEAD").exists() {
        eprintln!("skip: merge conflict not produced");
        return;
    }

    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    let conflicted = status.entries.iter()
        .find(|e| e.conflict.is_some());
    assert!(conflicted.is_some(), "conflict entry should appear in rich status");
    let cs = conflicted.unwrap().conflict.as_ref().unwrap();
    assert!(!cs.stages.is_empty(), "conflict should have stages");
}

// ── ignored files ─────────────────────────────────────────────────────────── //

#[test]
fn rich_status_ignored_excluded_by_default() {
    let f = Fixture::new();
    std::fs::write(f.path.join(".gitignore"), "*.log\n").unwrap();
    std::fs::write(f.path.join("debug.log"), "noise").unwrap();

    let repo = repository(f.path()).unwrap();
    let status = repo.rich_worktree_status(StatusOptions::default()).unwrap();
    assert!(
        !status.entries.iter().any(|e|
            e.path.to_str().unwrap_or("").ends_with(".log") &&
            e.worktree == Some(FileStatusKind::Ignored)
        ),
        "ignored files should not appear with include_ignored=false"
    );
}
