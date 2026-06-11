# Public contract statements

This page records high-value behavioral contracts in one place. When rustdoc,
the README, and this page disagree, file a bug — they must be consistent.
Tests that enforce these contracts are listed alongside each entry.

---

## Working-tree status

- `worktree_status().untracked` applies active gitignore rules when the
  backend can build the exclude stack (`.gitignore`, `info/exclude`, global
  excludes via `gix::Repository::excludes()`).
- If exclude-stack construction fails, the git backend degrades gracefully
  and reports all untracked files without filtering. No warning is surfaced
  to the caller in the degraded path.
- Bare repositories return an empty `WorktreeStatus` with no staged, unstaged,
  or untracked entries.
- `is_dirty()` returns `false` for bare repositories.

**Enforcing tests:** `crates/endringer/tests/git_status.rs`

---

## jj tags

- `create_tag(name)` creates a lightweight tag on jj repositories.
- `create_annotated_tag(name, message)` returns an explicit error on jj
  repositories. It does **not** silently fall back to creating a lightweight
  tag. The error message directs callers to use `create_tag` instead.
- jj does not support annotated tags through the current endringer API.

**Enforcing tests:** `crates/endringer/tests/jj.rs` (error path);
`crates/endringer-jj/src/tests.rs`

---

## Diff path ordering

- `DiffSummary.added`, `.modified`, and `.deleted` are sorted in ascending
  lexicographic order within each category.
- This ordering is enforced at the backend level, not as post-processing.
  Callers may rely on it without sorting again.

**Enforcing tests:** `crates/endringer/tests/git_diff.rs`

---

## Tag peel semantics

- `TagInfo.commit_id` is the commit reached by peeling the tag's target
  (following tag objects) down to a commit object.
- Tags that cannot be peeled to a commit are currently skipped in list
  methods. This behaviour may be refined in a future release.
- `TagAnnotation` is `None` for lightweight tags and `Some` for annotated
  tags.

**Enforcing tests:** `crates/endringer/tests/git_tags.rs`,
`crates/endringer/tests/git_worktree.rs` (annotation extraction)

---

## No public `gix` types

- No `gix` type appears in the public API surface of any `endringer` crate.
- `CommitId` hides `gix::ObjectId`. Backend modules are `pub(crate)`.
- Downstream crates have zero compile-time dependency on `gix`.

**Enforcing check:** build `endringer` without `endringer-git` or `endringer-jj`
in scope to verify no `gix` type leaks through `endringer-core`.

---

## Stash entry ordering

- `stash_entries()` returns entries newest-first: `stash@{0}` has index `0`.
- An empty stash returns an empty `Vec`, not an error.

**Enforcing tests:** `crates/endringer/tests/git_submodule_stash.rs`

---

## Concurrency

- A single `Repository` handle may be used from multiple threads concurrently
  without external synchronisation.
- `GitBackend` stores a `gix::ThreadSafeRepository`; each method call takes a
  cheap thread-local view via `to_thread_local()`. No mutex is held.
- `Repository` is `Send + Sync`.

---

## No external binaries at runtime

- Neither `git` nor `jj` is invoked by the library at runtime.
- Both backends read the object store directly through `gix`.
- The `git` CLI is used only in integration test fixtures (build/test
  dependency, not runtime).

---

## Typed errors since v0.23.0

- All public methods return `endringer::Result<T>`, not `anyhow::Result<T>`.
- Unknown objects/refs return `Error::NotFound { kind, name }`.
- Opening a non-repository path returns `Error::NotARepository`.
- Unsupported optional backend features return `Error::UnsupportedBackendFeature`.
- `remote_url(name)` returns `Result<Option<String>>` — `Ok(None)` when the remote does not exist.

**Enforcing tests:** `crates/endringer/tests/git_error_model.rs`

---

## `diff_entries` default behaviour

- `diff_entries(from, to, DiffOptions::default())` returns the same paths as
  `diff(from, to)` expressed as `DiffEntry` values, with no rename/copy
  detection applied.
- `detect_renames: true` is accepted without error. When heuristic detection
  is not yet implemented for a given backend, the output is identical to the
  default.
- The `DiffSummary` returned by `diff()` is always stable and cheap.

**Enforcing tests:** `crates/endringer/tests/git_snapshot_diff.rs`

---

## Snapshot semantics

- `snapshot()` is a batch read, not an atomic snapshot. A concurrent repository
  mutation can produce a mixed view across the included fields.
- `RepositoryInfo` is always populated regardless of `SnapshotRequest` flags.
- Fields not requested are `None` in the result.

**Enforcing tests:** `crates/endringer/tests/git_snapshot_diff.rs`

---

## `query_commits` truncation

- `CommitQueryResult::truncated` is `true` if and only if `max_count` was
  provided and at least one more commit exists beyond the returned page.
- `skip` offsets from the start of the walk; it is O(skip) in the history depth.
- No commit appears in the result more than once.

**Enforcing tests:** `crates/endringer/tests/git_unusual_repos.rs`
