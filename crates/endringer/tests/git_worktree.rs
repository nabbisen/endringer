//! Linked worktree and tag annotation tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::repository;

// ── Linked worktrees ──────────────────────────────────────────────────────── //

#[test]
fn no_worktrees_returns_empty() {
    let f = Fixture::new();
    let wts = repository(f.path()).unwrap().worktrees().unwrap();
    assert!(wts.is_empty(), "plain fixture should have no linked worktrees");
}

#[test]
fn linked_worktree_appears_after_git_worktree_add() {
    let f = Fixture::new();
    // Create a second worktree in a sibling directory.
    let wt_path = f.path.parent().unwrap().join("linked-wt");
    f.git(&["worktree", "add", wt_path.to_str().unwrap(), "-b", "wt-branch"]);

    let wts = repository(f.path()).unwrap().worktrees().unwrap();
    assert_eq!(wts.len(), 1);
    assert_eq!(wts[0].current_branch, "wt-branch");
    assert!(!wts[0].is_locked);
    assert_eq!(wts[0].path.canonicalize().unwrap(), wt_path.canonicalize().unwrap());

    // Cleanup
    f.git(&["worktree", "remove", "--force", wt_path.to_str().unwrap()]);
}

#[test]
fn multiple_worktrees_sorted_by_id() {
    let f = Fixture::new();
    let wt_a = f.path.parent().unwrap().join("wt-alpha");
    let wt_z = f.path.parent().unwrap().join("wt-zeta");
    f.git(&["worktree", "add", wt_a.to_str().unwrap(), "-b", "branch-a"]);
    f.git(&["worktree", "add", wt_z.to_str().unwrap(), "-b", "branch-z"]);

    let wts = repository(f.path()).unwrap().worktrees().unwrap();
    assert_eq!(wts.len(), 2);
    assert!(wts[0].id <= wts[1].id, "worktrees should be sorted by id");

    f.git(&["worktree", "remove", "--force", wt_a.to_str().unwrap()]);
    f.git(&["worktree", "remove", "--force", wt_z.to_str().unwrap()]);
}

// ── Tag annotation metadata ───────────────────────────────────────────────── //

#[test]
fn lightweight_tag_has_no_annotation() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let tags = repo.list_tags().unwrap();
    let v010 = tags.iter().find(|t| t.name == "v0.1.0").unwrap();
    assert!(
        v010.annotation.is_none(),
        "lightweight tag should have no annotation"
    );
}

#[test]
fn annotated_tag_exposes_message_and_tagger() {
    let f = Fixture::new();
    // Set up git identity so the annotated tag has a tagger.
    f.git(&["config", "user.name", "Tagger"]);
    f.git(&["config", "user.email", "tagger@example.com"]);
    f.git(&["tag", "-a", "v1.0.0", "-m", "Release 1.0.0"]);

    let repo = repository(f.path()).unwrap();
    let tags = repo.list_tags().unwrap();
    let v100 = tags.iter().find(|t| t.name == "v1.0.0").unwrap();

    let ann = v100.annotation.as_ref().expect("annotated tag must have annotation");
    assert_eq!(ann.message, "Release 1.0.0");
    assert_eq!(ann.tagger_name.as_deref(), Some("Tagger"));
    assert!(ann.tagger_timestamp.is_some(), "annotated tag must have tagger timestamp");
}

#[test]
fn list_tags_sorted_includes_annotation_field() {
    let f = Fixture::new();
    f.git(&["config", "user.name", "Tagger"]);
    f.git(&["config", "user.email", "tagger@example.com"]);
    f.git(&["tag", "-a", "v2.0.0", "-m", "Major release"]);

    let repo = repository(f.path()).unwrap();
    let tags = repo.list_tags_sorted(endringer::SortOrder::ByName).unwrap();
    let v200 = tags.iter().find(|t| t.name == "v2.0.0").unwrap();
    assert!(v200.annotation.is_some());
}
