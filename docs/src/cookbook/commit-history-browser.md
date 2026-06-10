# Cookbook: Commit history browser

**When to use this pattern.** You are building a commit log view, a "git log"
panel, or a code-review tool that needs to page through history and show
per-commit diffs.

## API calls

```rust
repo.query_commits(CommitQuery::head_page(n))  // first n commits, truncated flag
repo.find_commit(id)                            // single commit by OID
repo.diff(from, to)                             // DiffSummary between two commits
repo.file_at_commit(path, commit_id)            // file bytes at a historical commit
repo.tree_at_commit(commit_id)                  // root tree listing at a commit
repo.blame_at(path, commit_id)                  // per-line attribution at a commit
```

## Minimal example

```rust,no_run
use endringer::{repository, CommitQuery, CommitQueryStart, SortOrder};
use std::path::Path;

fn show_log(path: &Path, page: usize, per_page: usize) -> anyhow::Result<()> {
    let repo = repository(path)?;

    let result = repo.query_commits(endringer::CommitQuery {
        start: CommitQueryStart::Head,
        max_count: Some(per_page),
        skip: page * per_page,
        since: None, until: None,
        order: SortOrder::NewestFirst,
    })?;

    for commit in &result.commits {
        println!("{} {}", commit.commit_id.short(), commit.summary);
    }
    if result.truncated { println!("... more commits"); }
    Ok(())
}
```

## Expanding a commit

```rust,no_run
let commit = repo.find_commit(id.clone())?;
let diff   = if let Some(parent) = commit.parents.first() {
    repo.diff(parent, &commit.commit_id)?
} else {
    // Initial commit: compare empty tree. diff() accepts the same commit OID
    // for both sides and returns an empty diff; produce the added list manually
    // via tree_at_commit instead.
    let tree = repo.tree_at_commit(&commit.commit_id)?;
    endringer::DiffSummary {
        added:    tree.iter().map(|e| e.path.clone()).collect(),
        modified: vec![],
        deleted:  vec![],
    }
};
```

## Cost notes

- `query_commits()` with a small `max_count` is efficient — the walk stops
  early. Offset-based `skip` is O(skip) in the walk, so very deep pages on
  large histories are slower.
- `diff()` reads two commit trees; cheap for small diffs, linear in changed
  files.
- `blame_at()` and `file_at_commit()` both load blobs — suitable for on-demand
  expansion but not bulk prefetch.

## Boundary note

Pagination policy (page size, key-based vs offset), UI presentation, and
keyboard navigation are consumer-owned. endringer provides the raw commits;
the consumer decides how to present them.
