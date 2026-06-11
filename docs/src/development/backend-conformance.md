# Custom backend conformance

This page documents what a correct `VcsBackend` implementation must satisfy.
It supplements the trait documentation and the cookbook's
[Custom backend](../cookbook/custom-backend.md) page.

## Stability note

`VcsBackend` is **not yet stable** (pre-v1.0). Its method set may change.
Methods with default implementations are safe to skip; new defaulted methods
added in future releases will not break existing custom backends. Required
methods (those without a default) are the stable core.

## Required methods (no default)

These must be implemented:

| Method | Minimum correct behaviour |
|---|---|
| `status_digest` | Returns repo name, branch, HEAD commit, summary, timestamp |
| `is_dirty` | `false` for clean repos, `true` when unstaged/staged changes exist |
| `worktree_status` | Staged, unstaged, untracked lists; empty when clean |
| `local_branches` | Returns at least the current branch; sorted ascending by full name |
| `remote_branches` | Empty vec is valid if no remotes exist |
| `list_commits` | Commits newest-first; empty vec for empty/unborn repos |
| `list_commits_sorted` | Applies `SortOrder` consistently |
| `log_since` | Inclusive timestamp filter on author timestamp |
| `find_commit` | Returns `NotFound` for unknown IDs; O(1) lookup preferred |
| `list_tags` | Empty vec is valid; sorted by insertion order |
| `list_tags_sorted` | Applies `SortOrder` consistently |
| `diff` | Returns changed paths; empty when identical |
| `blame` | Returns one entry per source line; start/end are 1-indexed |
| `file_at_commit` | Returns raw bytes; `NotFound` for missing paths |
| `submodules` | Empty vec is valid when no submodules |
| `stash_entries` | Newest first; empty when no stash |
| `worktrees` | Empty vec is valid; main worktree excluded |
| `remote_url` | `Ok(None)` when remote does not exist |
| `repository_info` | Reports correct backend, format, head state |
| `backend_kind` | Returns the enum value passed to `with_backend` |

## Optional methods (with defaults)

These have `UnsupportedBackendFeature` defaults. Override only if your
backend supports the operation:

- `operation_state` — in-progress Git operation detection
- `unmerged_paths` / `conflict_summary` — conflict state
- `merge_base` / `is_ancestor` — graph queries
- `ahead_behind` / `branch_ahead_behind` — divergence counts
- `branch_tracking` / `local_branch_tracking` — upstream metadata
- `is_merged_into` — convenience predicate
- `query_commits` — bounded history with skip/filter
- `blame_at` / `tree_at_commit` / `tree_at_path` — point-in-time reads
- `references` / `references_by_kind` / `remotes` — ref/remote inventory
- `submodule_summaries` / `stash_detail` / `stash_diff` / `worktree_details` — rich detail
- `rich_worktree_status` — richer status model
- `diff_entries` — rename/copy-aware diff
- `snapshot` — batch read
- `create_tag` / `create_annotated_tag` / `delete_tag` — write operations

## Conformance contracts

### Sorting

`local_branches()` must be sorted ascending by full ref name (`refs/heads/…`).
`list_tags_sorted(ByName)` must be sorted ascending by short name.
`DiffSummary` fields (`added`, `modified`, `deleted`) must be sorted ascending.
`submodule_summaries()` and `worktree_details()` must be sorted by path/id.

### Owned values

All returned types must be `Send + 'static`. No lifetimes escape the method.

### Error model

Use `endringer::Error` variants: `NotFound { kind, name }` for missing
objects/refs, `NotARepository` for bad paths, `UnsupportedBackendFeature`
for unimplemented optional methods.

### Thread safety

`VcsBackend: Send + Sync`. Methods take `&self` and must be safe to call
concurrently.

## Testing your backend

The simplest approach is to pass your backend to `Repository::with_backend`
and call the same integration test patterns used in `endringer`'s own tests:

```rust,no_run
use endringer::{VcsBackend, BackendKind, repository::Repository};

fn smoke_test(backend: impl VcsBackend + 'static) {
    let repo = Repository::with_backend(Box::new(backend), BackendKind::Git);
    let _ = repo.list_commits().expect("list_commits should not panic");
    let _ = repo.local_branches().expect("local_branches should not panic");
    let _ = repo.list_tags().expect("list_tags should not panic");
}
```

A future `endringer-testkit` crate may provide a full conformance suite once
`VcsBackend` is declared stable.
