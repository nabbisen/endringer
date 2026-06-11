# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.33.0] — 2026-06-11

This release closes the **"no stale docs contradictions"** stabilization gate
(item 8/9). No code changes. All documentation was audited against the current
codebase; every discrepancy found was corrected. 317 tests continue to pass.

### Changed — documentation audit

**`docs/src/reference/api-overview.md`** — complete rewrite:
- Return type corrected from `anyhow::Result<_>` to `endringer::Result<T>`.
- `remote_url` return type corrected to `Result<Option<String>>`.
- All methods added since the page was last updated are now listed: `query_commits`,
  `rich_worktree_status`, `operation_state`, `unmerged_paths`, `conflict_summary`,
  `blame_at`, `tree_at_commit`, `tree_at_path`, `diff_entries`, `snapshot`,
  `references`, `references_by_kind`, `remotes`, `ahead_behind`,
  `branch_ahead_behind`, `branch_tracking`, `local_branch_tracking`,
  `is_merged_into`, `merge_base`, `is_ancestor`, `repository_info`,
  `stash_detail`, `stash_diff`, `submodule_summaries`, `worktree_details`.

**`docs/src/reference/types.md`** — complete rewrite:
- `TagAnnotation` now shows `tagger_email` field (added v0.28.0).
- All RFC types added since v0.19.0 now documented: `CommitQuery`,
  `CommitQueryResult`, `BranchTrackingInfo`, `AheadBehind`, `RichWorktreeStatus`,
  `RichStatusEntry`, `FileStatusKind`, `StatusOptions`, `OperationState`,
  `ConflictSummary`, `ConflictPath`, `ConflictStage`, `DiffEntry`, `DiffOptions`,
  `DiffChangeKind`, `TreeEntry`, `RefInfo`, `RefTarget`, `RefKind`, `RemoteInfo`,
  `RepositoryInfo`, `HeadState`, `ObjectFormat`, `SubmoduleSummary`,
  `SubmoduleState`, `StashDetail`, `StashId`, `WorktreeDetail`, `WorktreeState`,
  `RepositorySnapshot`, `SnapshotRequest`.

**`docs/src/getting-started/quickstart.md`** — version strings updated from
`"0.19"` to `"0.32"`.

**`docs/src/introduction.md`** — capabilities table updated with all new read
surfaces added since v0.19.0: operation state, rich status, bounded history,
tree snapshots, references, remotes, detail reads, snapshot, diff entries.
"Does not do" list expanded with network operations and config management.

**`docs/src/development/architecture.md`** — complete rewrite of the
`endringer-git` module table (adds 10 new modules: `blame.rs blame_at`,
`conflict.rs`, `diff.rs diff_entries`, `info.rs`, `operation.rs`, `refs.rs`,
`stash_detail.rs`, `submodule_summary.rs`, `tree.rs`, `worktree_detail.rs`)
and the test structure (adds all 17 test files added since v0.19.0).

**`docs/src/development/contributing.md`** — release checklist updated: removes
stale `./scripts/release.sh` and `git push origin master` references; adds
`check-public-contract.sh` step; shows the correct tarball build procedure.

**`docs/src/reference/backends.md`** — stale "RFC advancement themes §3 and §5"
reference replaced with plain prose.

**`docs/src/reference/contract.md`** — three new contracts added: typed errors
since v0.23.0, `diff_entries` default behaviour, snapshot semantics.

**`README.md`** — version string updated from `"0.19"` to `"0.32"`.

**`docs/src/development/stabilization-dashboard.md`** — "no stale docs
contradictions" gate item marked ✅ Done. **8/9 gate items now complete.**
The only remaining open gate is "Maintainer v1.0 approval" — a deliberate
human decision, not a code or documentation task.

---

This release implements **RFC 014** (platform matrix), **RFC 015** (CLI parity
harness), **RFC 017** (performance benchmarks), **RFC 026** (conformance docs),
**RFC 027** (snapshot batch reads), and **RFC 028** (rename/copy detection).
It adds 27 new tests (317 total, 0 failures) and introduces Criterion
benchmarks. No breaking changes.

### Added

**RFC 014 — Platform and path robustness matrix**

- `docs/src/development/platform-matrix.md`: path-format contract, platform
  support matrix, non-UTF-8 stance, case-sensitivity note.
- `crates/endringer/tests/git_platform.rs` (8 tests): spaces in filenames,
  Unicode filenames, Unicode directories, non-UTF-8 filenames (Unix; no panic),
  symlinks reported as `TreeEntryKind::Symlink` (Unix), executable bit read
  from tree entries (Unix; graceful if unsupported), nested git repo stability.

**RFC 015 — Git CLI parity test harness**

- `crates/endringer/tests/support/git_cli.rs`: `git_output`, `git_lines`,
  `git_ahead_behind`, `git_merge_base`, `git_is_ancestor`, `git_tag_names`,
  `git_branch_names`, `git_status_short` helpers.
- `crates/endringer/tests/git_cli_parity.rs` (6 parity tests): merge_base,
  is_ancestor, ahead/behind, tag listing, branch listing, blame line count.
- `crates/endringer/tests/parity/KNOWN-DEVIATIONS.md`: documents `ChangeKind`
  simplification, tag listing order match, ahead/behind unrelated history
  behaviour.
- `docs/src/development/git-cli-parity.md`: explains test-only CLI use, command
  table, environment isolation, known deviations.

**RFC 017 — Performance benchmarks**

- `crates/endringer/benches/repository_reads.rs`: Criterion benchmark groups
  for status, refs, history, and object reads. Uses a deterministic fixture
  builder.
- `docs/src/development/performance.md`: performance classification table
  (Cheap/Moderate/Expensive), how to run benchmarks, informative baseline
  numbers.
- Criterion 0.5 added as a dev-dependency.

**RFC 026 — Custom backend conformance documentation**

- `docs/src/development/backend-conformance.md`: required vs optional
  `VcsBackend` methods, sorting contracts, owned-value contract, error model,
  thread-safety requirement, smoke-test recipe. Defers a testkit crate to
  post-v1.0.

**RFC 027 — Snapshot consistency and batch reads**

New public types, re-exported from `endringer`:

- `SnapshotRequest` struct: `include_status_digest`, `include_operation_state`,
  `include_local_branches`, `include_tags` (all `bool`). Implements `Default`
  (status + operation state on, branches + tags off).
- `RepositorySnapshot` struct: `info: RepositoryInfo`, `status_digest`,
  `operation_state`, `local_branches`, `tags` (all `Option<_>`).

New method (sync + async): `snapshot(request) -> Result<RepositorySnapshot>`.
Default `VcsBackend` implementation calls each included method sequentially.
GitBackend inherits the default (batch-optimised override deferred).
Tests: 6 in `git_snapshot_diff.rs` + 1 async parity.

**RFC 028 — Rename and copy detection**

New public types, re-exported from `endringer`:

- `DiffChangeKind` enum: `Added | Modified | Deleted | Renamed | Copied | TypeChanged | ModeChanged`.
- `DiffEntry` struct: `new_path`, `old_path`, `kind: DiffChangeKind`, `similarity: Option<u8>`.
- `DiffOptions` struct: `detect_renames: bool`, `detect_copies: bool`, `rename_threshold: Option<u8>`. Implements `Default` (all off — opt-in because detection is expensive).

New method (sync + async): `diff_entries(from, to, options) -> Result<Vec<DiffEntry>>`.
First version maps `DiffSummary` → `DiffEntry` without heuristic rename
detection (sets the API surface; detection can be added in a future release
when benchmarked). `detect_renames: true` is accepted without error.
Existing `diff()` and `DiffSummary` are unchanged.
Tests: 5 in `git_snapshot_diff.rs` + 1 async parity.

### Changed

- RFC 014, 015, 017, 026, 027, 028 moved from `rfcs/proposed/` to `rfcs/done/`.
- `docs/src/SUMMARY.md` updated with new development docs.
- `docs/src/development/stabilization-dashboard.md` updated to v0.32.0.
  **Gate status: 5/9 items now complete.** RFC 014 cleared the "path/platform
  robustness matrix" gate; RFC 015 cleared the "git CLI parity harness" gate.
  RFC 017 provides the performance baseline. Remaining open gates: no stale
  docs contradictions, maintainer v1.0 approval.

---

This release implements **RFC 013** (rich status model), **RFC 016** (dependency
policy), **RFC 018** (async semantics), **RFC 023** (SHA-256 validation), and
**RFC 025** (security hardening). It adds 25 new tests (290 total, 0 failures).
No breaking changes.

### Added

**RFC 013 — Rich status model**

New public types, re-exported from `endringer`:

- `FileStatusKind` enum: `Added | Modified | Deleted | Renamed | Copied | TypeChanged | ModeChanged | Untracked | Ignored | SubmoduleChanged`.
- `ConflictStatus` struct: `stages: Vec<u8>`.
- `RichStatusEntry` struct: `path`, `old_path`, `index: Option<FileStatusKind>`, `worktree: Option<FileStatusKind>`, `conflict: Option<ConflictStatus>`.
- `RichWorktreeStatus` struct: `entries: Vec<RichStatusEntry>`, sorted ascending by path.
- `StatusOptions` struct: `include_untracked: bool` (default `true`), `include_ignored: bool` (default `false`). Implements `Default`.

New Repository methods (sync and async):

- `rich_worktree_status(options: StatusOptions) -> Result<RichWorktreeStatus>` — maps staged, unstaged, conflict, and untracked state into `RichStatusEntry` values. Implemented in `endringer-git/src/status.rs` by compositing the existing `worktree_status()` call with conflict-stage reading from the gix index. `VcsBackend` default: `UnsupportedBackendFeature`.

Tests: `crates/endringer/tests/git_rich_status.rs` (10 tests) + 2 async parity tests.

**RFC 016 — Dependency and feature policy**

- `docs/src/development/dependency-policy.md`: per-crate dependency rules, feature flag policy, public dependency rule, when to add a crate, runtime binary policy.

**RFC 018 — Async API operational semantics**

- `docs/src/development/async-semantics.md`: `spawn_blocking` semantics, cancellation contract, recommended semaphore pattern, error mapping, sync/async parity checklist.

**RFC 023 — Object format and SHA-256 validation**

- `docs/src/reference/object-formats.md`: SHA-1/SHA-256 support matrix, `CommitId` behaviour, jj SHA-256 stance, `ObjectFormat` usage example.
- `crates/endringer/tests/git_core.rs` (3 new tests): `CommitId::from_hex` rejects wrong lengths, accepts SHA-1 and SHA-256, SHA-256 repo opens with `ObjectFormat::Sha256` (skips gracefully if git doesn't support `--object-format=sha256`).

**RFC 025 — Security and resource-exhaustion hardening**

- `docs/src/security.md`: threat model, what endringer doesn't do (no hooks, no network, no external commands), resource considerations, SHA-1 collision handling, reporting guidance.
- `crates/endringer/tests/git_hardening.rs` (10 tests): `CommitId` rejects non-hex and wrong lengths, short is always 7 chars, non-git dir returns typed error, nonexistent path returns error without panic, `file_at_commit` errors on missing path/invalid commit, bounded history never exceeds `max_count`, bare repo reads don't panic, external command guarantee documented.

### Changed

- RFC 013, 016, 018, 023, 025 moved from `rfcs/proposed/` to `rfcs/done/`.
- `docs/src/SUMMARY.md` updated with new pages in development, reference, and security sections.

---

This release implements **RFC 019** (submodule detail), **RFC 020** (stash
detail and diff), **RFC 021** (worktree detail), and **RFC 029** (documentation
cookbook). It adds 16 new tests (265 total, 0 failures). No breaking changes.

### Added

**RFC 019 — Submodule read model**

- `SubmoduleState` enum: `Registered | Initialized | MissingWorktree | MissingGitDir | Detached | Unknown`.
- `SubmoduleSummary` struct: `name`, `path`, `url`, `expected_commit_id`,
  `checked_out_commit_id`, `state`, `is_dirty: Option<bool>` (conservative
  first version — always `None`; dirty detection deferred).
- `Repository::submodule_summaries()` (sync + async). Reads `.gitmodules`,
  resolves gitlink OIDs from the index, and opens each nested repository via
  `gix::discover`. More expensive than `submodules()`. Sorted by path.
- New `endringer-git/src/submodule_summary.rs`.

**RFC 020 — Stash detail and diff reads**

- `StashId` struct: `index: usize`.
- `StashDetail` struct: `id`, `commit_id`, `message`, `author`, `timestamp`,
  `parents`.
- `Repository::stash_detail(index)` (sync + async) — detailed metadata for
  `stash@{index}`. Returns `NotFound` for invalid index.
- `Repository::stash_diff(index)` (sync + async) — `DiffSummary` of the
  stash vs its first parent. Reuses the existing `diff()` internals.
- New `endringer-git/src/stash_detail.rs`.

**RFC 021 — Linked worktree detail**

- `WorktreeState` enum: `Present | MissingPath | MissingGitFile | Prunable | Unknown`.
- `WorktreeDetail` struct: `id`, `path`, `current_branch`, `head_commit_id`,
  `is_locked`, `lock_reason`, `state`.
- `Repository::worktree_details()` (sync + async). Reads `.git/worktrees/`
  administrative directories. Missing worktrees reported as
  `WorktreeState::MissingPath` rather than omitted. Sorted by id.
- New `endringer-git/src/worktree_detail.rs`.

**RFC 029 — Documentation cookbook**

Eight new pages under `docs/src/cookbook/`:

1. `status-widget.md` — status_digest, is_dirty, worktree_status, operation_state
2. `branch-table.md` — local_branch_tracking, ahead/behind columns
3. `commit-history-browser.md` — query_commits, diff, file_at_commit, blame_at
4. `tag-management.md` — list_tags_sorted, create_tag, create_annotated_tag
5. `jj-repositories.md` — jj_repository, supported reads, git-store view limits
6. `async-multi-repo-scan.md` — AsyncRepository, Semaphore pattern
7. `write-then-read-boundary.md` — consumer writes, endringer reads, no invalidation
8. `custom-backend.md` — VcsBackend implementation, stability note

Cookbook section added to `docs/src/SUMMARY.md`.

**Tests** (16 new):

- `crates/endringer/tests/git_detail_reads.rs` (13 tests): submodule summaries
  empty/sorted/URL-present; stash detail empty error, metadata, message match,
  invalid index error, diff summary, diff invalid error; worktree details
  empty, single linked, sorted, locked with reason.
- `crates/endringer-async/tests/async_tests.rs` (3 new): submodule summaries,
  stash detail empty, worktree details empty.

### Changed

- RFC 019, 020, 021, 029 moved from `rfcs/proposed/` to `rfcs/done/`.
- `docs/src/development/stabilization-dashboard.md` updated to v0.30.0.

---

This release implements **RFC 012** (bounded history queries) and **RFC 024**
(unusual repository semantics). It adds 22 new tests (249 total, 0 failures).
No breaking changes.

### Added

**RFC 012 — Bounded history queries**

New public types, re-exported from `endringer`:

- `CommitQueryStart` enum: `Head | Commit(CommitId) | Ref(String)`.
- `CommitQuery` struct: `start`, `max_count: Option<usize>`, `skip: usize`,
  `since/until: Option<SystemTime>`, `order: SortOrder`. Constructor:
  `CommitQuery::head_page(n)` — first `n` commits from HEAD, newest first.
- `CommitQueryResult` struct: `commits: Vec<CommitInfo>`, `truncated: bool`.
  `truncated` is `true` when `max_count` was reached and more commits exist.

New `Repository` method (sync and async):

- `query_commits(query: CommitQuery) -> Result<CommitQueryResult>` — bounded
  walk starting from `Head`, a specific `CommitId`, or a named ref. Applies
  `skip` (offset-style), timestamp filters (`since`/`until`), and sort order.
  Uses the "fetch one extra" pattern for correct truncation detection.

New `VcsBackend` default (returns `UnsupportedBackendFeature`). `GitBackend`
overrides; `JjBackend` inherits the default.

**RFC 024 — Unusual repository semantics**

New integration test file `crates/endringer/tests/git_unusual_repos.rs`
(20 tests) documenting and verifying behaviour for:

- **Unborn repositories** (`git init`, no commits): `repository_info()` returns
  `HeadState::Unborn`; `list_commits()` returns empty or errors gracefully;
  `status_digest()` errors; `local_branches()` and `list_tags()` return empty.
- **Detached HEAD**: `status_digest().current_branch == "(detached)"`;
  `repository_info().head == HeadState::Detached`; `list_commits()` and
  `query_commits()` succeed.
- **Bare repositories**: open, `list_commits()`, `local_branches()`, and
  `query_commits()` all succeed; `worktree_status()` returns empty or a
  documented error.

New docs page `docs/src/reference/unusual-repositories.md`: method behaviour
matrix, `HeadState` variants, bare repository notes, unborn repository notes.
Added to `docs/src/SUMMARY.md`.

Tests:

- `git_unusual_repos.rs` (20 tests): 6 unborn, 5 detached, 5 bare, 4
  `query_commits` behaviour.
- `async_tests.rs` (2 new): `query_commits` head page, truncation flag.

### Changed

- `docs/src/development/stabilization-dashboard.md` now also reflects RFC 012
  and RFC 024 in the read surface completeness table.
- RFC 012 and RFC 024 moved from `rfcs/proposed/` to `rfcs/done/`.

---

This release implements **RFC 022** (tag API refinement) and **RFC 030**
(release quality gates and stabilization dashboard). It adds 3 new tests
(227 total, 0 failures).

### Added

**RFC 022 — Tag API refinement**

- `TagAnnotation` gains a `tagger_email: Option<String>` field.
  The Git backend populates it from the tagger signature's `email` field
  (adjacent to the already-read `name` field in `tag.rs::read_annotation`).
  This is a **breaking change** for code that constructs `TagAnnotation`
  directly: add `tagger_email: None` to all literal constructions.
- `TagInfo` doc comment updated to document peeling semantics explicitly:
  `commit_id` is always the result of peeling to a commit; tags that cannot
  be peeled are skipped by list methods.
- Migration note added to `TagInfo` doc comment.

**RFC 030 — Release quality gates and stabilization dashboard**

- `docs/src/development/release-gates.md`: three gate levels (patch, minor,
  stabilization discussion). Each level is a checklist; they are cumulative.
- `docs/src/development/stabilization-dashboard.md`: per-item status table
  covering the stabilization discussion gate. Currently 4/9 items complete;
  5 remain open. v1.0 is explicitly not planned.
- Both files added to `docs/src/SUMMARY.md`.

### Tests

- `git_tags.rs` (3 new): `annotated_tag_tagger_email_populated` verifies
  the email field matches the fixture identity (`fixture@test.local`);
  `lightweight_tag_annotation_is_none` confirms lightweight tags have no
  annotation; `annotated_tag_tagger_email_is_none_when_not_recorded`
  verifies `TagAnnotation { tagger_email: None, … }` compiles correctly.

### Changed

- RFC 022 and RFC 030 moved from `rfcs/proposed/` to `rfcs/done/`.
- `TagAnnotation` has a new public field (breaking for exhaustive struct
  literals; all usages inside endringer updated).

---

This release implements **RFC 011** (remote and reference inventory).
It adds 15 new tests (224 total, 0 failures). No breaking changes.

### Added

**RFC 011 — Remote and reference inventory**

New public types, re-exported from `endringer`:

- `RemoteInfo` struct: `name: String`, `fetch_urls: Vec<String>`,
  `push_urls: Vec<String>`. `push_urls` is empty when no explicit
  `pushurl` is configured (git falls back to the fetch URL for pushes;
  endringer reports only what is explicitly set).
- `RefKind` enum: `LocalBranch | RemoteBranch | Tag | Head | Other`.
- `RefTarget` enum: `Direct(ObjectId) | Symbolic(String) | Unborn`.
- `RefInfo` struct: `name: String`, `kind: RefKind`, `target: RefTarget`.

New `Repository` methods (sync and async):

- `remotes() -> Result<Vec<RemoteInfo>>` — all configured remotes sorted
  ascending by name. Reads `remote.<name>.url` and `remote.<name>.pushurl`
  from git config via `gix::Repository::remote_names()` +
  `find_remote()` + `.url(Direction::Fetch/Push)`.
- `references() -> Result<Vec<RefInfo>>` — all refs including HEAD, sorted
  ascending by full name. Covers local branches, remote-tracking branches,
  tags, and any other refs (notes, stash, bisect, …).
- `references_by_kind(kind) -> Result<Vec<RefInfo>>` — refs filtered by
  `RefKind`, sorted ascending. Uses gix prefix-based iteration for
  `LocalBranch`, `RemoteBranch`, and `Tag`; HEAD is a special case.

New `VcsBackend` trait methods (all have `UnsupportedBackendFeature`
defaults; `GitBackend` overrides all three; `JjBackend` inherits defaults
and delegates via the git store where applicable):

- `fn remotes(&self) -> Result<Vec<RemoteInfo>>`
- `fn references(&self) -> Result<Vec<RefInfo>>`
- `fn references_by_kind(&self, kind: RefKind) -> Result<Vec<RefInfo>>`

New `endringer-git/src/refs.rs` module.

Tests:

- `crates/endringer/tests/git_refs.rs` (12 tests): remotes empty when
  none configured, single origin after clone, remotes sorted ascending,
  explicit push URL reported separately from fetch URL; references contain
  main branch, contain HEAD, are sorted ascending, contain tag with Direct
  target, HEAD is Symbolic pointing at main; `references_by_kind` for
  local branches, tags, and HEAD.
- `crates/endringer-async/tests/async_tests.rs` (3 new): async remotes
  empty, async references contains main, async references_by_kind tags
  matches sync.

### Changed

- RFC 011 moved from `rfcs/proposed/` to `rfcs/done/`.

---

This release implements **RFC 010** (point-in-time reads and tree snapshots).
It adds 14 new tests (209 total, 0 failures). No breaking changes.

### Added

**RFC 010 — Point-in-time reads and tree snapshots**

New public types, re-exported from `endringer`:

- `TreeEntryKind` enum: `File | Directory | Symlink | Submodule | Other`.
- `TreeEntry` struct: `path: PathBuf`, `name: String`, `kind: TreeEntryKind`,
  `object_id: ObjectId`, `size: Option<u64>` (populated for blobs),
  `executable: bool`.

New `Repository` methods (sync and async):

- `tree_at_commit(commit_id) -> Result<Vec<TreeEntry>>` — non-recursive root
  tree listing at `commit_id`, sorted ascending by name.
- `tree_at_path(commit_id, path) -> Result<Vec<TreeEntry>>` — non-recursive
  listing of the directory at `path` within `commit_id`. Returns `Err` if
  `path` does not exist or is not a directory.
- `blame_at(path, commit_id) -> Result<Vec<BlameEntry>>` — per-line commit
  attribution for `path` at an arbitrary historical commit rather than HEAD.

New `VcsBackend` trait methods (all have `UnsupportedBackendFeature` defaults
per RFC 003; `GitBackend` overrides all three):

- `fn tree_at_commit(&self, commit_id: &CommitId) -> Result<Vec<TreeEntry>>`
- `fn tree_at_path(&self, commit_id: &CommitId, path: &Path) -> Result<Vec<TreeEntry>>`
- `fn blame_at(&self, path: &Path, commit_id: &CommitId) -> Result<Vec<BlameEntry>>`

New `endringer-git/src/tree.rs` module implements tree listing via the gix
tree iterator. Blob sizes are read by loading the blob object. Tree entries
are sorted ascending by name at the backend level.

`blame_at` reuses the existing gix `blame_file` call, passing the caller's
`commit_id` instead of HEAD.

Tests:

- `crates/endringer/tests/git_tree.rs` (12 tests): root listing contains
  expected files, sorting is ascending, entry kinds (file/directory), file has
  size, `tree_at_path` into subdirectory, nested directory, missing path
  returns error, root path equals `tree_at_commit`, historical commit differs
  from HEAD, `blame_at` HEAD matches `blame`, `blame_at` differs across
  commits, missing file returns error.
- `crates/endringer-async/tests/async_tests.rs` (2 new): `tree_at_commit`
  matches sync, `blame_at` returns entries.

### Changed

- RFC 010 moved from `rfcs/proposed/` to `rfcs/done/`.

---

This release implements **RFC 008** (read-side operation and conflict state).
It adds 13 new tests (195 total, 0 failures). No breaking changes.

### Added

**RFC 008 — Read-side operation and conflict state**

New public types, re-exported from `endringer`:

- `OperationState` enum: `None | Merge { heads } | Rebase { kind } | CherryPick { head } | Revert { head } | Bisect`.
- `RebaseKind` enum: `Merge | Apply | Unknown`.
- `ConflictStage` struct: `stage: u8` (1 = base, 2 = ours, 3 = theirs), `object_id: ObjectId`.
- `ConflictPath` struct: `path: PathBuf`, `stages: Vec<ConflictStage>`.
- `ConflictSummary` struct: `paths: Vec<ConflictPath>`, `is_empty()`, `len()`.

New Repository methods (sync and async):

- `operation_state() -> Result<OperationState>` — reads Git marker files in detection order: `rebase-merge/` → `rebase-apply/` → `MERGE_HEAD` → `CHERRY_PICK_HEAD` → `REVERT_HEAD` → `BISECT_LOG` / `refs/bisect/` → `None`.
- `unmerged_paths() -> Result<Vec<PathBuf>>` — sorted deduplicated paths with higher-stage index entries. Empty when no conflicts.
- `conflict_summary() -> Result<ConflictSummary>` — per-stage object IDs for every conflicted path. Uses `ObjectId` from RFC 031.

New `VcsBackend` trait methods (all have `UnsupportedBackendFeature` defaults per RFC 003; `GitBackend` overrides all three; `JjBackend` uses the unsupported defaults since jj conflicts are not index-stage conflicts):

- `fn operation_state(&self) -> Result<OperationState>`
- `fn unmerged_paths(&self) -> Result<Vec<PathBuf>>`
- `fn conflict_summary(&self) -> Result<ConflictSummary>`

Implementation modules:

- `endringer-git/src/operation.rs` — Git marker-file detection.
- `endringer-git/src/conflict.rs` — index stage reading via `gix::index`.

`RepositoryCapabilities` updated: `operation_state: true`, `conflict_state: true` for `GitBackend`.

Tests:

- `crates/endringer/tests/git_operation_state.rs` (10 tests): clean repo, merge conflict, cherry-pick conflict, revert conflict, rebase (merge backend), paths sorted, conflict summary stages, async parity.
- `crates/endringer-async/tests/async_tests.rs` (3 new): async `operation_state`, `unmerged_paths`, `conflict_summary` on clean repo.

### Changed

- `git_repository_info.rs`: updated capability assertions to reflect `operation_state: true` and `conflict_state: true` for the Git backend (RFC 008 now implemented).
- RFC 008 moved from `rfcs/proposed/` to `rfcs/done/`.

---

This release implements **RFC 007** (jj real-repository verification and CI
fixture). It adds 10 new tests (182 total, 0 failures). No public API changes.

### Added

**RFC 007 — jj real-repository verification**

- `crates/endringer/tests/support/jj_fixture.rs` — a `JjFixture` helper that
  creates native (`.jj/` only) and colocated (`.git/` + `.jj/`) jj repositories
  using the real `jj` CLI. Includes environment isolation and a `require_jj()`
  guard that skips gracefully when `jj` is absent.
- `crates/endringer/tests/jj_real.rs` — 10 tests verifying the git-store view
  against real jj repositories:
  1. open native repository
  2. open colocated repository
  3. `status_digest` reports project root name (not `git` store dir)
  4. commit history includes jj-authored commits
  5. `file_at_commit` reads from jj-created commits
  6. lightweight tag roundtrip (create → verify → delete)
  7. annotated tag returns typed `Error::UnsupportedBackendFeature { backend: Some(Jj) }`
  8. `repository_info` reports `BackendKind::Jj` and `.jj` as `vcs_dir`
  9. colocated layout has both `.git/` and `.jj/`
  10. compile-time boundary check: no jj-native concepts in public API
- `ENDRINGER_REQUIRE_JJ_CLI_TESTS=1` env var makes missing `jj` a test
  failure instead of a skip (for CI).

**Documentation**

- `docs/src/reference/backends.md` updated with a precise jj support boundary:
  what is supported (commit objects, refs, trees, lightweight tags), what is not
  (change IDs, operation log, working-copy commit, first-class conflict objects),
  and the "git-store view" stance.

**Acceptance criteria met**

- CI path: tests run when `jj` is installed, skip when absent, fail loudly with
  `ENDRINGER_REQUIRE_JJ_CLI_TESTS=1`.
- Native and colocated layouts both tested.
- `docs/src/reference/backends.md` defines the jj promise precisely.
- No runtime dependency on `jj` — library never invokes the `jj` binary.

### Changed

- RFC 007 moved from `rfcs/proposed/` to `rfcs/done/`.

---

This release implements **RFC 006** (typed public error model). It adds 26
new tests (172 total, 0 failures).

### Breaking changes

- All public sync and async methods now return `endringer::Result<T>`
  (= `std::result::Result<T, endringer::Error>`) instead of `anyhow::Result<T>`.
  Call sites using `?` are unchanged; function signatures that name `anyhow::Result`
  must be updated to `endringer::Result`.
- `Repository::remote_url(name)` now returns `Result<Option<String>>` instead
  of `Option<String>`. `Ok(None)` = no such remote; `Err` = real I/O failure.
- Custom backend implementors must update `impl VcsBackend` method signatures
  from `anyhow::Result` to `endringer_core::error::Result`.

See `docs/src/development/migration-v0.23-errors.md` for the full migration guide.

### Added

**`endringer_core::error` module**

- `Error` enum: `#[non_exhaustive]`, `Debug`, `Display`, `std::error::Error`
  (with `source()`), `From<std::io::Error>`. Not `Clone`/`PartialEq` (carries
  `io::Error` and boxed source).
- Variants: `NotARepository`, `EmptyRepository`, `NotFound { kind, name }`,
  `InvalidCommitId`, `InvalidObjectId`, `InvalidRefName`, `NotACommit`,
  `NotATree`, `PathNotFound`, `NonUtf8Path`, `BareRepositoryUnsupported`,
  `UnsupportedBackendFeature { backend, feature }`, `UnsupportedObjectFormat`,
  `HashCollision`, `CorruptRepository`, `Io`, `TaskJoin`, `Backend`.
- `NotFoundKind` enum: `#[non_exhaustive]`, `Clone/Copy/Debug/PartialEq/Eq`,
  `Display`. Variants: `Commit`, `Ref`, `Branch`, `Tag`, `Remote`, `Path`,
  `Worktree`, `Submodule`.
- `Result<T>` type alias = `std::result::Result<T, Error>`.
- `anyhow_to_backend(err)` helper for backend crates transitioning from `anyhow`.
- 9 unit tests in `endringer-core/src/error.rs`.

**Re-exports**

- `endringer::Error`, `endringer::NotFoundKind`, `endringer::Result`.
- `endringer_async::Error`, `endringer_async::NotFoundKind`, `endringer_async::Result`.

**Error classification**

- `repository()` / `jj_repository()` constructors: gix "could not find
  repository" → `NotARepository`; jj "not a jj repository" → `NotARepository`.
- `find_commit()`: gix object-not-found → `NotFound { kind: Commit }`.
- `file_at_commit()`: path absent → `PathNotFound`.
- `GitBackend::remote_url()`: gix "did not exist" → `Ok(None)`.
- `JjBackend::create_annotated_tag()`: → `UnsupportedBackendFeature { backend: Some(Jj), feature: "create_annotated_tag" }`.
- All VcsBackend defaults now return `UnsupportedBackendFeature` instead of `anyhow::bail!`.
- Unclassified gix errors → `Backend { message, source: None }` via `anyhow_to_backend`.

**`endringer-git/src/backend.rs`**

- Added `be!()` macro for converting `anyhow::Result` → `endringer_core::Result`
  at the VcsBackend dispatch boundary. Internal modules remain `anyhow`-based.

**`endringer-async`**

- `spawn_blocking` `JoinError` → `Error::TaskJoin { message }`.
- `async_tests.rs`: 2 new typed-error parity tests.

**Tests and docs**

- New `crates/endringer/tests/git_error_model.rs` (12 tests): not-a-repo,
  missing commit, missing path, invalid commit hex, jj annotated-tag
  unsupported, Display is human-readable, `Error: Send + Sync`,
  `remote_url` returns `Ok(None)`.
- `vcsbackend_defaults.rs` rewritten: 15 tests match variants, not strings.
- New `docs/src/development/migration-v0.23-errors.md` migration guide.

### Changed

- RFC 006 moved from `rfcs/proposed/` to `rfcs/done/`.

---

This release implements **RFC 005** (branch tracking and sync state) and
**RFC 009** (repository information and capability discovery), with the
branch-listing sort-order contract from RFC 005 applied to `local_branches()`
and `remote_branches()`. It adds 21 new tests (146 total, 0 failures).

### Breaking changes

- `VcsBackend` now requires implementing `repository_info` (new required core
  method). Custom backends implementing the trait directly must add an
  implementation; the `GitBackend` and `JjBackend` both do.

### Added

**RFC 005 — Branch tracking and sync state**

- `BranchTrackingInfo` struct: `branch`, `full_name`, `tip_commit_id`,
  `upstream: Option<String>`, `upstream_gone: bool`,
  `ahead_behind: Option<AheadBehind>`. Public and re-exported from `endringer`.
- `Repository::branch_tracking(branch) -> Result<BranchTrackingInfo>` —
  tracking metadata and divergence for a single local branch.
- `Repository::local_branch_tracking() -> Result<Vec<BranchTrackingInfo>>` —
  tracking metadata for all local branches, sorted ascending by full ref name.
- `Repository::is_merged_into(branch, target) -> Result<bool>` — whether
  `branch` has been merged into `target`. Named to prevent argument reversal.
- `VcsBackend::branch_tracking`, `local_branch_tracking`, and
  `is_merged_into` have unsupported-feature error defaults (RFC 003
  convention); `GitBackend` and `JjBackend` override all three.
- Upstream resolution handles `remote = "."` (local tracking) and
  `upstream_gone` (configured upstream ref no longer resolvable locally).
- Async wrappers: `AsyncRepository::branch_tracking`, `local_branch_tracking`,
  `is_merged_into`.
- 9 new integration tests in `git_branch_tracking.rs`.
- 3 new async parity tests.

**RFC 005 — Branch-listing sort order (explicit contract)**

- `local_branches()` and `remote_branches()` now guarantee ascending order
  by full ref name, enforced at the backend level. Previously the order was
  gix iteration order (ascending in practice but uncontracted). `local_branch_tracking()`
  shares the same contract.
- New integration tests `local_branches_sorted` and `local_branch_tracking_sorted`
  verify the contract.

**RFC 009 — Repository information and capability discovery**

- `RepositoryInfo` struct: `backend`, `repo_name`, `workdir`, `vcs_dir`,
  `is_bare`, `object_format`, `head`, `capabilities`.
- `ObjectFormat` enum: `Sha1 | Sha256 | Unknown(String)`, `#[non_exhaustive]`.
- `HeadState` enum: `Attached { branch, full_name, commit_id } | Detached { commit_id } |
  Unborn { branch } | Missing`, `#[non_exhaustive]`.
- `RepositoryCapabilities` struct: `working_tree`, tag write flags, `branch_tracking`,
  `operation_state` (false until RFC 008), `jj_native_state` (false until a future RFC).
- `Repository::repository_info() -> Result<RepositoryInfo>` — fresh metadata snapshot.
- `JjBackend` overrides `repo_name`, `vcs_dir`, `backend`, and `capabilities` to
  reflect the jj project root rather than the underlying git store.
- Async wrapper: `AsyncRepository::repository_info`.
- New `endringer-git/src/info.rs` module.
- 6 new integration tests in `git_repository_info.rs`.
- 3 new async parity tests.

### Changed

- RFC 005 and RFC 009 moved from `rfcs/proposed/` to `rfcs/done/`.

---

## [0.21.0] — 2026-06-10

This release implements **RFC 003** (`VcsBackend` default implementations and
extension stance), **RFC 031** (`ObjectId` identity foundation), and **RFC 004**
(ahead/behind graph computation). It adds 37 new tests (125 total, 0 failures).

### Breaking changes

- `VcsBackend` now requires implementing `ahead_behind` (new required core
  method). No existing crate-local backend was broken; only custom backends
  implementing the trait directly need updating. All other new trait methods
  have defaults.

### Added

**RFC 003 — `VcsBackend` default implementations and extension stance**

- `remote_url`, `submodules`, `stash_entries`, and `worktrees` now have
  default implementations in `VcsBackend` (returning `None` / empty `Vec`
  respectively). Custom backends no longer need to implement these to compile.
- `create_tag`, `create_annotated_tag`, and `delete_tag` now have unsupported-
  feature error defaults. Custom backends that do not support tag writes compile
  without implementing them.
- `branch_ahead_behind` has an unsupported-feature error default; backends that
  can resolve upstream config should override it.
- The `VcsBackend` module doc now states the pre-v1.0 extension stance: the
  trait is implementable but not yet fully stable; new required methods will
  always be given a default implementation where a truthful default exists.
- New integration test `vcsbackend_defaults.rs` verifies that a minimal backend
  (implementing only required core methods) compiles and that all defaults return
  the documented values.

**RFC 031 — `ObjectId` identity foundation**

- `ObjectId` and `ObjectIdFromHexError` are now public in `endringer-core` and
  re-exported from `endringer`. Mirrors `CommitId` (opaque, `Vec<u8>` backed,
  40/64 hex, `from_hex`/`from_bytes`/`as_bytes`/`short`/`Display`,
  `Clone`/`Debug`/`Eq`/`Hash`/`Ord`). No `gix` type is exposed.
- `CommitId::to_object_id(&self) -> ObjectId` — lossless conversion (a commit
  id is always a valid object id).
- `impl From<CommitId> for ObjectId` — for `.into()` call sites.
- `ObjectId::assume_commit(self) -> CommitId` — the caller asserts the object
  is a commit; endringer does not check kind. Use `Repository::find_commit` when
  verification is needed.
- `gix_id_to_object_id` helper in `endringer-git::util` for future backend
  consumers (RFC 010/011 tree and ref enumeration).
- `AheadBehind` is now defined in `endringer-core::types` (used by both
  RFC 003 defaults and RFC 004).
- `endringer-core/src/types.rs` split: identity types (`CommitId`, `ObjectId`,
  `CommitIdFromHexError`, `ObjectIdFromHexError`, hex helpers) moved to
  `types/identity.rs`; public paths are unchanged via re-exports.
- 15 new unit tests in `endringer-core` covering both identity types, the
  `CommitId`↔`ObjectId` conversions, and SHA-1/SHA-256 edge cases.

**RFC 004 — Ahead/behind graph computation**

- `Repository::ahead_behind(local, upstream) -> Result<AheadBehind>` — symmetric
  difference between two commit tips. Equivalent to
  `git rev-list --left-right --count local...upstream`. Uses a two-pass flag-
  propagation walk; cost is O(commits between the merge base and the two tips).
- `Repository::branch_ahead_behind(branch) -> Result<Option<AheadBehind>>` —
  resolves `branch.<name>.remote` + `branch.<name>.merge` from git config to
  find the upstream ref, then calls `ahead_behind`. Returns `Ok(None)` when no
  upstream is configured. Handles `remote = "."` (local upstream) correctly.
- `AheadBehind` struct: `ahead: usize`, `behind: usize`,
  `merge_base: Option<CommitId>`.
- All edge cases covered: identical tips, fast-forward in either direction,
  both diverged, unrelated histories (no merge base), merge commits in history,
  missing commit ID (returns `Err`).
- `VcsBackend::ahead_behind` is a required core method (no safe default exists).
  `VcsBackend::branch_ahead_behind` has an unsupported-feature error default
  (per RFC 003 convention); `GitBackend` and `JjBackend` override it.
- Async wrappers: `AsyncRepository::ahead_behind(local, upstream)` and
  `AsyncRepository::branch_ahead_behind(branch)`.
- 10 new integration tests in `git_graph.rs` covering all RFC 004 §7.1 and §7.2
  scenarios, with ground-truth comparison against `git rev-list --left-right --count`.
- 3 new async parity tests in `async_tests.rs`.

### Changed

- RFC 001 and RFC 002 moved from `rfcs/proposed/` to `rfcs/done/` (shipped in
  v0.20.0).
- RFC 003, RFC 031, and RFC 004 moved from `rfcs/proposed/` to `rfcs/done/`
  (shipped in this release).
- `rfcs/README.md` updated with Implemented table.

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
