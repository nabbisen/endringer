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
| [v0.17.0] | 2026-05-04 | Content-hash dirty fallback, gitignore-aware untracked, `submodules`, `stash_entries`. |
| [v0.18.0] | 2026-05-04 | Linked worktrees, `TagAnnotation`, `commit_id_to_short_id` deprecation. |
| [v0.18.1] | 2026-05-04 | Remove deprecated `commit_id_to_short_id`, crate READMEs, feature flag analysis. |
| [v0.19.0] | 2026-05-04 | Bug fixes, README restructure, full docs/ site, codebase audit in ROADMAP. |
| [v0.19.2] | 2026-05-04 | `gix` bump to 0.83, minor bug fixes. |
| [v0.20.0] | 2026-06-10 | Handoff/archive integrity (RFC 001), public contract consistency (RFC 002). |

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

### `CommitId` inline storage optimisation

The current `CommitId(Vec<u8>)` heap-allocates on every clone.  A future
release may switch to an enum over fixed-size byte arrays
(`Sha1([u8; 20])` / `Sha256([u8; 32])`) for zero-allocation identity
comparisons and more compact `HashMap<CommitId, _>` usage.

### `commit_id_to_short_id` removal

Deprecated in v0.18.0; scheduled for removal at v1.0.0.

### API stability review

Before v1.0.0: audit every public type and method for naming consistency and
completeness. Anything marked "wish we'd done this differently" should be
changed now while pre-v1.0 minor bumps permit it.

---

## Codebase audit (v0.19.0 findings)

This is a snapshot audit from v0.19.0. Items flagged here are candidates for
a dedicated v1.0-prep refactor phase.

### API naming inconsistencies

| Item | Issue | Proposed fix |
|---|---|---|
| `CommitInfo::timestamp` | Author timestamp; the paired field is `committer_timestamp`. Asymmetric: one is named by *what* it is, the other by *whose* it is. | Rename to `author_timestamp` (breaking). |
| `BranchInfo::last_commit_*` fields | "last" is ambiguous (last in time? last in order?). Means "tip commit". | Rename to `tip_commit_*` (breaking). |
| `StatusDigest::last_commit_*` fields | Same as above. | Rename to `head_commit_*` (breaking). |
| `VcsBackend` method count (22) | Growing; no default impls. Custom backends must implement all 22 methods. | Add `default fn` impls for less-common methods (`worktrees`, `stash_entries`, `submodules`) that return `Ok(vec![])`. Non-breaking. |

### File-size observations

| File | Lines | Notes |
|---|---|---|
| `endringer-core/src/types.rs` | ~320 | Accumulating types; consider splitting into `types/commit.rs`, `types/status.rs`, etc. — but wait until v1.0 stabilises the type set. |
| `endringer/src/repository.rs` | ~250 | Delegation boilerplate; no structural concern. |
| `endringer-async/src/async_api.rs` | ~200 | Mirrors `repository.rs`; both grow together. Acceptable. |
| `crates/endringer-git/src/status.rs` | ~165 | Contains `is_dirty`, `worktree_status`, and helpers. Could split off `worktree_status` into its own module if it grows further. |

### docs/README.md (legacy)

The file `docs/README.md` is a single-file Japanese developer document from
the early single-crate era. It was superseded by `docs/src/` in v0.19.0.
The file should be removed in the next housekeeping pass.

### Deferred items (no action required now)

- `CommitId` inline storage (enum over `[u8; 20]` / `[u8; 32]`) — noted in
  Planned section; defer until post-v1.0.
- Feature flag scheme — analysed above; defer until post-v1.0.
- `commit_id_to_short_id` — already removed in v0.18.1.

---

## Feature flag architecture (design consideration)

This section records the analysis of whether to introduce Cargo feature flags
to give users finer-grained control over what gets compiled.

### Current state

The workspace already separates concerns into five crates:

| Crate | What it adds |
|---|---|
| `endringer-core` | Types + `VcsBackend` trait only — minimal |
| `endringer-git` | Full git backend via gix (~120 transitive deps) |
| `endringer-jj` | jj support (thin delegation to endringer-git) |
| `endringer` | Facade with all APIs |
| `endringer-async` | Optional async wrapper — separate crate already |

A user who only needs commit history can depend directly on `endringer-git`
instead of `endringer`, keeping the façade's API surface out of scope.

### Proposed groupings for optional feature flags in `endringer`

If we were to add feature flags to the main facade, the natural groups are:

| Feature | Methods covered | Notes |
|---|---|---|
| `core` (always on) | `repository()`, `status_digest()`, `backend_kind()` | Cannot be disabled |
| `branches` | `local_branches()`, `remote_branches()` | Very low cost |
| `commits` | `list_commits*()`, `log_since()`, `find_commit()` | Needed by most apps |
| `tags` | `list_tags*()`, `create_tag()`, `delete_tag()`, `create_annotated_tag()` | Often optional |
| `diff` | `diff()` | Low incremental cost |
| `graph` | `merge_base()`, `is_ancestor()` | Moderate |
| `blame` | `blame()` | Heavy — gix-blame required |
| `worktree` | `is_dirty()`, `worktree_status()`, `file_at_commit()` | Moderate |
| `metadata` | `submodules()`, `stash_entries()`, `worktrees()` | Moderate |

A `default` feature would bundle `branches + commits + tags + diff + worktree`,
matching the needs of a typical VCS UI widget.

### Trade-offs

**For feature flags:**
- Users who only need commit history avoid compiling blame and worktree scanning.
- `gix-blame` is a meaningful compile-time cost that can be saved.

**Against feature flags:**
- `VcsBackend` is a public trait: feature-gating methods would break custom
  backend implementations that are compiled with different feature sets.
- The complexity of `#[cfg(feature = ...)]` throughout the codebase
  significantly increases maintenance burden.
- gix itself is the dominant compile-time cost; our wrapper code is negligible.
  Saving the wrapper still compiles all of gix.

### Decision

**Defer** feature-gating inside the existing crates for now.  The preferred
path is:
1. Keep the workspace as the primary dependency-control mechanism.
2. Add `gix-blame` behind an optional `blame` feature flag in `endringer-git`
   if benchmark evidence shows it causes meaningful overhead for non-blame users.
3. Revisit the full feature-flag scheme post-v1.0 when the API is stable — at
   that point, adding `#[cfg]` gating would itself be a non-breaking change
   (turning always-on APIs into opt-in requires a major bump anyway).

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
[v0.17.0]: https://github.com/nabbisen/endringer/releases/tag/v0.17.0
[v0.18.0]: https://github.com/nabbisen/endringer/releases/tag/v0.18.0
[v0.19.0]: https://github.com/nabbisen/endringer/releases/tag/v0.19.0
[v0.19.2]: https://github.com/nabbisen/endringer/releases/tag/v0.19.2
[v0.20.0]: https://github.com/nabbisen/endringer/releases/tag/v0.20.0
