//! Integration tests for remote and reference inventory (RFC 011).

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;
use endringer::repository::repository;
use endringer::{RefKind, RefTarget};

// ── remotes() ────────────────────────────────────────────────────────────── //

#[test]
fn remotes_empty_when_no_remote_configured() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let remotes = repo.remotes().unwrap();
    assert!(remotes.is_empty(), "fixture has no remotes; expected empty list");
}

#[test]
fn remotes_single_origin() {
    // Create a bare "remote" repo and clone to get origin.
    let f = Fixture::new();
    let bare = tempfile::TempDir::new().unwrap();
    std::process::Command::new("git")
        .args(["clone", "--bare", f.path().to_str().unwrap(),
               bare.path().to_str().unwrap()])
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null()).status().unwrap();

    let work = tempfile::TempDir::new().unwrap();
    std::process::Command::new("git")
        .args(["clone", bare.path().to_str().unwrap(),
               work.path().to_str().unwrap()])
        .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
        .env("GIT_EDITOR","true").env("GIT_TERMINAL_PROMPT","0")
        .stdin(std::process::Stdio::null()).status().unwrap();

    let repo = repository(work.path()).unwrap();
    let remotes = repo.remotes().unwrap();

    assert_eq!(remotes.len(), 1, "cloned repo should have exactly one remote");
    assert_eq!(remotes[0].name, "origin");
    assert!(!remotes[0].fetch_urls.is_empty(), "origin should have a fetch URL");
}

#[test]
fn remotes_sorted_ascending_by_name() {
    let f = Fixture::new();
    // Add two remotes in reverse order.
    let git = |args: &[&str]| {
        std::process::Command::new("git").args(args).current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .env("GIT_TERMINAL_PROMPT","0").status().unwrap();
    };
    git(&["remote", "add", "zzz-remote", "https://example.com/zzz"]);
    git(&["remote", "add", "aaa-remote", "https://example.com/aaa"]);

    let repo = repository(f.path()).unwrap();
    let remotes = repo.remotes().unwrap();
    let names: Vec<&str> = remotes.iter().map(|r| r.name.as_str()).collect();
    let sorted = { let mut v = names.clone(); v.sort(); v };
    assert_eq!(names, sorted, "remotes must be sorted ascending by name");
}

#[test]
fn remotes_push_url_when_explicitly_set() {
    let f = Fixture::new();
    let git = |args: &[&str]| {
        std::process::Command::new("git").args(args).current_dir(f.path())
            .env("GIT_CONFIG_NOSYSTEM","1").env("GIT_CONFIG_GLOBAL","/dev/null")
            .env("GIT_TERMINAL_PROMPT","0").status().unwrap();
    };
    git(&["remote", "add", "upstream", "https://example.com/fetch"]);
    git(&["remote", "set-url", "--push", "upstream", "https://example.com/push"]);

    let repo = repository(f.path()).unwrap();
    let remotes = repo.remotes().unwrap();
    let upstream = remotes.iter().find(|r| r.name == "upstream")
        .expect("upstream remote should exist");

    assert!(!upstream.fetch_urls.is_empty(), "fetch URL should be present");
    assert!(!upstream.push_urls.is_empty(), "explicit push URL should be present");
    assert_ne!(upstream.fetch_urls[0], upstream.push_urls[0],
        "fetch and push URLs should differ");
}

// ── references() ─────────────────────────────────────────────────────────── //

#[test]
fn references_contains_main_branch() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let refs = repo.references().unwrap();

    assert!(
        refs.iter().any(|r| r.name == "refs/heads/main"),
        "references should contain refs/heads/main; got: {refs:?}"
    );
}

#[test]
fn references_contains_head() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let refs = repo.references().unwrap();

    let head = refs.iter().find(|r| r.name == "HEAD");
    assert!(head.is_some(), "references should include HEAD");
    assert_eq!(head.unwrap().kind, RefKind::Head);
}

#[test]
fn references_sorted_ascending() {
    let f = Fixture::new();
    // Fixture already has v0.1.0 tag. Add an extra branch for diversity.
    f.git(&["checkout", "-b", "feature-x"]);
    f.git(&["checkout", "main"]);

    let repo = repository(f.path()).unwrap();
    let refs = repo.references().unwrap();
    let names: Vec<&str> = refs.iter().map(|r| r.name.as_str()).collect();
    let sorted = { let mut v = names.clone(); v.sort(); v };
    assert_eq!(names, sorted, "references must be sorted ascending by full name");
}

#[test]
fn references_contains_tag() {
    let f = Fixture::new();
    // Fixture creates tag v0.1.0 in Fixture::new().
    let repo = repository(f.path()).unwrap();
    let refs = repo.references().unwrap();

    let tag = refs.iter().find(|r| r.name == "refs/tags/v0.1.0");
    assert!(tag.is_some(), "references should contain the v0.1.0 tag");
    assert_eq!(tag.unwrap().kind, RefKind::Tag);
    assert!(
        matches!(tag.unwrap().target, RefTarget::Direct(_)),
        "lightweight tag should have a Direct target"
    );
}

#[test]
fn head_target_is_symbolic_on_main() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let refs = repo.references().unwrap();

    let head = refs.iter().find(|r| r.name == "HEAD").unwrap();
    assert!(
        matches!(&head.target, RefTarget::Symbolic(s) if s.contains("main")),
        "HEAD should be symbolic pointing to main; got {:?}", head.target
    );
}

// ── references_by_kind() ─────────────────────────────────────────────────── //

#[test]
fn references_by_kind_local_branches() {
    let f = Fixture::new();
    f.git(&["checkout", "-b", "dev"]);
    f.git(&["checkout", "main"]);

    let repo = repository(f.path()).unwrap();
    let branches = repo.references_by_kind(RefKind::LocalBranch).unwrap();

    assert!(
        branches.iter().all(|r| r.kind == RefKind::LocalBranch),
        "all entries should be LocalBranch"
    );
    assert!(
        branches.iter().any(|r| r.name == "refs/heads/main"),
        "should contain refs/heads/main"
    );
    assert!(
        branches.iter().any(|r| r.name == "refs/heads/dev"),
        "should contain refs/heads/dev"
    );
}

#[test]
fn references_by_kind_tags() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let tags = repo.references_by_kind(RefKind::Tag).unwrap();

    assert!(tags.iter().all(|r| r.kind == RefKind::Tag));
    assert!(tags.iter().any(|r| r.name == "refs/tags/v0.1.0"));
}

#[test]
fn references_by_kind_head() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let heads = repo.references_by_kind(RefKind::Head).unwrap();

    assert_eq!(heads.len(), 1, "should return exactly one HEAD entry");
    assert_eq!(heads[0].name, "HEAD");
}
