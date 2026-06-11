# API overview

All methods are on `endringer::repository::Repository`. Each returns
`endringer::Result<T>` (a type alias for `Result<T, endringer::Error>`).
See the [error model](../development/migration-v0.23-errors.md) for the
full `Error` enum.

## Constructors

| Function | Returns |
|---|---|
| `repository(path)` | `Result<Repository>` — open a Git repository |
| `jj_repository(path)` | `Result<Repository>` — open a Jujutsu repository |
| `Repository::with_backend(backend, kind)` | `Repository` — inject a custom `VcsBackend` |

## Status

| Method | Returns | Notes |
|---|---|---|
| `status_digest()` | `StatusDigest` | Current branch, HEAD commit, timestamp |
| `is_dirty()` | `bool` | Any uncommitted changes (staged or unstaged) |
| `worktree_status()` | `WorktreeStatus` | Per-file staged / unstaged / untracked |
| `rich_worktree_status(options)` | `RichWorktreeStatus` | Richer status with conflict info and more change kinds |
| `operation_state()` | `OperationState` | In-progress operation: None/Merge/Rebase/CherryPick/Revert/Bisect |
| `unmerged_paths()` | `Vec<PathBuf>` | Paths with conflict-stage index entries |
| `conflict_summary()` | `ConflictSummary` | Per-stage object IDs for each conflicted path |
| `backend_kind()` | `BackendKind` | `Git` or `Jj` |
| `repository_info()` | `RepositoryInfo` | Format, head state, capabilities |

## Branches

| Method | Returns |
|---|---|
| `local_branches()` | `Vec<BranchInfo>` — sorted ascending by full ref name |
| `remote_branches()` | `Vec<BranchInfo>` |
| `branch_tracking(branch)` | `BranchTrackingInfo` — upstream + ahead/behind for one branch |
| `local_branch_tracking()` | `Vec<BranchTrackingInfo>` — tracking info for all local branches |

## Commits

| Method | Returns | Notes |
|---|---|---|
| `list_commits()` | `Vec<CommitInfo>` | Newest first |
| `list_commits_sorted(order)` | `Vec<CommitInfo>` | `SortOrder::NewestFirst / OldestFirst / ByName` |
| `log_since(since, until)` | `Vec<CommitInfo>` | Filter by author timestamp |
| `find_commit(id)` | `CommitInfo` | O(1) object-DB lookup |
| `query_commits(query)` | `CommitQueryResult` | Bounded page: max_count, skip, since/until, sort order |

## Tags

| Method | Returns |
|---|---|
| `list_tags()` | `Vec<TagInfo>` |
| `list_tags_sorted(order)` | `Vec<TagInfo>` |
| `create_tag(name)` | `()` |
| `create_annotated_tag(name, message)` | `()` |
| `delete_tag(name)` | `()` |

## Commit graph

| Method | Returns | Notes |
|---|---|---|
| `merge_base(a, b)` | `Option<CommitId>` | `None` for unrelated histories |
| `is_ancestor(candidate, descendant)` | `bool` | Inclusive (a commit is its own ancestor) |
| `is_merged_into(branch, target)` | `bool` | Convenience wrapper around `is_ancestor` |
| `ahead_behind(local, upstream)` | `AheadBehind` | Commits each ref has that the other doesn't |
| `branch_ahead_behind(branch)` | `Option<AheadBehind>` | Uses configured upstream; `None` if no upstream |
| `CommitInfo::parents` | `Vec<CommitId>` | Direct parents (empty for initial commit) |

## Diff, blame, and content

| Method | Returns | Notes |
|---|---|---|
| `diff(from, to)` | `DiffSummary` | Paths sorted ascending within each category |
| `diff_entries(from, to, options)` | `Vec<DiffEntry>` | Rename/copy-aware diff; `DiffOptions::default()` disables detection |
| `blame(path)` | `Vec<BlameEntry>` | Path relative to repo root; entries in line order; HEAD |
| `blame_at(path, commit_id)` | `Vec<BlameEntry>` | Blame at an arbitrary historical commit |
| `file_at_commit(path, commit_id)` | `Vec<u8>` | Raw bytes at a specific commit |
| `tree_at_commit(commit_id)` | `Vec<TreeEntry>` | Non-recursive root tree listing; sorted by name |
| `tree_at_path(commit_id, path)` | `Vec<TreeEntry>` | Non-recursive subtree listing |

## References and remotes

| Method | Returns | Notes |
|---|---|---|
| `references()` | `Vec<RefInfo>` | All refs including HEAD; sorted ascending |
| `references_by_kind(kind)` | `Vec<RefInfo>` | Filter by `RefKind` |
| `remotes()` | `Vec<RemoteInfo>` | All configured remotes; sorted by name |
| `remote_url(name)` | `Result<Option<String>>` | Fetch URL of named remote; `Ok(None)` if absent |

## Repository metadata

| Method | Returns |
|---|---|
| `submodules()` | `Vec<SubmoduleInfo>` |
| `submodule_summaries()` | `Vec<SubmoduleSummary>` — richer: initialization state, sync status |
| `stash_entries()` | `Vec<StashEntry>` |
| `stash_detail(index)` | `StashDetail` — author, timestamp, parent commits |
| `stash_diff(index)` | `DiffSummary` — stash vs its first parent |
| `worktrees()` | `Vec<WorktreeInfo>` |
| `worktree_details()` | `Vec<WorktreeDetail>` — richer: state, lock reason, head commit |

## Snapshot

| Method | Returns | Notes |
|---|---|---|
| `snapshot(request)` | `RepositorySnapshot` | Batch read; reduces inter-call drift for status widgets |

## Async (`endringer-async`)

`AsyncRepository` mirrors every `Repository` method as `async fn`, delegating
to `tokio::task::spawn_blocking`. Constructors: `AsyncRepository::open(path)`
and `AsyncRepository::open_jj(path)`.

See [async semantics](../development/async-semantics.md) for cancellation
behaviour and the semaphore pattern.
