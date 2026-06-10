# Async API operational semantics

`endringer-async` provides an `AsyncRepository` that mirrors every sync
method. This page documents what "async" means in this context and what
consumers should expect.

## What AsyncRepository actually does

Every method calls `tokio::task::spawn_blocking` to run the underlying
repository read on the blocking thread pool. Repository reads are
filesystem and object-store operations — they are inherently blocking and
there is no non-blocking kernel API underneath them.

**`AsyncRepository` is not a non-blocking filesystem API.** It is a
convenience wrapper that avoids blocking async executor threads. The
underlying work still runs to completion on a blocking thread.

## Cancellation semantics

If an async future is dropped (e.g. via `tokio::time::timeout` or task
cancellation) **before** the blocking task starts, tokio may avoid
scheduling it and the work will not run.

If the blocking task has **already started**, it will run to completion.
The result is discarded when the awaiting future is dropped, but the
read operation on disk continues until it finishes. This is standard
`spawn_blocking` behaviour and is not specific to endringer.

**Practical implication:** do not assume that dropping an `AsyncRepository`
future immediately frees disk resources. Use a consumer-owned semaphore to
bound how many blocking reads run concurrently.

## Recommended concurrency pattern

```rust,no_run
use endringer_async::AsyncRepository;
use std::sync::Arc;
use tokio::sync::Semaphore;

async fn scan_repos(repos: Vec<AsyncRepository>) {
    let sem = Arc::new(Semaphore::new(8)); // max 8 concurrent reads

    let tasks: Vec<_> = repos.into_iter().map(|repo| {
        let sem = Arc::clone(&sem);
        tokio::spawn(async move {
            let _permit = sem.acquire_owned().await.unwrap();
            repo.status_digest().await
        })
    }).collect();

    for t in tasks {
        let _ = t.await;
    }
}
```

The semaphore is consumer-owned. endringer imposes no internal global
concurrency limits.

## Error mapping

- Sync backend errors become `endringer::Error` (typed, matchable).
- A blocking-task panic becomes `Error::TaskJoin { message }`.
- Normal errors are not conflated with panics.

## Sync/async API parity

Every sync `Repository` method has an async mirror in `AsyncRepository`.
New sync methods added in any release also add an async wrapper in the
same release.

| Area | Sync | Async |
|---|---|---|
| Status | `status_digest`, `is_dirty`, `worktree_status`, `operation_state`, `unmerged_paths`, `conflict_summary` | ✓ mirrored |
| Commits | `list_commits*`, `log_since`, `find_commit`, `query_commits` | ✓ mirrored |
| Branches | `local_branches`, `remote_branches`, `branch_tracking`, `local_branch_tracking` | ✓ mirrored |
| Tags | `list_tags*`, `create_tag`, `create_annotated_tag`, `delete_tag` | ✓ mirrored |
| Graph | `merge_base`, `is_ancestor`, `is_merged_into`, `ahead_behind`, `branch_ahead_behind` | ✓ mirrored |
| Diff & blame | `diff`, `blame`, `blame_at`, `file_at_commit` | ✓ mirrored |
| Tree | `tree_at_commit`, `tree_at_path` | ✓ mirrored |
| Refs | `references`, `references_by_kind`, `remotes` | ✓ mirrored |
| Repository | `repository_info`, `backend_kind`, `remote_url` | ✓ mirrored |
| Metadata | `submodules`, `stash_entries`, `worktrees` | ✓ mirrored |
| Detail | `submodule_summaries`, `stash_detail`, `stash_diff`, `worktree_details` | ✓ mirrored |

## Sync-only usage (no tokio required)

Consumers that do not use async add only `endringer` to `Cargo.toml`.
They pay zero compile-time cost for `tokio` or `endringer-async`.

```toml
[dependencies]
endringer = "0.31"           # sync only — no tokio
```

```toml
[dependencies]
endringer       = "0.31"     # sync methods
endringer-async = "0.31"     # async wrappers (brings tokio)
```
