//! Git CLI parity tests (RFC 015).
//!
//! Each test verifies that endringer's output matches the corresponding
//! `git` plumbing/porcelain command on the same fixture.
//!
//! See `tests/parity/KNOWN-DEVIATIONS.md` for documented deviations.

#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

#[path = "support/git_cli.rs"]
mod git_cli;

use endringer::repository::repository;
use endringer::SortOrder;

// ── Helper ────────────────────────────────────────────────────────────────── //

fn rev_parse(f: &Fixture, r: &str) -> String {
    git_cli::git_output(f.path(), &["rev-parse", r])
}

// ── 1. merge_base ────────────────────────────────────────────────────────── //

#[test]
fn parity_merge_base() {
    let f = Fixture::new();
    // Create a diverged branch.
    f.git(&["checkout", "-b", "branch-x"]);
    std::fs::write(f.path.join("bx.txt"), "bx").unwrap();
    f.git(&["add", "."]);
    f.git(&["commit", "-m", "branch-x commit"]);
    f.git(&["checkout", "main"]);

    let main_sha = rev_parse(&f, "main");
    let bx_sha   = rev_parse(&f, "branch-x");

    let git_mb = git_cli::git_merge_base(f.path(), "main", "branch-x")
        .expect("git merge-base should succeed");

    let repo = repository(f.path()).unwrap();
    let main_id = endringer::CommitId::from_hex(&main_sha).unwrap();
    let bx_id   = endringer::CommitId::from_hex(&bx_sha).unwrap();
    let endo_mb = repo.merge_base(&main_id, &bx_id).unwrap()
        .expect("merge_base should return Some");

    assert_eq!(
        git_mb,
        endo_mb.to_string(),
        "merge_base should match git merge-base"
    );
}

// ── 2. is_ancestor ───────────────────────────────────────────────────────── //

#[test]
fn parity_is_ancestor() {
    let f = Fixture::new();
    let initial = rev_parse(&f, "HEAD^");
    let head    = rev_parse(&f, "HEAD");

    let initial_id = endringer::CommitId::from_hex(&initial).unwrap();
    let head_id    = endringer::CommitId::from_hex(&head).unwrap();

    let git_result = git_cli::git_is_ancestor(f.path(), &initial, &head);
    let repo = repository(f.path()).unwrap();
    let endo_result = repo.is_ancestor(&initial_id, &head_id).unwrap();

    assert_eq!(git_result, endo_result,
        "is_ancestor should match git merge-base --is-ancestor");

    // Also verify: HEAD is NOT an ancestor of HEAD^ (reverse is false).
    let git_rev   = git_cli::git_is_ancestor(f.path(), &head, &initial);
    let endo_rev  = repo.is_ancestor(&head_id, &initial_id).unwrap();
    assert_eq!(git_rev, endo_rev,
        "reverse is_ancestor should match git");
}

// ── 3. ahead/behind ─────────────────────────────────────────────────────── //

#[test]
fn parity_ahead_behind() {
    let f = Fixture::new();
    // Create a branch 2 commits ahead of main.
    f.git(&["checkout", "-b", "ahead-branch"]);
    for i in 0..2 {
        std::fs::write(f.path.join(format!("a{i}.txt")), "x").unwrap();
        f.git(&["add", "."]);
        f.git(&["commit", "-m", &format!("ahead {i}")]);
    }
    f.git(&["checkout", "main"]);

    let (git_a, git_b) = git_cli::git_ahead_behind(f.path(), "ahead-branch", "main");

    let repo = repository(f.path()).unwrap();
    let ab_id = endringer::CommitId::from_hex(&rev_parse(&f, "ahead-branch")).unwrap();
    let main_id = endringer::CommitId::from_hex(&rev_parse(&f, "main")).unwrap();
    let ab = repo.ahead_behind(&ab_id, &main_id).unwrap();

    assert_eq!(git_a, ab.ahead,  "ahead count should match git rev-list");
    assert_eq!(git_b, ab.behind, "behind count should match git rev-list");
}

// ── 4. tag listing ──────────────────────────────────────────────────────── //

#[test]
fn parity_tag_names() {
    let f = Fixture::new();
    // Add a second tag.
    f.git(&["tag", "v0.2.0"]);

    let git_tags: Vec<String> = {
        let mut v = git_cli::git_tag_names(f.path());
        v.sort();
        v
    };

    let repo = repository(f.path()).unwrap();
    let mut endo_tags: Vec<String> = repo.list_tags_sorted(SortOrder::ByName)
        .unwrap()
        .into_iter()
        .map(|t| t.name)
        .collect();
    endo_tags.sort();

    assert_eq!(git_tags, endo_tags,
        "tag names should match git for-each-ref refs/tags");
}

// ── 5. local branch listing ─────────────────────────────────────────────── //

#[test]
fn parity_branch_names() {
    let f = Fixture::new();
    f.git(&["checkout", "-b", "feature-a"]);
    f.git(&["checkout", "main"]);

    let git_branches: Vec<String> = {
        let mut v = git_cli::git_branch_names(f.path());
        v.sort();
        v
    };

    let repo = repository(f.path()).unwrap();
    let mut endo_branches: Vec<String> = repo.local_branches()
        .unwrap()
        .into_iter()
        .map(|b| b.name)
        .collect();
    endo_branches.sort();

    assert_eq!(git_branches, endo_branches,
        "branch names should match git for-each-ref refs/heads");
}

// ── 6. blame line count ──────────────────────────────────────────────────── //

#[test]
fn parity_blame_line_count() {
    let f = Fixture::new();
    // README.md has exactly 1 line in the fixture.
    // git blame --porcelain outputs one 40-char SHA line per blamed line.
    let git_blame_output = git_cli::git_output(f.path(),
        &["blame","--porcelain","README.md"]);
    // Each blame entry starts with a 40-hex SHA followed by line numbers.
    // Count lines that start with a hex char and have at least 40 chars.
    let git_line_count = git_blame_output.lines()
        .filter(|l| {
            let b = l.as_bytes();
            b.len() >= 40 && b[0..40].iter().all(|c| c.is_ascii_hexdigit())
        })
        .count();

    let repo = repository(f.path()).unwrap();
    let endo_blame = repo.blame(std::path::Path::new("README.md")).unwrap();

    let endo_line_count: u32 = endo_blame.iter()
        .map(|e| e.end_line - e.start_line + 1)
        .sum();

    assert_eq!(git_line_count, endo_line_count as usize,
        "blame line count should match git blame --porcelain; \
         git={git_line_count}, endo={endo_line_count}");
}
