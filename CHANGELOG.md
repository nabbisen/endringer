# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.20.0] — 2026-06-10

This release implements **RFC 001** (handoff, archive, and release-manifest
integrity) and **RFC 002** (public contract consistency and documentation
tests). There are no public API changes. All 88 tests pass.

### Fixed

- **Stale rustdoc: gitignore filtering.** `WorktreeStatus`, the `worktree_status`
  docs in both `endringer-core` and the `endringer` façade, and
  `WorktreeStatus` previously said gitignore rules are *not* applied to
  untracked files. This was correct for v0.16.0 but stale since v0.17.0, which
  added the content-hash fallback and gitignore-aware untracked. The doc now
  accurately states that gitignore rules are applied, with a note on the
  graceful-degradation fallback path. *(RFC 002)*

- **Stale rustdoc: jj annotated-tag fallback.** The `Repository::create_annotated_tag`
  doc claimed the jj backend falls back to a lightweight tag. The actual
  implementation (since v0.14.0) returns an explicit error. The jj backend
  module doc carried the same incorrect claim. Both are corrected. *(RFC 002)*

- **Duplicate release-history row in `ROADMAP.md`.** The table listed
  `v0.15.0` twice; the second row was a mis-labelled duplicate of `v0.14.0`
  content. Removed the duplicate. *(RFC 001)*

- **Stale spacing artifact in jj error message.** The `create_annotated_tag`
  error message had extra internal whitespace (`"does not support annotated tags;
  &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;use create_tag…"`). Fixed.

### Added

- **`rfcs/` directory.** The source tree and release archive now include the
  full RFC directory: `rfcs/000-rfc-lifecycle-policy.md`, `rfcs/README.md`
  (index), `rfcs/proposed/001`–`031`, and the `done/` and `archive/` folders.
  *(RFC 001)*

- **`RELEASE-MANIFEST.md`.** A machine-readable-friendly record of required
  files, required directories, excluded paths, publish order, and the
  verification command for release archives. *(RFC 001)*

- **`scripts/check-public-contract.sh`.** A CI-runnable shell script that
  searches for known stale phrases in source and docs and verifies required
  project files are present. Runs in CI after this release. *(RFC 002)*

- **`scripts/verify-release-manifest.sh`.** Verifies an unpacked release
  archive against `RELEASE-MANIFEST.md`. Run this on the archive before
  tagging. *(RFC 001)*

- **`docs/src/reference/contract.md`.** A new mdBook page cataloguing
  high-value behavioral contracts: gitignore filtering, jj tag semantics, diff
  ordering, tag peel semantics, no public `gix` types, stash ordering,
  concurrency model, and the no-external-binaries-at-runtime guarantee.
  *(RFC 002)*

- **`docs/src/SUMMARY.md`** now includes `contract.md` under the Reference
  section. *(RFC 002)*

### Changed

- **`docs/README.md` (legacy)** moved to `docs/legacy/README-pre-mdbook.md`
  and marked with a prominent legacy banner. This file was a single-file
  Japanese developer document from the pre-mdBook era, superseded by
  `docs/src/` in v0.19.0. *(RFC 001)*

---

## [0.19.2] — 2026-05-04

### Changed

- **`gix` dependency updated `0.77` → `0.83`** (latest as of 2026-04-27).

  No API changes were required in endringer — all 88 tests pass unchanged.

  Sub-crate version deltas across the 6 minor releases:

  | Sub-crate | 0.77 | 0.83 | Δ |
  |---|---|---|---|
  | gix-hash | 0.21 | 0.25 | +4 |
  | gix-actor | 0.37 | 0.41 | +4 |
  | gix-object | 0.54 | 0.60 | +6 |
  | gix-ref | 0.57 | 0.63 | +6 |
  | gix-blame | 0.7 | 0.13 | +6 |
  | gix-ignore | 0.18 | 0.21 | +3 |
  | gix-index | 0.45 | 0.51 | +6 |
  | gix-submodule | 0.24 | 0.30 | +6 |
  | gix-worktree | 0.46 | 0.52 | +6 |

  The other direct dependencies (`anyhow 1.0.102`, `tokio 1.52.1`,
  `tempfile 3.27.0`) are already at their latest published versions.

---

## [0.19.1] — 2026-05-04

### Fixed

- **`cargo test` hangs when a git editor (e.g. neovim) is configured.**

  `Fixture::git()` — the helper used by all integration tests — was running
  git subprocesses with fully inherited environment and stdin. If the host
  has `GIT_EDITOR`, `VISUAL`, `EDITOR`, or `core.editor` set to an
  interactive editor (neovim, vim, …), any git command that consults the
  editor would open it in the test process's terminal and block indefinitely.
  In tag tests, this manifested as neovim opening and the test suite
  hanging; closing neovim without a message body produced
  `fatal: no tag message?`.

  **Root cause**: missing environment isolation in the fixture git helper.

  **Fix applied to both fixture files**
  (`crates/endringer/tests/support/fixture.rs` and
  `crates/endringer-async/tests/async_tests.rs`):

  ```rust
  .env("GIT_CONFIG_NOSYSTEM", "1")        // skip /etc/gitconfig
  .env("GIT_CONFIG_GLOBAL", "/dev/null")  // skip ~/.gitconfig (hooks, editor, GPG…)
  .env("GIT_EDITOR", "true")              // no-op editor — never opens a terminal UI
  .env("GIT_TERMINAL_PROMPT", "0")        // suppress credential / terminal prompts
  .stdin(std::process::Stdio::null())     // disconnect from the test process's stdin
  ```

  With these overrides the fixture subprocesses are fully isolated from the
  developer's git configuration. All 88 tests now pass in any environment,
  regardless of the configured git editor.

---

## [0.19.0] — 2026-05-04

### Fixed

- **`it_works_remote_url` test was environment-dependent.** The assertion
  `remote_url("origin").is_none()` fails in any environment where the
  workspace git repo has an origin remote. Changed to use a name that cannot
  exist in any git repository (`__endringer_test_nonexistent__`).
- **Unused imports** `Duration` and `SystemTime` in
  `crates/endringer/src/repository/tests.rs` removed.
- **Unused import** `endringer_core::backend::VcsBackend` in
  `crates/endringer-jj/src/tests.rs` removed.

### Documentation

- **README.md** fully rewritten to follow the structured format:
  Hero → Overview → Why/When → Quick Start → Design Notes → Docs link.
  API reference table removed (moved to `docs/`). No licence prose in body.

- **`docs/` created** as an mdbook-ready documentation site organised by
  reader persona:

  | Persona | Location |
  |---|---|
  | First-time users | `docs/src/getting-started/` — introduction, quickstart, FAQ |
  | Experienced users | `docs/src/reference/` — full API overview, type reference, backend details |
  | Maintainers / contributors | `docs/src/development/` — philosophy, architecture, contributing guide |

### Added (ROADMAP)

- **Codebase audit findings** documented in ROADMAP: API naming inconsistencies
  (`timestamp` vs `committer_timestamp`, `last_commit_*`, `head_commit_*`),
  `VcsBackend` missing default impls, file-size observations, legacy
  `docs/README.md` removal candidate.
- **Feature flag architecture analysis** added to ROADMAP: proposed groupings,
  trade-offs, and the decision to defer until post-v1.0.

---

## [0.18.1] — 2026-05-04

### Changed — Breaking

- **`commit_id_to_short_id()` removed.** The function was deprecated in
  v0.18.0. Use `commit_id.short()` directly.

  **Migration**: replace every call site:
  ```rust
  // before
  let s = endringer::commit_id_to_short_id(id);
  // after
  let s = id.short();
  ```

### Added

- **`README.md` for each sub-crate** (`endringer-core`, `endringer-git`,
  `endringer-jj`, `endringer-async`), wired via the `readme` field in their
  `Cargo.toml`. These serve as the crates.io landing page for each crate.

- **Feature-flag architecture analysis in ROADMAP.** Documents the trade-offs
  of introducing Cargo feature flags to give users finer-grained compile-time
  control, and records the decision to defer until post-v1.0 — with the
  exception of potentially gating `gix-blame` behind an opt-in `blame`
  feature if benchmark evidence justifies it.

---

## [0.18.0] — 2026-05-04

### Changed — Breaking

- **`TagInfo` gained an `annotation: Option<TagAnnotation>` field.**
  Lightweight tags have `annotation: None`; annotated tags carry the message,
  tagger name, and tagger timestamp.

  **Migration**: add `annotation: None` to any `TagInfo { .. }` struct literal
  you construct directly.

- **`VcsBackend` gained a new required method**: `worktrees`.

  **Migration**: implement `fn worktrees(&self) -> Result<Vec<WorktreeInfo>>`.
  For backends without linked-worktree support, return `Ok(vec![])`.

### Deprecated

- **`commit_id_to_short_id(id)`** — use `id.short()` directly.
  The function remains but emits a deprecation warning. It will be removed at
  or after v1.0.0.

### Added

- **`Repository::worktrees() -> Result<Vec<WorktreeInfo>>`** — lists all
  linked worktrees (created with `git worktree add`). The main worktree is
  not included. Results are sorted by worktree id. Each `WorktreeInfo` carries
  the worktree path, current branch, and lock state.

- **`TagAnnotation`** type — `message`, `tagger_name`, `tagger_timestamp`.
  Populated for annotated tags created with `git tag -a`.

- **`WorktreeInfo`** type — `id`, `path`, `current_branch`, `is_locked`.
  Re-exported from `endringer`.

- **`AsyncRepository::worktrees()`** async variant via `spawn_blocking`.

- `tests/git_worktree.rs` — 7 new integration tests covering linked worktrees
  (empty list, single worktree, multiple sorted) and tag annotations
  (lightweight has `None`, annotated has message and tagger).

---

## [0.17.0] — 2026-05-04

### Changed — Breaking

- **`VcsBackend` gained two new required methods**: `submodules` and
  `stash_entries`. Custom backend implementations must add these.

  **Migration**: implement both methods. For read-only or non-git backends,
  both may return `Ok(vec![])`.

### Fixed

- **Content-hash fallback in dirty detection.** When a file's mtime and size
  both match the index stat cache, the implementation now computes the
  file's git blob SHA-1 and compares it to the index entry OID.  This
  correctly detects modifications made within the same clock second without
  changing the file size — the sole blind spot in the previous heuristic.
  The fallback uses `gix::hash::hasher` and matches git's own
  `update-index --refresh` behaviour.

- **Gitignore-aware untracked file detection.** `WorktreeStatus::untracked`
  and `is_dirty` no longer report files that match active ignore rules
  (`.gitignore`, `$GIT_DIR/info/exclude`, global excludes). The check uses
  gix's exclude stack (`Repository::excludes`, enabled via the `excludes`
  feature already in the default feature set). If the exclude stack cannot
  be initialised for any reason, the check degrades gracefully and reports
  all untracked files as before.

### Added

- **`Repository::submodules() -> Result<Vec<SubmoduleInfo>>`** — lists every
  submodule declared in `.gitmodules`, with `name`, `path`, and `url`.
  Returns an empty `Vec` when `.gitmodules` is absent. Results are sorted by
  path. Uses the `gix-submodule` parser (enabled via the new `attributes`
  gix feature).

- **`Repository::stash_entries() -> Result<Vec<StashEntry>>`** — lists all
  stash entries newest-first (`stash@{0}` at index 0). Reads the
  `logs/refs/stash` reflog without spawning the `git` binary. Returns an
  empty `Vec` when the stash is empty.

- **`SubmoduleInfo`** and **`StashEntry`** types in `endringer-core::types`
  (re-exported from `endringer`).

- **`AsyncRepository::submodules()`** and **`stash_entries()`** async
  variants via `spawn_blocking`.

- `tests/git_submodule_stash.rs` — 6 new integration tests for submodules
  and stash.

- Gitignore test in `tests/git_status.rs`.

### Changed (internal)

- `gix` dependency now enables `attributes` feature in addition to `blame`.
  This adds `gix-submodule`, `gix-attributes`, and related crates.

---

## [0.16.0] — 2026-05-04

### Changed — Breaking

- **`VcsBackend` gained two new required methods**: `worktree_status` and
  `file_at_commit`. Custom backend implementations must add these.

  **Migration**: add both methods. For backends that do not support working-tree
  inspection, `worktree_status` may return `Ok(WorktreeStatus::default())` and
  `file_at_commit` may return `Err(anyhow!("not supported"))`.

### Added

- **`Repository::worktree_status() -> Result<WorktreeStatus>`** — full per-file
  working-tree status equivalent to `git status`:
  - `staged`: files whose blob OID in the index differs from the HEAD tree
    (Added, Modified, Deleted).
  - `unstaged`: files whose on-disk mtime or size differs from the index stat
    cache (Modified, Deleted).
  - `untracked`: files present in the working tree but not in the index.
    **Note**: gitignore rules are not yet applied — all unindexed files are
    reported. A future release will honour `.gitignore`.
  All three lists are sorted ascending by path.

- **`Repository::file_at_commit(path, commit_id) -> Result<Vec<u8>>`** —
  returns the raw bytes of a file as it exists in any commit's tree. Supports
  nested paths (e.g. `src/util/helper.rs`). Returns an error if the path does
  not exist in that commit.

- **`WorktreeStatus`**, **`StatusEntry`**, **`ChangeKind`** types in
  `endringer-core::types` (re-exported from `endringer`).

- **`AsyncRepository::worktree_status()`** and **`file_at_commit()`** async
  variants via `spawn_blocking`.

- **`tests/git_status.rs`** — 12 new integration tests covering staged,
  unstaged, untracked detection, sorted output, and `file_at_commit` (including
  nested paths, missing files, and wrong-commit errors).

### Known limitations

- The unstaged / `is_dirty` heuristic uses mtime + file-size only. Files
  modified without changing size within the same clock second will not be
  detected. A SHA-1 content-hash fallback will be added in a future release.
- `WorktreeStatus.untracked` does not apply gitignore rules.

---

## [0.15.0] — 2026-05-04

### Changed — Breaking

- **`VcsBackend` gained three new required methods**: `merge_base`, `is_ancestor`,
  and `blame`. Custom backend implementations must add these methods.

  **Migration**: implement the three new methods on any type that implements
  `VcsBackend`. See the documentation for default-feasible stubs if the
  operation is unsupported by your backend.

### Changed

- **`tests/integration.rs` split into seven focused files** under
  `crates/endringer/tests/`. Each file is an independent test binary covering
  one concern. A shared `tests/support/fixture.rs` is included via `#[path]`,
  avoiding `mod.rs` while eliminating duplication.

  | File | Coverage |
  |---|---|
  | `git_core.rs` | constructors, status digest, remote URL |
  | `git_branches.rs` | local and remote branch listing |
  | `git_commits.rs` | commit listing, sorting, log_since, find, parents, commit-graph |
  | `git_tags.rs` | tag listing, create/delete (lightweight + annotated) |
  | `git_diff.rs` | diff between commits, sorted paths |
  | `git_dirty.rs` | is_dirty scenarios (clean, modified, deleted, staged) |
  | `git_blame.rs` | per-line blame attribution |
  | `jj.rs` | jj backend rejection paths |

### Added

- **`Repository::merge_base(a, b) -> Result<Option<CommitId>>`** — best
  common ancestor via `gix::Repository::merge_base` (same algorithm as
  `git merge-base`). Returns `None` when the commits have no shared history.
- **`Repository::is_ancestor(candidate, descendant) -> Result<bool>`** —
  returns `true` if `candidate` is a direct or transitive ancestor of
  `descendant`. A commit is its own ancestor. Implemented via `merge_base`.
- **`Repository::blame(path) -> Result<Vec<BlameEntry>>`** — per-line commit
  attribution for a file at HEAD. Delegates to `gix::Repository::blame_file`.
  Requires gix `blame` feature (now enabled in `endringer-git`).
- **`BlameEntry`** type in `endringer-core::types` (re-exported from
  `endringer`): `commit_id`, `start_line`, `end_line` (1-indexed inclusive),
  and `original_path` (set when the file was renamed).
- **`AsyncRepository::merge_base`**, **`is_ancestor`**, **`blame`** async
  variants via `spawn_blocking`.

---

## [0.14.0] — 2026-05-04

### Changed — Breaking

- **`CommitInfo.parents: Vec<CommitId>` added.** Every commit now carries its
  direct parent IDs. Code that constructs `CommitInfo` directly (outside this
  library) must add `parents: vec![]` (or the real parent IDs).

  **Migration**: add `parents: vec![]` to any `CommitInfo { .. }` struct
  literal that does not already include the field.

- **`JjBackend::create_annotated_tag` now returns `Err`** instead of silently
  falling back to a lightweight tag. Callers that need a tag should use
  `create_tag()`, or explicitly catch and handle the error.

  **Migration**: replace `repo.create_annotated_tag(name, msg)?` with
  `repo.create_tag(name)?` when targeting jj repositories, or add error
  handling for the unsupported-operation case.

### Changed

- **`GitBackend` is now lock-free.** The internal `Mutex<gix::Repository>`
  has been replaced with `gix::ThreadSafeRepository`. Each method call
  obtains a thread-local view via `to_thread_local()` — no serialization
  under concurrent async load.

### Added

- **`Repository::is_dirty() -> Result<bool>`** — returns `true` when the
  working tree has uncommitted changes. Detection uses two passes: the index
  stat cache (mtime + file size) for unstaged changes, and a blob-OID
  comparison against the HEAD tree for staged changes. Bare repositories
  always return `false`.
- **`AsyncRepository::is_dirty()`** — async variant via `spawn_blocking`.
- **`CommitInfo.parents: Vec<CommitId>`** — direct parent commit IDs, enabling
  merge detection and commit-graph traversal by callers.
- Integration tests for `CommitInfo.parents`, `is_dirty`, and the jj annotated
  tag error path.

### Fixed

- Dirty check uses both mtime (seconds) **and** file size to catch
  same-second modifications — matches git's own stat-cache strategy.

---


## [0.13.0] — 2026-05-04

### Changed — Breaking

- **Cargo workspace restructure.** The single `endringer` crate is now a
  five-crate workspace. External crate API (name, types, constructors) is
  unchanged for users who depend on `endringer` by name.
  | Crate | Role |
  |---|---|
  | `endringer-core` | `CommitId`, all public types, `VcsBackend` trait |
  | `endringer-git`  | `GitBackend` (gix-powered) |
  | `endringer-jj`   | `JjBackend` (delegates to `endringer-git`) |
  | `endringer`      | Facade — `Repository`, constructors, re-exports |
  | `endringer-async`| Async facade (replaces the `async` feature flag) |
- **`async` feature flag removed from `endringer`.**  
  Migrate: replace `endringer = { features = ["async"] }` with  
  `endringer-async = "0.13"` and update imports to `endringer_async::AsyncRepository`.

### Added

- **`CommitId: PartialOrd + Ord`** — byte-level lexicographic ordering.  
  `BTreeSet<CommitId>`, `BTreeMap<CommitId, _>`, and `.sort()` on ID collections now work.
- **`DiffSummary` path ordering guarantee** — `added`, `modified`, and `deleted`
  paths are now sorted ascending within each category (enforced by the backend).
- **`Repository::with_backend(backend, kind)`** — public constructor for
  injecting custom [`VcsBackend`] implementations.
- **`VcsBackend` re-exported** from `endringer` — downstream crates can implement
  custom backends without depending on `endringer-core` directly.
- **`AsyncRepository::open_jj(path)`** — async constructor for Jujutsu repos.
- **Fixture-based integration tests** (`crates/endringer/tests/integration.rs`) —
  environment-independent test suite using `tempfile` + `git` CLI.
- **`#[tokio::test]` async tests** (`crates/endringer-async/tests/async_tests.rs`) —
  7 async integration tests covering `AsyncRepository`.

### Fixed

- `GitBackend::open` now uses `gix::discover` instead of `gix::open`, so
  `repository(Path::new("."))` works from any subdirectory of a git worktree.

---


## [0.12.0] — 2026-05-04

### Changed

- **`JjBackend` no longer requires the `jj` binary.** The backend now opens
  jj's underlying git object store directly with gix. Both co-located
  (`.git/` + `.jj/`) and native (`.jj/repo/store/git/`) repository layouts
  are supported. `src/jj/parse.rs` has been removed.
- **`create_annotated_tag` on the jj backend** falls back to a lightweight tag
  and ignores the message; this matches jj's own tag model explicitly.
- Test modules are now in separate `tests.rs` files instead of inline `mod tests`
  blocks (`src/repository/tests.rs`).

### Added

- Test `it_works_jj_repository_error_on_non_jj_path` verifies that
  `jj_repository` rejects paths without a `.jj/` directory.
- `it_works_create_and_delete_annotated_tag` skips gracefully when no git
  committer identity is configured (avoids false failures in bare CI
  environments).

### Removed

- `src/jj/parse.rs` — CLI output parser, no longer needed.

---


## [0.11.0] — 2026-05-04

### Added

- **Jujutsu (jj) backend** — `repository::jj_repository(path)` opens a
  Jujutsu repository.  All 15 `VcsBackend` operations are implemented via `jj`
  CLI invocations (no native jj-lib dependency).  Requires `jj` on `$PATH`.
  - `src/jj/mod.rs` — `JjBackend` implementing `VcsBackend`
  - `src/jj/parse.rs` — tab-delimited jj template output parser
- **`async` feature flag** — opt-in async façade via `tokio::task::spawn_blocking`.
  - `AsyncRepository` in `src/async_api.rs`
  - `Cargo.toml`: `[features] async = ["dep:tokio"]`
  - tokio dependency is optional; default features unchanged
- **`Repository::backend_kind()`** — returns `BackendKind::Git` or
  `BackendKind::Jj`; re-exported as `endringer::BackendKind`.
- **`CommitId::from_bytes(Vec<u8>)`** — low-level constructor for backend
  implementors.
- **`CommitId::as_bytes()`** — access the raw bytes.
- **`types::BackendKind`** — `enum { Git, Jj }`.

### Architecture (Breaking changes)

- **`VcsBackend` trait** (`src/backend.rs`) — `pub(crate)` trait abstracting
  all VCS operations.  `Repository` now holds `Box<dyn VcsBackend>` instead
  of a concrete git handle.  This enables runtime backend selection.
- **`CommitId` storage changed** — internal representation changed from
  `gix::ObjectId` to `Vec<u8>`.  `CommitId::from_hex` now accepts both 40-char
  (SHA-1) and 64-char (SHA-256) hex strings.  `Display` outputs raw lowercase
  hex rather than delegating to `gix`.  **External API is unchanged.**
- **Git code moved** — `src/repository/{branch,commit,tag,diff}.rs` →
  `src/git/{branch,commit,tag,diff}.rs` as a `GitBackend` struct implementing
  `VcsBackend`.
- **`jj_repository` constructor added** to `src/repository.rs`.

### Non-breaking

- `gix::Repository` is not `Sync`; `GitBackend` wraps it in `std::sync::Mutex`
  so `Repository` is now `Send + Sync`, enabling use in async runtimes.

---


## [0.10.0] — 2026-05-04

### Added

- **`CommitInfo::committer`** — committer name (distinct from `author` after
  rebase or cherry-pick).
- **`CommitInfo::committer_timestamp`** — committer timestamp; matches the
  `committer` field.
- **`types::DiffSummary`** — file-level diff result with `added`, `modified`,
  `deleted: Vec<PathBuf>`.  Re-exported at the crate root
  (`endringer::DiffSummary`).
- **`Repository::diff(from, to)`** — returns a `DiffSummary` between two
  commits.  No patch text; renames reported as delete + add.
- **`Repository::find_commit(id)`** — O(1) object-database lookup for a single
  `CommitInfo` by `CommitId`.  Does not walk history.
- **`Repository::remote_url(name)`** — returns the fetch URL of a named remote
  (e.g. `"origin"`) as `Option<String>`.  Pure config read; no network I/O.
- 4 new unit tests: `find_commit`, `diff` (including self-diff), `remote_url`,
  `CommitInfo` committer fields.

### Changed (Breaking)

- **`CommitInfo`** gains two new fields: `committer: String` and
  `committer_timestamp: SystemTime`.  Any code constructing `CommitInfo`
  directly must be updated.

---


## [0.9.0] — 2026-05-04

### Added

- **`CommitId::from_hex(hex: &str)`** — constructs a `CommitId` from a
  40-character lowercase hex string.  Returns `CommitIdFromHexError` on
  invalid input.
- **`CommitIdFromHexError`** — error type for `CommitId::from_hex`, re-exported
  at the crate root (`endringer::CommitIdFromHexError`).
- **`types::SortOrder`** — enum with variants `NewestFirst`, `OldestFirst`,
  `ByName`.  Re-exported at the crate root (`endringer::SortOrder`).
- **`Repository::list_commits_sorted(order: SortOrder)`** — returns commit
  history sorted by the given `SortOrder`.
- **`Repository::list_tags_sorted(order: SortOrder)`** — returns tags sorted by
  the given `SortOrder`.
- **`Repository::create_annotated_tag(name, message)`** — creates a full
  annotated tag object (records tagger identity from git config and current
  timestamp).
- 3 new unit tests: annotated tags, `list_commits_sorted`, `CommitId::from_hex`.

---


## [0.8.1] — 2026-05-04

### Fixed (post-0.8.0 quality pass)

- **`status_digest` repo_name**  — `workdir()` returns `"."` when the
  repository is opened via a relative path; `file_name()` on `"."` returns
  `None`.  Fixed by calling `canonicalize()` before `file_name()`.
- **`status_digest` current_branch** — `referent_name().to_string()` returned
  the full ref (`refs/heads/master`).  Fixed by calling `.shorten()` so the
  field holds the conventional short name (`master`).
- **`commit_id_to_short_id` doc example** — imported `endringer::repository`
  (module) and called it as a function.  Corrected to
  `endringer::repository::repository`.
- **Timestamp type safety** — `gix_date::SecondsSinceUnixEpoch` is `i64`, but
  all call-sites were casting to `u64`, silently wrapping for pre-1970
  timestamps.  `seconds_to_systemtime` now accepts `i64` and saturates
  negative values to `UNIX_EPOCH`.
- **Author / committer mismatch in `CommitInfo`** — `CommitInfo.author` was
  populated from the author signature while `CommitInfo.timestamp` used
  `commit.time()` which returns the *committer* time.  Both now come from the
  author signature, matching `git log` default behaviour.

### Changed

- **`CommitId::short()`** — encodes only the first 4 raw bytes (8 hex chars)
  then truncates to 7, avoiding the full 40-char string allocation.
- **Derives** — `BranchInfo` gained `Clone + PartialEq + Eq`; `StatusDigest`,
  `CommitInfo`, `TagInfo` gained `PartialEq + Eq`.  All public types now share
  a consistent `Clone + Debug + PartialEq + Eq` baseline.
- **Release tarball naming** — changed from `endringer-v{version}.tar.gz` to
  `endringer-{version}.tar.gz` (no `v` prefix in the filename; git tag keeps
  the `v` prefix).  Example: `dist/endringer-0.8.1.tar.gz`.

### Tests

- All 10 unit tests strengthened with meaningful assertions (field-value
  checks, ordering invariants, timestamp-range guards, error-path checks).

---


## [0.8.0] — 2026-05-04

### Added

- **`types::CommitId`** — opaque SHA-1 commit identifier that replaces
  `gix::ObjectId` in the public API.  Implements `Display` (40-char hex) and
  provides `CommitId::short()` for the conventional 7-character abbreviation.
- **`types::TagInfo`** — information about a tag (name, full ref, target commit
  ID, commit summary, commit timestamp).
- **`Repository::list_tags()`** — returns all tags, peeling annotated tag
  objects to their underlying commit automatically.
- **`Repository::create_tag(name)`** — creates a lightweight tag at HEAD.
- **`Repository::delete_tag(name)`** — deletes a tag by name.
- **`Repository::log_since(since, until)`** — returns commits whose author
  timestamp falls within the given `SystemTime` range.
- Rust doc comments on all public items.

### Changed

- `types::BranchInfo`, `types::StatusDigest`, `types::CommitInfo` — all
  `last_commit_id` / `commit_id` fields changed from `gix::ObjectId` to the
  new `CommitId` type.  **Breaking change.**
- `commit_id_to_short_id` — parameter type changed from `gix::ObjectId` to
  `CommitId`.  **Breaking change.**
- `repository::branch` and `repository::commit` submodules changed from `pub`
  to `pub(crate)`.  **Breaking change.**

### Motivation

`gix::ObjectId` was leaking into the public API, forcing downstream crates to
take a transitive dependency on `gix`.  The new `CommitId` newtype closes this
boundary.  Internal submodules were made `pub(crate)` to enforce the intended
interface contract.

---


## [0.7.1] — 2025

Initial public release with branch listing, commit history, and status digest.

[Unreleased]: https://github.com/example/endringer/compare/v0.8.1...HEAD
[0.8.1]: https://github.com/example/endringer/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/example/endringer/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/example/endringer/releases/tag/v0.7.1
