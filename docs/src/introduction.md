# endringer

`endringer` is a Rust library for reading local Git and Jujutsu repositories.
It wraps [`gix`](https://github.com/Byron/gitoxide) and exposes a clean,
owned-value API — no `gix` types leak into your code, no external binaries
are needed.

## What it does

| Category | Capabilities |
|---|---|
| **Status** | Current branch, HEAD commit, dirty check, per-file worktree status, rich status with conflict info |
| **Operation state** | In-progress Merge / Rebase / CherryPick / Revert / Bisect detection |
| **History** | Full commit log, bounded pages, time-range filter, commit lookup by ID |
| **Branches** | Local and remote-tracking branches, upstream tracking, ahead/behind counts |
| **Tags** | List, create (lightweight + annotated), delete |
| **Graph** | Merge base, ancestry check, parent commit IDs, is-merged-into predicate |
| **Diff** | File-level summary (added / modified / deleted); rename/copy-aware diff |
| **Blame** | Per-line commit attribution at HEAD or any historical commit |
| **File content** | Read any file at any commit |
| **Tree snapshots** | Non-recursive directory listing at any commit |
| **References** | All refs (branches, tags, HEAD) with kind and target |
| **Remotes** | Configured remote names and fetch/push URLs |
| **Metadata** | Submodules, stash entries, linked worktrees (with rich detail variants) |
| **Snapshot** | Batch read for status widget data (reduces inter-call drift) |
| **Jujutsu** | All reads via `.jj/` repositories (git backend, no `jj` binary) |
| **Async** | Optional `endringer-async` crate wraps every method in `spawn_blocking` |

## What it does not do

- Write commits, merge branches, or push
- Fetch, clone, or pull from remotes
- Manage application config, scheduling, or UI

These are the caller's concerns. See the [boundary cookbook page](cookbook/write-then-read-boundary.md)
for the recommended write-then-read pattern.

## Next steps

- New here? Start with the [Quick start](getting-started/quickstart.md).
- Looking for a specific method? See the [API overview](reference/api-overview.md).
- Want to understand the internals? Read the [Architecture](development/architecture.md).
