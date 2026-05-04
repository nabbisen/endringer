//! WorktreeStatus and file_at_commit tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use std::path::Path;

use endringer::repository::repository;
use endringer::ChangeKind;

// ── worktree_status ───────────────────────────────────────────────────────── //

#[test]
fn clean_repo_has_no_staged_or_unstaged() {
    let f = Fixture::new();
    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    // A freshly committed repo should have no staged or unstaged changes.
    assert!(status.staged.is_empty(),
        "unexpected staged: {:?}", status.staged);
    assert!(status.unstaged.is_empty(),
        "unexpected unstaged: {:?}", status.unstaged);
}

#[test]
fn modified_tracked_file_appears_in_unstaged() {
    let f = Fixture::new();
    std::fs::write(f.path.join("README.md"), "# changed content — size differs from original\n").unwrap();
    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    let modified: Vec<_> = status.unstaged.iter()
        .filter(|e| e.kind == ChangeKind::Modified)
        .collect();
    assert!(!modified.is_empty(), "README.md modification not detected");
    assert!(modified.iter().any(|e| e.path.ends_with("README.md")));
}

#[test]
fn deleted_tracked_file_appears_in_unstaged() {
    let f = Fixture::new();
    std::fs::remove_file(f.path.join("README.md")).unwrap();
    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    assert!(
        status.unstaged.iter().any(|e| e.kind == ChangeKind::Deleted
            && e.path.ends_with("README.md")),
        "README.md deletion not detected; unstaged: {:?}", status.unstaged
    );
}

#[test]
fn staged_new_file_appears_in_staged() {
    let f = Fixture::new();
    std::fs::write(f.path.join("new.txt"), "content\n").unwrap();
    f.git(&["add", "new.txt"]);
    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    assert!(
        status.staged.iter().any(|e| e.kind == ChangeKind::Added
            && e.path.ends_with("new.txt")),
        "staged new file not detected; staged: {:?}", status.staged
    );
}

#[test]
fn staged_modification_appears_in_staged() {
    let f = Fixture::new();
    std::fs::write(f.path.join("README.md"), "# modified\n").unwrap();
    f.git(&["add", "README.md"]);
    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    assert!(
        status.staged.iter().any(|e| e.kind == ChangeKind::Modified
            && e.path.ends_with("README.md")),
        "staged modification not detected; staged: {:?}", status.staged
    );
}

#[test]
fn untracked_file_reported() {
    let f = Fixture::new();
    std::fs::write(f.path.join("untracked.txt"), "hi\n").unwrap();
    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    assert!(
        status.untracked.iter().any(|p| p.ends_with("untracked.txt")),
        "untracked file not reported; untracked: {:?}", status.untracked
    );
}

#[test]
fn status_entries_are_sorted() {
    let f = Fixture::new();
    // Stage multiple files
    for name in &["z.txt", "a.txt", "m.txt"] {
        std::fs::write(f.path.join(name), "x").unwrap();
        f.git(&["add", name]);
    }
    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    for w in status.staged.windows(2) {
        assert!(w[0].path <= w[1].path, "staged not sorted");
    }
}

// ── file_at_commit ────────────────────────────────────────────────────────── //

#[test]
fn file_at_commit_reads_initial_readme() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap(); // [B, A]
    let initial = &commits[1]; // commit A added README.md

    let content = repo.file_at_commit(Path::new("README.md"), &initial.commit_id).unwrap();
    assert_eq!(content, b"# fixture\n");
}

#[test]
fn file_at_commit_reads_head_file() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let head = &repo.list_commits().unwrap()[0];

    let content = repo.file_at_commit(Path::new("src.rs"), &head.commit_id).unwrap();
    assert_eq!(content, b"fn main() {}\n");
}

#[test]
fn file_at_commit_missing_path_returns_error() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let id = &repo.list_commits().unwrap()[0].commit_id;
    assert!(repo.file_at_commit(Path::new("no_such.rs"), id).is_err());
}

#[test]
fn file_at_commit_wrong_commit_for_file() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let commits = repo.list_commits().unwrap();
    // src.rs was not yet added in commit A (initial commit)
    let initial = &commits[1];
    assert!(
        repo.file_at_commit(Path::new("src.rs"), &initial.commit_id).is_err(),
        "src.rs should not exist in the initial commit"
    );
}

#[test]
fn file_at_commit_nested_path() {
    let f = Fixture::new();
    // Create a file in a subdirectory
    std::fs::create_dir_all(f.path.join("src/util")).unwrap();
    std::fs::write(f.path.join("src/util/helper.rs"), "// helper\n").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "add nested file"]);

    let repo = repository(f.path()).unwrap();
    let head = &repo.list_commits().unwrap()[0];
    let content = repo
        .file_at_commit(Path::new("src/util/helper.rs"), &head.commit_id)
        .unwrap();
    assert_eq!(content, b"// helper\n");
}

// ── gitignore ────────────────────────────────────────────────────────────── //

#[test]
fn gitignored_file_not_in_untracked() {
    let f = Fixture::new();
    // Create a .gitignore that ignores *.log files.
    std::fs::write(f.path.join(".gitignore"), "*.log\n").unwrap();
    f.git(&["add", ".gitignore"]);
    f.git(&["commit", "-m", "add gitignore"]);

    // Write an ignored file and a non-ignored untracked file.
    std::fs::write(f.path.join("debug.log"), "log content\n").unwrap();
    std::fs::write(f.path.join("notes.txt"), "some notes\n").unwrap();

    let status = repository(f.path()).unwrap().worktree_status().unwrap();

    assert!(
        !status.untracked.iter().any(|p| p.ends_with("debug.log")),
        "ignored file should not appear in untracked: {:?}", status.untracked
    );
    assert!(
        status.untracked.iter().any(|p| p.ends_with("notes.txt")),
        "non-ignored untracked file should appear: {:?}", status.untracked
    );
}
