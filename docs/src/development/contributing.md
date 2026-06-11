# Contributing & local development

## Prerequisites

- Rust 1.85 or later (the workspace uses `edition = "2024"`)
- `git` available on `$PATH` (required by integration tests that call the git CLI
  to set up fixture repositories)

## Building

```sh
cargo build --workspace
```

## Running tests

```sh
# All library and integration tests (recommended)
cargo test --workspace --lib --tests

# A specific crate
cargo test -p endringer --lib --tests

# A specific test file
cargo test -p endringer --test git_commits
```

**Note on doctests**: `cargo test --doc` may fail in some environments due to
a version mismatch between cargo and `rustdoc`. Use `--lib --tests` to run all
meaningful tests without doctests.

## Test architecture

Unit tests in `src/*/tests.rs` use the workspace's own git history (discovered
via `Path::new(".")`). These are fast but depend on the repository having at
least one commit.

Integration tests in `crates/endringer/tests/` create isolated temporary
repositories using `tempfile::TempDir` + git CLI commands. Each test function
creates its own fresh repository so tests are fully independent.

The shared helper is `tests/support/fixture.rs`, included via:

```rust
#[path = "support/fixture.rs"]
mod fixture;
```

This avoids `mod.rs` while sharing fixture code across test files.

## Adding a new backend method

1. Add the method signature to `VcsBackend` in `endringer-core/src/backend.rs`.
2. Implement it in `endringer-git/src/` (usually a new module or existing one).
3. Delegate in `endringer-git/src/backend.rs` via `repo!(self)`.
4. Delegate in `endringer-jj/src/backend.rs` (usually `self.git.method()`).
5. Expose on `Repository` in `endringer/src/repository.rs`.
6. Re-export any new types from `endringer/src/lib.rs`.
7. Add the async wrapper in `endringer-async/src/async_api.rs`.
8. Write integration tests in `crates/endringer/tests/`.

## Release checklist

1. Bump `version` in workspace `Cargo.toml`.
2. Add a `[x.y.z]` section to `CHANGELOG.md`.
3. Update the release history table in `ROADMAP.md`.
4. Run `cargo test --workspace --lib --tests` — all must pass.
5. Run `sh scripts/check-public-contract.sh` — must pass.
6. Commit all changes, then `git tag vX.Y.Z`.
7. Build the release archive:
   ```sh
   STAGING=$(mktemp -d)
   cp -a . "$STAGING/"
   rm -rf "$STAGING/.git" "$STAGING/target"
   tar -czf endringer-X.Y.Z.tar.gz -C "$STAGING" .
   sh scripts/verify-release-manifest.sh "$STAGING"
   rm -rf "$STAGING"
   ```
8. Publish crates in dependency order:
   ```sh
   cargo publish -p endringer-core
   cargo publish -p endringer-git
   cargo publish -p endringer-jj
   cargo publish -p endringer
   cargo publish -p endringer-async
   ```
