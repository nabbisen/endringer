# endringer

`endringer` is a Rust library for reading local Git and Jujutsu repositories.
It wraps [`gix`](https://github.com/Byron/gitoxide) and exposes a clean,
owned-value API — no `gix` types leak into your code, no external binaries
are needed.

## What it does

| Category | Capabilities |
|---|---|
| **Status** | Current branch, HEAD commit, dirty check, per-file worktree status |
| **History** | Full commit log, time-range filter, commit lookup by ID |
| **Branches** | Local and remote-tracking branches |
| **Tags** | List, create (lightweight + annotated), delete |
| **Graph** | Merge base, ancestry check, parent commit IDs |
| **Diff** | File-level summary (added / modified / deleted) between two commits |
| **Blame** | Per-line commit attribution |
| **File content** | Read any file at any commit |
| **Metadata** | Submodules, stash entries, linked worktrees |
| **Jujutsu** | All of the above via `.jj/` repositories (git backend) |
| **Async** | Optional `endringer-async` crate wraps every method in `spawn_blocking` |

## What it does not do

- Write commits, merge branches, or push
- Manage application config or scheduling
- Provide UI or i18n

These are the caller's concerns.

## Next steps

- New here? Start with the [Quick start](getting-started/quickstart.md).
- Looking for a specific method? See the [API overview](reference/api-overview.md).
- Want to understand the internals? Read the [Architecture](development/architecture.md).
