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

// ── worktree_status: a stat mismatch is not a verdict ───────────────────── //

/// A repository copied without preserving timestamps (`cp -r`'s own
/// behaviour): every tracked file's mtime moves forward, content and size
/// unchanged. `git status` reports this clean; `worktree_status` must too.
#[test]
fn copied_repository_with_rewritten_mtimes_is_clean() {
    let f = Fixture::new();
    let later = f.mtime_secs("README.md") + 3600;
    f.set_mtime("README.md", later);
    f.set_mtime("src.rs", later);

    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    assert!(status.staged.is_empty(), "unexpected staged: {:?}", status.staged);
    assert!(
        status.unstaged.is_empty(),
        "a copied, content-identical file must not read as modified; unstaged: {:?}",
        status.unstaged
    );
}

/// `core.autocrlf=true`: the blob stores LF, the working tree holds CRLF.
/// `git status` reports this clean (it compares through the same
/// conversion); `worktree_status` must too — the raw bytes differ, but the
/// *git form* of the working tree file does not.
#[test]
fn autocrlf_checkout_is_clean() {
    let f = Fixture::new();
    f.git(&["config", "core.autocrlf", "true"]);
    std::fs::remove_file(f.path.join("README.md")).unwrap();
    f.git(&["checkout", "--", "README.md"]);

    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    assert!(
        status.unstaged.is_empty(),
        "an autocrlf checkout must not read as modified; unstaged: {:?}",
        status.unstaged
    );
}

/// `.gitattributes` `*.txt eol=crlf`: the same conversion as `core.autocrlf`,
/// scoped to one pattern instead of the whole repository.
#[test]
fn gitattributes_eol_crlf_checkout_is_clean() {
    let f = Fixture::new();
    std::fs::write(f.path.join(".gitattributes"), "*.txt eol=crlf\n").unwrap();
    f.git(&["add", ".gitattributes"]);
    f.git(&["commit", "-m", "add .gitattributes"]);
    std::fs::write(f.path.join("notes.txt"), "line one\nline two\n").unwrap();
    f.git(&["add", "notes.txt"]);
    f.git(&["commit", "-m", "add notes.txt"]);

    std::fs::remove_file(f.path.join("notes.txt")).unwrap();
    f.git(&["checkout", "--", "notes.txt"]);

    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    assert!(
        status.unstaged.is_empty(),
        "an eol=crlf checkout must not read as modified; unstaged: {:?}",
        status.unstaged
    );
}

/// A regression guard: rewriting a file's mtime must not make a *real*
/// edit invisible. The fix removes the stat-mismatch shortcut entirely
/// (every file is content-compared), so this should hold trivially — it
/// is asserted directly so a future change that reintroduces a shortcut
/// fails here first.
#[test]
fn real_edit_after_mtime_touch_is_still_modified() {
    let f = Fixture::new();
    let later = f.mtime_secs("README.md") + 3600;
    f.set_mtime("README.md", later);
    std::fs::write(f.path.join("README.md"), "# fixture\nactually edited\n").unwrap();

    let status = repository(f.path()).unwrap().worktree_status().unwrap();
    assert!(
        status.unstaged.iter().any(|e| e.kind == ChangeKind::Modified
            && e.path.ends_with("README.md")),
        "a genuine edit must still be detected; unstaged: {:?}", status.unstaged
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

// ── An unborn repository (no commits yet) has no .git/index ─────────────── //

/// A fresh `git init`, before the first `git add`, has no `.git/index` file
/// at all — confirmed directly against a real repository (`ls .git/index`:
/// no such file). `git status` still works on this, reading the missing
/// index as empty and listing every file as untracked. `worktree_status`
/// must read it the same way, not refuse because the file isn't there yet.
/// Built by hand, not through `Fixture::new()` — that fixture always
/// commits, which would create the very index file this test needs absent.
#[test]
fn unborn_repository_with_one_untracked_file_has_no_index_error() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path();
    let git = |args: &[&str]| {
        let ok = std::process::Command::new("git")
            .args(args)
            .current_dir(path)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_EDITOR", "true")
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(std::process::Stdio::null())
            .status()
            .expect("git")
            .success();
        assert!(ok, "git {} failed", args.join(" "));
    };
    git(&["init", "-b", "main"]);
    std::fs::write(path.join("new.txt"), "hello\n").unwrap();
    assert!(
        !path.join(".git/index").exists(),
        "fixture invariant: no git add has run yet, so no index file should exist"
    );

    let repo = repository(path).unwrap();
    let status = repo.worktree_status().unwrap();

    assert!(status.staged.is_empty(), "nothing is staged: {:?}", status.staged);
    assert!(status.unstaged.is_empty(), "nothing is tracked yet: {:?}", status.unstaged);
    assert!(
        status.untracked.iter().any(|p| p.ends_with("new.txt")),
        "the untracked file must still be listed: {:?}", status.untracked
    );
    assert!(!repo.is_dirty().unwrap(), "an untracked-only file is not dirty");
}

/// Parity with `git status --porcelain` on the same unborn-repository
/// fixture — endringer's own read comes first (same ordering rule the
/// copied-repository/symlink parity tests above use), `git` second, as a
/// sanity check.
#[test]
fn parity_unborn_repository_with_one_untracked_file() {
    #[path = "support/git_cli.rs"]
    mod git_cli;

    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path();
    let git = |args: &[&str]| {
        let ok = std::process::Command::new("git")
            .args(args)
            .current_dir(path)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_EDITOR", "true")
            .env("GIT_TERMINAL_PROMPT", "0")
            .stdin(std::process::Stdio::null())
            .status()
            .expect("git")
            .success();
        assert!(ok, "git {} failed", args.join(" "));
    };
    git(&["init", "-b", "main"]);
    std::fs::write(path.join("new.txt"), "hello\n").unwrap();

    let status = repository(path).unwrap().worktree_status().unwrap();
    assert!(
        status.untracked.iter().any(|p| p.ends_with("new.txt")),
        "endringer must list the untracked file: {:?}", status.untracked
    );

    let git_lines = git_cli::git_status_short(path);
    assert!(
        git_lines.iter().any(|l| l.contains("new.txt")),
        "git status --porcelain should also list it: {git_lines:?}"
    );
}
