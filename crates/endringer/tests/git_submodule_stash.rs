//! Submodule listing and stash entry tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::repository;

// ── submodules ────────────────────────────────────────────────────────────── //

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
