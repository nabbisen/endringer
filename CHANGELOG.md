# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

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
