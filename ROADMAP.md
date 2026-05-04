# Roadmap

This document describes endringer's development direction and version plan.
Version numbers follow [Semantic Versioning](https://semver.org/).

---

## Release policy

### Versioning rules

| Change type | Version bump |
|---|---|
| Breaking public API change (type removal, signature change) | major (minor while pre-v1.0) |
| Backward-compatible feature addition | minor |
| Bug fix, doc fix, internal refactor | patch |

Before v1.0 (current state), minor version bumps may include breaking changes.
From v1.0 onward, breaking changes in any version require a migration guide
in the CHANGELOG.
These are marked `Breaking change:` in the CHANGELOG.

### Tarball naming

```
dist/endringer-{version}.tar.gz   # e.g. dist/endringer-0.9.0.tar.gz
```

The filename has no `v` prefix. Git tags continue to use the `v{version}` form
(e.g. `v0.9.0`).

### Release procedure

```sh
# 1. Bump version in each crate's Cargo.toml (or workspace Cargo.toml)
# 2. Add a [x.y.z] section to CHANGELOG.md
# 3. Update the release history table in ROADMAP.md
# 4. Verify tests pass and warnings are clean
cargo test --workspace

# 5. Run the release script
./scripts/release.sh

# 6. Push to remote
git push origin master
git push origin v{version}

# 7. Publish to crates.io (optional)
cargo publish -p endringer-core
cargo publish -p endringer-git
cargo publish -p endringer-jj
cargo publish -p endringer
cargo publish -p endringer-async
```

---

## Release history

| Version | Date | Summary |
|---|---|---|
| [v0.7.1] | 2025 | Initial public release. Branch listing, commit history, status digest. |
| [v0.8.0] | 2026-05-04 | `CommitId` newtype, tag operations, `log_since`, public API cleanup. |
| [v0.8.1] | 2026-05-04 | Bug fixes (repo_name, current_branch, timestamp safety, author consistency), tarball naming. |
| [v0.9.0] | 2026-05-04 | `CommitId::from_hex`, `SortOrder`, `list_commits_sorted`, `list_tags_sorted`, annotated tags. |
| [v0.10.0] | 2026-05-04 | `CommitInfo` committer fields, `find_commit`, `diff`, `remote_url`. |
| [v0.11.0] | 2026-05-04 | `async` feature flag (`AsyncRepository`), Jujutsu backend (`JjBackend`), `VcsBackend` trait. |
| [v0.12.0] | 2026-05-04 | `JjBackend` rewritten to use gix directly (no `jj` binary), test module separation. |
| [v0.13.0] | 2026-05-04 | Cargo workspace (5 crates), `CommitId: Ord`, `DiffSummary` ordering, fixture tests, async tests. |
| [v0.14.0] | 2026-05-04 | `GitBackend` lock-free, `CommitInfo.parents`, `is_dirty()`, jj annotated tag `Err`. |
| [v0.15.0] | 2026-05-04 | Test split (`support/fixture.rs`), `merge_base`, `is_ancestor`, `blame`, `BlameEntry`. |
| [v0.16.0] | 2026-05-04 | `WorktreeStatus`, `file_at_commit`, recursive tree traversal. |
| [v0.15.0] | 2026-05-04 | `GitBackend` lock-free via `ThreadSafeRepository`, `CommitInfo.parents`, `is_dirty()`, jj annotated tag error. |

---

## v0.14.0 ✅ Released (2026-05-04)

### Lock-free `GitBackend` ✅

Replaced `Mutex<gix::Repository>` with `gix::ThreadSafeRepository`. Each
method call gets a cheap thread-local view via `.to_thread_local()`, eliminating
serialization under concurrent async load.

### `CommitInfo.parents: Vec<CommitId>` ✅

Every `CommitInfo` now carries the commit's direct parent IDs. Enables merge
detection and graph construction by callers.  
**Breaking change**: code that constructs `CommitInfo` directly must add `parents`.

### `Repository::is_dirty()` ✅

Returns `true` if the working tree has any uncommitted changes (staged or unstaged).
Bare repositories always return `false`.

### `JjBackend::create_annotated_tag` returns `Err` ✅

Previously fell back silently to a lightweight tag. Now returns an explicit error
so callers can decide how to handle the limitation.

---

## v0.13.0 ✅ Released (2026-05-04)

### Cargo workspace ✅

Five-crate workspace: `endringer-core`, `endringer-git`, `endringer-jj`,
`endringer`, `endringer-async`. The `async` feature flag was removed from
`endringer`; use the `endringer-async` crate instead.

### `CommitId: Ord` ✅

Byte-level lexicographic ordering. `BTreeSet<CommitId>` and `.sort()` work.

### `DiffSummary` path ordering ✅

`added`, `modified`, `deleted` paths are sorted ascending within each category.

### Fixture-based integration tests ✅

`tests/integration.rs` creates isolated repositories in `tempfile::TempDir`.

### `#[tokio::test]` async tests ✅

Seven async integration tests in `endringer-async/tests/async_tests.rs`.

---

## Planned

### Status heuristic: content-hash fallback

The current `worktree_status` / `is_dirty` implementation uses mtime + file
size as a heuristic. Files modified without changing size within the same
clock second will not be detected. A future release will add a SHA-1 content
comparison for entries where mtime and size match (matching git's own
`update-index` behaviour).

### Gitignore support for untracked files

`WorktreeStatus.untracked` currently lists all unindexed files regardless of
`.gitignore`. A future release will apply gitignore rules via gix's dirwalk API.

### Submodule listing

`Repository::submodules()` — enumerate submodule paths and remote URLs without
running the `git submodule` binary.

### Stash entries

`Repository::stash_entries()` — list stash entries with their commit IDs and
descriptions.

---

## v1.0.0 (stable API)

The v1.0.0 release signals a mature, well-tested public API. Breaking changes
remain possible in any version, but from v1.0 onward every breaking change
**must** be accompanied by a migration guide in the CHANGELOG that explains
what changed and how to update calling code.

Readiness criteria:
- Core APIs (`find_commit`, `diff`, annotated tags, `parents`, `is_dirty`, `worktree_status`, `blame`) are all stable.
- At least two downstream crates have used the library across two minor versions.
- The status heuristic fallback and gitignore support are either implemented or explicitly deferred.
- No remaining "wish we'd done this differently" items in the public API.

---

## Out of scope by design

| Item | Reason |
|---|---|
| Commit, merge, push | endringer is read-oriented; only tag writes are in scope. |
| Config file persistence | Application-layer concern. |
| UI / i18n | Library responsibility ends at data. |
| Scheduled polling | Caller's responsibility (e.g. `iced::Subscription`). |
| Authentication / credential management | Delegated to OS credential store via gix. |

[v0.7.1]: https://github.com/nabbisen/endringer/releases/tag/v0.7.1
[v0.8.0]: https://github.com/nabbisen/endringer/releases/tag/v0.8.0
[v0.8.1]: https://github.com/nabbisen/endringer/releases/tag/v0.8.1
[v0.9.0]: https://github.com/nabbisen/endringer/releases/tag/v0.9.0
[v0.10.0]: https://github.com/nabbisen/endringer/releases/tag/v0.10.0
[v0.11.0]: https://github.com/nabbisen/endringer/releases/tag/v0.11.0
[v0.12.0]: https://github.com/nabbisen/endringer/releases/tag/v0.12.0
[v0.13.0]: https://github.com/nabbisen/endringer/releases/tag/v0.13.0
[v0.14.0]: https://github.com/nabbisen/endringer/releases/tag/v0.14.0
[v0.15.0]: https://github.com/nabbisen/endringer/releases/tag/v0.15.0
[v0.16.0]: https://github.com/nabbisen/endringer/releases/tag/v0.16.0
