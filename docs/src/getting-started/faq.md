# FAQ

## Does endringer require `git` or `jj` to be installed?

No. Both backends read the repository's object store directly via `gix`.
No external binary is spawned.

## Can I use endringer with a Jujutsu repository?

Yes. Use `jj_repository(path)` instead of `repository(path)`. endringer
detects the `.jj/` directory and opens the underlying git store. Both
co-located (`.git/` + `.jj/`) and native jj layouts are supported.

## Why do I see my own crate depending on `gix` transitively?

`gix` is a transitive dependency of `endringer-git`. It does not appear in
your crate's public API, but it is compiled as part of the dependency tree.
This is expected. If compile time is a concern, see the workspace architecture
section on crate-level dependency control.

## What does `status_digest()` return for a detached HEAD?

`StatusDigest::current_branch` is set to `"(detached)"`.

## How does `is_dirty()` work?

It compares the index stat cache (mtime and file size) against the working
tree, with a SHA-1 content-hash fallback for same-size modifications. Staged
changes are detected by comparing index blob OIDs against the HEAD tree.
gitignore rules are applied to untracked files.

## Does blame work on renamed files?

Yes. When a file was renamed between commits, `BlameEntry::original_path`
is set to the former path in the source commit.

## Why is `commit_id_to_short_id()` missing?

It was removed in v0.18.1. Use `commit_id.short()` directly.

## Can I implement my own backend?

Yes. `endringer` re-exports `VcsBackend` (from `endringer-core`). Implement
all methods on your type, then construct a `Repository` with
`Repository::with_backend(Box::new(your_backend), BackendKind::Git)`.
Note: `VcsBackend` is not yet stable (may change before v1.0).
