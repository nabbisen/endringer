//! Submodule listing and stash entry tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::repository;
use endringer::ChangeKind;

// ── submodules ────────────────────────────────────────────────────────────── //

/// Adds `source` (any plain repo — `Fixture::new()` itself is a fine
/// submodule source: it is just a repo with commits) as an initialized
/// submodule of `parent`, at `name`, and commits the result.
/// `protocol.file.allow=always` is needed from Git 2.38.1 on, which
/// otherwise refuses a local-path submodule for the same reason it refuses
/// `file://` clones by default (CVE-2022-39253) — both source and target
/// are this test's own tempdirs, never anything untrusted.
fn add_submodule(parent: &Fixture, source: &Fixture, name: &str) {
    parent.git(&[
        "-c", "protocol.file.allow=always",
        "submodule", "add", source.path().to_str().unwrap(), name,
    ]);
    parent.git(&["commit", "-m", &format!("add submodule {name}")]);
}

#[test]
fn no_submodules_returns_empty() {
    let f = Fixture::new();
    let subs = repository(f.path()).unwrap().submodules().unwrap();
    assert!(subs.is_empty(), "plain fixture should have no submodules");
}

#[test]
fn submodule_appears_after_adding_gitmodules() {
    // Create a minimal .gitmodules file to verify the parser.
    let f = Fixture::new();
    let gitmodules = "[submodule \"lib\"]\n\tpath = lib\n\turl = https://example.com/lib.git\n";
    std::fs::write(f.path.join(".gitmodules"), gitmodules).unwrap();
    f.git(&["add", ".gitmodules"]);
    f.git(&["commit", "-m", "add submodule config"]);

    let subs = repository(f.path()).unwrap().submodules().unwrap();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].name, "lib");
    assert_eq!(subs[0].path.to_str().unwrap(), "lib");
    assert_eq!(subs[0].url.as_deref(), Some("https://example.com/lib.git"));
}

#[test]
fn multiple_submodules_are_sorted_by_path() {
    let f = Fixture::new();
    let gitmodules = concat!(
        "[submodule \"z-mod\"]\n\tpath = zzz\n\turl = https://z.example.com/z.git\n",
        "[submodule \"a-mod\"]\n\tpath = aaa\n\turl = https://a.example.com/a.git\n",
    );
    std::fs::write(f.path.join(".gitmodules"), gitmodules).unwrap();
    f.git(&["add", ".gitmodules"]);
    f.git(&["commit", "-m", "multi submodule"]);

    let subs = repository(f.path()).unwrap().submodules().unwrap();
    assert_eq!(subs.len(), 2);
    assert!(subs[0].path < subs[1].path, "submodules not sorted by path");
    assert_eq!(subs[0].path.to_str().unwrap(), "aaa");
}

// ── Submodule status, not just .gitmodules parsing ──────────────────────── //

#[test]
fn clean_repository_with_initialized_submodule_reads_clean_and_nothing_untracked() {
    let source = Fixture::new();
    let parent = Fixture::new();
    add_submodule(&parent, &source, "lib");

    let ws = repository(parent.path()).unwrap().worktree_status().unwrap();
    assert!(ws.unstaged.is_empty(), "unexpected unstaged: {:?}", ws.unstaged);
    assert!(
        ws.untracked.is_empty(),
        "the submodule's own files must not appear as the parent's untracked: {:?}",
        ws.untracked
    );
}

#[test]
fn submodule_head_moved_reads_modified() {
    let source = Fixture::new();
    let parent = Fixture::new();
    add_submodule(&parent, &source, "lib");

    // Advance the submodule's own checkout past what the parent's gitlink
    // recorded, without telling the parent (no `git add`/`commit` in
    // `parent` for this) — exactly "the submodule's own HEAD moved".
    let sub_path = parent.path.join("lib");
    std::fs::write(sub_path.join("new.txt"), "new\n").unwrap();
    let git_in_sub = |args: &[&str]| {
        let ok = std::process::Command::new("git").args(args).current_dir(&sub_path)
            .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_EDITOR", "true").env("GIT_TERMINAL_PROMPT", "0")
            .stdin(std::process::Stdio::null())
            .status().unwrap().success();
        assert!(ok, "git {args:?} in submodule failed");
    };
    git_in_sub(&["config", "user.email", "fixture@test.local"]);
    git_in_sub(&["config", "user.name", "Fixture"]);
    git_in_sub(&["add", "new.txt"]);
    git_in_sub(&["commit", "-m", "advance submodule HEAD"]);

    let ws = repository(parent.path()).unwrap().worktree_status().unwrap();
    assert!(
        ws.unstaged.iter().any(|e| e.kind == ChangeKind::Modified && e.path.ends_with("lib")),
        "a submodule whose HEAD moved must read Modified: {:?}", ws.unstaged
    );
}

#[test]
fn file_edited_inside_submodule_reads_modified() {
    let source = Fixture::new();
    let parent = Fixture::new();
    add_submodule(&parent, &source, "lib");

    // HEAD stays exactly what the gitlink recorded; only the submodule's
    // own working tree gets dirty.
    std::fs::write(parent.path.join("lib").join("README.md"), "# fixture\nedited\n").unwrap();

    let ws = repository(parent.path()).unwrap().worktree_status().unwrap();
    assert!(
        ws.unstaged.iter().any(|e| e.kind == ChangeKind::Modified && e.path.ends_with("lib")),
        "an edit inside the submodule must read the submodule itself Modified: {:?}", ws.unstaged
    );
}

#[test]
fn uninitialized_submodule_reads_clean() {
    let source = Fixture::new();
    let parent = Fixture::new();
    add_submodule(&parent, &source, "lib");

    // `git submodule deinit` removes the checkout (the `.git` marker and
    // the submodule's own files) but keeps the gitlink in the index and
    // the config in `.gitmodules` — exactly "uninitialized", not "removed".
    parent.git(&["submodule", "deinit", "-f", "lib"]);

    let ws = repository(parent.path()).unwrap().worktree_status().unwrap();
    assert!(
        ws.unstaged.is_empty(),
        "an uninitialized submodule must read clean, not Modified: {:?}", ws.unstaged
    );
}

// ── A submodule that cannot be read is not clean ────────────────────────── //

#[test]
fn broken_submodule_gitfile_reads_modified_not_compared_with_superproject() {
    let source = Fixture::new();
    let parent = Fixture::new();
    add_submodule(&parent, &source, "lib");

    // Break the submodule's own gitfile so it cannot be opened at all —
    // `git status` on the superproject fails outright here ("fatal: ...
    // gitfile does not point to a valid repository").
    let gitfile = parent.path.join("lib").join(".git");
    std::fs::write(&gitfile, "gitdir: ../../nowhere/not-a-repo\n").unwrap();

    let ws = repository(parent.path()).unwrap().worktree_status().unwrap();
    assert!(
        ws.unstaged.iter().any(|e| e.kind == ChangeKind::Modified && e.path.ends_with("lib")),
        "a submodule whose own repository cannot be opened must read Modified: {:?}",
        ws.unstaged
    );
    // Not compared with the superproject: the verdict is scoped to "lib"
    // alone. (Opening the submodule at its own path, `gix::open`, never
    // escalates to the superproject's own `.git` the way `gix::discover`
    // used to — so nothing from the parent's own HEAD/history can leak
    // into this result as a second entry.)
    assert_eq!(
        ws.unstaged.len(),
        1,
        "the broken submodule must be the only unstaged entry: {:?}",
        ws.unstaged
    );
}

#[test]
fn submodule_with_unborn_head_reads_modified() {
    let source = Fixture::new();
    let parent = Fixture::new();
    add_submodule(&parent, &source, "lib");

    // `checkout --orphan` moves HEAD to a symbolic ref with no commit at
    // all yet — an initialized submodule whose HEAD cannot be peeled to a
    // commit, so there is nothing to match the gitlink against.
    let sub_path = parent.path.join("lib");
    let git_in_sub = |args: &[&str]| {
        let ok = std::process::Command::new("git").args(args).current_dir(&sub_path)
            .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_EDITOR", "true").env("GIT_TERMINAL_PROMPT", "0")
            .stdin(std::process::Stdio::null())
            .status().unwrap().success();
        assert!(ok, "git {args:?} in submodule failed");
    };
    git_in_sub(&["checkout", "--orphan", "unborn"]);

    let ws = repository(parent.path()).unwrap().worktree_status().unwrap();
    assert!(
        ws.unstaged.iter().any(|e| e.kind == ChangeKind::Modified && e.path.ends_with("lib")),
        "an unborn HEAD in an initialized submodule must read Modified: {:?}",
        ws.unstaged
    );
}

// ── stash ─────────────────────────────────────────────────────────────────── //

#[test]
fn empty_stash_returns_empty_vec() {
    let f = Fixture::new();
    let entries = repository(f.path()).unwrap().stash_entries().unwrap();
    assert!(entries.is_empty(), "fresh repo should have no stash entries");
}

#[test]
fn stash_entry_appears_after_git_stash() {
    let f = Fixture::new();
    // Make a dirty change, then stash it.
    std::fs::write(f.path.join("README.md"), "# changed\n\nextra line\n").unwrap();
    f.git(&["stash", "push", "-m", "test stash message"]);

    let entries = repository(f.path()).unwrap().stash_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].index, 0);
    assert_eq!(entries[0].commit_id.short().len(), 7);
    assert!(!entries[0].message.is_empty());
}

#[test]
fn multiple_stash_entries_are_newest_first() {
    let f = Fixture::new();

    std::fs::write(f.path.join("README.md"), "version A\n").unwrap();
    f.git(&["stash", "push", "-m", "stash A"]);

    std::fs::write(f.path.join("README.md"), "version B\n").unwrap();
    f.git(&["stash", "push", "-m", "stash B"]);

    let entries = repository(f.path()).unwrap().stash_entries().unwrap();
    assert_eq!(entries.len(), 2);
    // Newest first: stash B should be stash@{0}
    assert_eq!(entries[0].index, 0);
    assert_eq!(entries[1].index, 1);
    assert!(entries[0].message.contains('B'),
        "newest stash should come first; messages: {:?}", 
        entries.iter().map(|e| &e.message).collect::<Vec<_>>());
}
