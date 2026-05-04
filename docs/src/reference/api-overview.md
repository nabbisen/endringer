# API overview

All methods are on `endringer::repository::Repository`. Each returns
`anyhow::Result<_>` unless stated otherwise.

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
| `backend_kind()` | `BackendKind` | `Git` or `Jj` |

## Branches

| Method | Returns |
|---|---|
| `local_branches()` | `Vec<BranchInfo>` |
| `remote_branches()` | `Vec<BranchInfo>` |

## Commits

| Method | Returns | Notes |
|---|---|---|
| `list_commits()` | `Vec<CommitInfo>` | Newest first |
| `list_commits_sorted(order)` | `Vec<CommitInfo>` | `SortOrder::NewestFirst / OldestFirst / ByName` |
| `log_since(since, until)` | `Vec<CommitInfo>` | Filter by author timestamp |
| `find_commit(id)` | `CommitInfo` | O(1) object-DB lookup |

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
| `CommitInfo::parents` | `Vec<CommitId>` | Direct parents (empty for initial commit) |

## Diff & content

| Method | Returns | Notes |
|---|---|---|
| `diff(from, to)` | `DiffSummary` | Paths sorted ascending within each category |
| `blame(path)` | `Vec<BlameEntry>` | Path relative to repo root; entries in line order |
| `file_at_commit(path, commit_id)` | `Vec<u8>` | Raw bytes; path relative to repo root |

## Repository metadata

| Method | Returns |
|---|---|
| `remote_url(name)` | `Option<String>` |
| `submodules()` | `Vec<SubmoduleInfo>` |
| `stash_entries()` | `Vec<StashEntry>` |
| `worktrees()` | `Vec<WorktreeInfo>` |

## Async (`endringer-async`)

`AsyncRepository` mirrors every `Repository` method as `async fn`, delegating
to `tokio::task::spawn_blocking`. Constructors: `AsyncRepository::open(path)`
and `AsyncRepository::open_jj(path)`.
