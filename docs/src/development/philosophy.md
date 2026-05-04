# Philosophy & goals

## Core principle

> endringer is a VCS adapter. It reads repository state; the caller owns
> everything else.

This means:

- **No persistence** — endringer does not write config files, logs, or caches.
- **No scheduling** — polling, file-watching, and refresh intervals are the
  caller's concern.
- **No UI** — labels, icons, colours, and i18n strings belong in the
  application layer.
- **Minimal writes** — only lightweight tag creation and deletion are in scope.
  Commits, merges, pushes, and rebases are explicitly out of scope.

## Design decisions

### gix stays internal

`gix::ObjectId` and all other `gix` types are contained behind `CommitId` and
the `pub(crate)` module boundary. A downstream crate that depends on
`endringer` does not need a `gix` dependency in its `Cargo.toml`.

This was the primary motivation for the `CommitId` newtype (introduced in
v0.8.0) and the `VcsBackend` trait (introduced in v0.11.0).

### Owned-value API

Every public method returns owned data. There are no lifetime-parameterised
return types in the public API. This simplifies downstream code: results can be
stored, sent across threads, and returned from async functions without
lifetime gymnastics.

### Lock-free concurrency

`GitBackend` wraps `gix::ThreadSafeRepository`, which provides lock-free
concurrent access by creating a cheap thread-local view per call. There is no
`Mutex` contention when multiple async tasks call the same repository
simultaneously.

### Test strategy

- Unit tests (in `src/`) run against the workspace's own git repository.
- Integration tests (in `crates/endringer/tests/`) use isolated temporary
  repositories created fresh per test via `git` CLI + `tempfile`.
- Async tests (in `crates/endringer-async/tests/`) use the same fixture approach
  with `#[tokio::test]`.

This ensures tests are reproducible and environment-independent.
