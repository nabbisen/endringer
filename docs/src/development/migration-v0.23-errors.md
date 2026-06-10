# Migration guide — v0.23.0 typed errors

v0.23.0 replaces bare `anyhow::Error` returns with a structured
`endringer::Error` enum. This guide explains what changed and how to update
code that depends on `endringer`.

---

## Who is affected

- Any code whose functions are annotated `-> anyhow::Result<T>` and call
  endringer methods (needs a type alias change or minor edit).
- Any code that downcasts `anyhow::Error` or matches on `.to_string()` to
  detect endringer error conditions (now replaced by variant matching).
- The `remote_url` method now returns `Result<Option<String>>` instead of
  `Option<String>` (small mechanical change).

Code that only uses `?` to propagate errors is largely unaffected — `?`
still works with both `anyhow::Result` and `endringer::Result` at
call sites.

---

## What changed

### 1. Error type

Before:
```rust
use anyhow::Result;   // endringer methods returned anyhow::Error
```

After:
```rust
use endringer::Result;   // endringer methods now return endringer::Error
```

### 2. Function signatures

If your function returns `anyhow::Result<T>` and calls endringer, the `?`
operator will still work in many cases because `anyhow` implements
`From<endringer::Error>` via the blanket impl. However, for clarity and
correctness, update the return type:

Before:
```rust
fn inspect(path: &Path) -> anyhow::Result<StatusDigest> {
    let repo = endringer::repository::repository(path)?;
    repo.status_digest()
}
```

After:
```rust
fn inspect(path: &Path) -> endringer::Result<StatusDigest> {
    let repo = endringer::repository::repository(path)?;
    repo.status_digest()
}
```

### 3. Error matching (the main benefit)

Before (string matching — fragile):
```rust
if err.to_string().contains("could not find repository") {
    // show setup prompt
}
```

After (variant matching — reliable):
```rust
use endringer::{Error, NotFoundKind};

match repository(path) {
    Ok(repo) => { /* … */ }
    Err(Error::NotARepository { .. }) => {
        // show setup prompt
    }
    Err(err) => return Err(err),
}
```

### 4. `remote_url` signature change

Before:
```rust
let url: Option<String> = repo.remote_url("origin");
```

After:
```rust
let url: Option<String> = repo.remote_url("origin")?;
// Ok(None)  → no such remote configured
// Ok(Some)  → URL found
// Err(..)   → real I/O / config failure (rare)
```

---

## Error variants reference

| Variant | When |
|---|---|
| `NotARepository { path }` | path is not a git/jj repository |
| `EmptyRepository` | repository has no commits yet |
| `NotFound { kind, name }` | named commit, ref, branch, tag, etc. not found |
| `InvalidCommitId { value }` | malformed commit ID hex string |
| `InvalidObjectId { value }` | malformed object ID hex string |
| `NotACommit { id }` | object exists but is not a commit |
| `PathNotFound { path, commit }` | path absent in the given commit's tree |
| `UnsupportedBackendFeature { backend, feature }` | e.g. jj annotated tags |
| `HashCollision` | SHA-1 collision detected by gix |
| `CorruptRepository { message }` | repository data appears corrupt |
| `Io(..)` | I/O error |
| `TaskJoin { message }` | async task join failure (endringer-async only) |
| `Backend { message, source }` | unclassified internal error |

Both `Error` and `NotFoundKind` are `#[non_exhaustive]` — always include a
wildcard arm when matching.

---

## Test updates

If your tests matched error strings, change them to variant matching:

Before:
```rust
assert!(err.to_string().contains("does not support"));
```

After:
```rust
assert!(matches!(err, endringer::Error::UnsupportedBackendFeature { .. }));
```

---

## Custom backend implementors

The `VcsBackend` trait now requires returning `endringer_core::Result<T>`
(same as `endringer::Result<T>`) from all required methods.

The simplest update: import `endringer_core::error::Result` and change
`anyhow::Result` to `Result` in your `impl VcsBackend` block.

You may use `anyhow` internally for error chaining and convert at the
impl boundary with `endringer_core::error::anyhow_to_backend(err)`.

`remote_url` now returns `Result<Option<String>>` instead of
`Option<String>`. Return `Ok(None)` when the remote is not configured.
