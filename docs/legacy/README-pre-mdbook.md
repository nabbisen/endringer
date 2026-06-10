> **Legacy document.** This file is not part of the current public documentation.
> It is a single-file developer guide from the pre-mdBook era (v0.8.x), superseded
> by the structured documentation under `docs/src/`. It is retained here for
> historical reference only.

# endringer — Developer Documentation

This document describes the design decisions, architecture, and module responsibilities of `endringer`. For end-user usage instructions, please refer to [README.md](../README.md).

---

## Design Philosophy

Following the UNIX philosophy, `endringer` is a library designed to **do one thing and do it well**:

- **Read VCS state** (branches, commits, tags, and HEAD snapshots).
- Write operations are strictly limited to creating and deleting lightweight tags (commits, merges, and pushes are out of scope).
- Higher-level concerns such as configuration persistence, scheduling, UI, and i18n are the responsibility of the calling application.

### Boundary Principles

> The VCS implementation (including `gix`) is fully encapsulated within `endringer`. 
> Downstream crates only need to interface with `Repository` and `types::*`.

To uphold this principle, the following changes were implemented in v0.8.0:

| Change | Reasoning |
|---|---|
| Introduced `CommitId` newtype | `gix::ObjectId` was leaking into the public API. |
| Changed `repository::branch` / `repository::commit` visibility to `pub(crate)` | Internal functions were being accessed directly. |
| Added `create_tag` / `delete_tag` / `list_tags` / `log_since` to `Repository` | Operations essential to the `VcsAdapter` layer were not previously exposed. |

---

## Module Structure

```
src/
  lib.rs                   Public entry point, crate-level documentation
  types.rs                 Public type definitions (CommitId, BranchInfo, CommitInfo, StatusDigest, TagInfo)
  util.rs                  Internal utilities (pub(crate))
  repository.rs            Repository struct, public methods, and constructor
  repository/
    branch.rs              local_branches / remote_branches / list_commits / log_since (pub(crate))
    branch/util.rs         Helper for scanning ref prefixes (pub(super))
    commit.rs              status_digest (pub(crate))
    tag.rs                 list_tags / create_tag / delete_tag (pub(crate))
```

### Visibility Rules

| Scope | Purpose |
|---|---|
| `pub` | `types::*`, `repository::Repository`, `repository::repository()`, `commit_id_to_short_id()` |
| `pub(crate)` | `repository::{branch, commit, tag}` — Internal VCS implementation modules. |
| `pub(super)` | `repository/branch/util` — Shared helpers within submodules. |
| private | `util`, raw `gix` operations. |

---

## Type Design

### `CommitId`

```rust
pub struct CommitId(pub(crate) gix::ObjectId);
```

- Wraps `gix::ObjectId`, but the inner field is `pub(crate)` to remain opaque to external users.
- Implements `Display`, producing a 40-character hex string.
- `CommitId::short()` returns the first 7 characters (conventional short hash).
- Eliminates the need for downstream crates to depend on `gix`.

### `TagInfo`

Both lightweight and annotated tags are "peeled" to their respective commits before being returned. Note that tag objects themselves (including messages and signatures) are not exposed in the current version.

---

## Implementation Notes

### Tag Creation (`create_tag`)

Creates a lightweight tag pointing to the current `HEAD`.
The implementation uses `gix::refs::transaction::PreviousValue::MustNotExist`, ensuring that an error is returned if a tag with the same name already exists.

### `log_since(since, until)`

Since Git history is a Directed Acyclic Graph (DAG) and commit timestamps can be set arbitrarily by authors, this method traverses all ancestors to filter commits where `since <= timestamp <= until`.
In large repositories, this involves an $O(n)$ traversal cost.



### Doctest Limitations

Due to a known toolchain constraint in Edition 2024 + rustdoc 1.91, `cargo test` doctests currently fail because `--check-cfg` requires `-Z unstable-options`. 
All library unit tests can be verified via `cargo test --lib`.

---

## Versioning Policy

This library adheres to [Semantic Versioning](https://semver.org/).

v0.8.0 introduces breaking changes due to the removal of `gix::ObjectId`. To migrate from v0.7.x, replace occurrences of `gix::ObjectId` with `endringer::types::CommitId`.

---

## Dependencies

| Crate | Purpose |
|---|---|
| `gix` | Reading and writing to Git repositories (internal only). |
| `anyhow` | Unified error handling. |

Because `gix` is not exposed in the public API, it does not appear directly in the dependency tree of downstream crates (though it remains in `Cargo.lock` as a transitive dependency).
