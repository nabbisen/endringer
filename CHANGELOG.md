# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.8.0] — 2026-05-04

### Added

- **`types::CommitId`** — opaque SHA-1 commit identifier that replaces `gix::ObjectId`
  in the public API.  Implements `Display` (40-char hex) and provides
  `CommitId::short()` for the conventional 7-character abbreviation.
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
  to `pub(crate)` — they were never intended as public API.  **Breaking change.**
- New `repository::tag` submodule added as `pub(crate)`.

### Motivation

The `gix::ObjectId` type was leaking into the public API of `BranchInfo`,
`StatusDigest`, and `CommitInfo`, forcing downstream crates to take a
transitive dependency on `gix` even if they only needed endringer's
higher-level abstractions.  The new `CommitId` newtype closes this boundary.

Similarly, the internal `repository::branch` / `repository::commit` submodules
were `pub`, making it possible for callers to bypass `Repository` and call
internal helpers directly.  Making them `pub(crate)` enforces the intended
interface contract.

---

## [0.7.1] — 2025

Initial public release with branch listing, commit history, and status digest.
