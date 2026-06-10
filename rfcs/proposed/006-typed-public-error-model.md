# RFC 006 — Typed public error model

**Status.** Proposed  
**Priority.** P1  
**Target band.** v0.23.x  
**Breaking change.** Yes  
**Primary area.** API stabilization / consumer ergonomics

---

## 1. Summary

Replace public `anyhow::Result<T>` returns with an `endringer` typed error model.

The main goal is to let consumers distinguish common failures without string matching: not a repository, missing commit, missing ref, unsupported backend feature, corrupt object, I/O failure, invalid commit ID, path not found, and hash collision.

This is a pre-v1 stabilization change. It should happen before the public API grows much further.

---

## 2. Motivation

`anyhow` is good for application boundaries and internal prototyping, but it is weak as a library API contract. Consumers of `endringer` need to answer questions like:

- should I show "not a repository" or "repository is corrupt"?
- should I hide a feature because the backend does not support it?
- should I ask the user to fetch/prune because an upstream ref is gone?
- should I retry after I/O failure?
- should I report a security-sensitive hash collision?

Today they cannot do this reliably without parsing error strings.

---

## 3. Goals

- Introduce `endringer_core::Error` and `endringer_core::Result<T>`.
- Re-export them from `endringer` and `endringer-async`.
- Map backend errors into stable public variants where feasible.
- Preserve error sources for debugging.
- Make unsupported backend behavior explicit.

---

## 4. Non-goals

- Do not expose `gix` error types publicly.
- Do not classify every possible internal error perfectly in the first version.
- Do not remove helpful context messages.
- Do not change runtime behavior except error types.

---

## 5. External design

### 5.1 Result alias

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

### 5.2 Error enum

Initial public shape:

```rust
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    NotARepository { path: std::path::PathBuf },
    EmptyRepository,
    NotFound { kind: NotFoundKind, name: String },
    InvalidCommitId { value: String },
    InvalidObjectId { value: String },
    InvalidRefName { value: String },
    NotACommit { id: CommitId },
    NotATree { id: ObjectId },
    PathNotFound { path: std::path::PathBuf, commit: Option<CommitId> },
    NonUtf8Path { path: std::path::PathBuf },
    BareRepositoryUnsupported { operation: &'static str },
    UnsupportedBackendFeature { backend: Option<BackendKind>, feature: &'static str },
    UnsupportedObjectFormat { format: String },
    HashCollision,
    CorruptRepository { message: String },
    Io(std::io::Error),
    TaskJoin { message: String },
    Backend { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotFoundKind {
    Commit,
    Ref,
    Branch,
    Tag,
    Remote,
    Path,
    Worktree,
    Submodule,
}
```

`ObjectId` is the foundation type from **RFC 031**, which must land before
this RFC for exactly this reason (it is referenced by `NotATree` here and by
RFCs 008/010/011). With RFC 031 sequenced into v0.21.x, the earlier
"if `ObjectId` does not exist yet, `NotATree` may temporarily use `String`"
hedge is no longer needed and should be removed. `InvalidObjectId` is the
sibling of `InvalidCommitId` for `ObjectId::from_hex` failures.

Both enums are `#[non_exhaustive]` so new variants (and new
`NotFoundKind`s) are non-breaking additions — consumers must already include
a wildcard arm. This is the single most important property for letting later
RFCs (008 conflict state, 010 trees, 011 refs) add error cases without a new
breaking wave.

#### Required trait implementations

The public error type must implement the standard error surface so `?` and
the wider ecosystem work:

```rust
impl std::fmt::Display for Error { /* one human line per variant */ }
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Backend { source: Some(s), .. } => Some(&**s),
            _ => None,
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Error::Io(e) }
}
```

`Error` is intentionally **not** `Clone`/`PartialEq` (it carries
`std::io::Error` and a boxed `source`); tests therefore match on variants via
`matches!`, never on equality or `Display` strings (see §7).

### 5.3 Error classification expectations

| Scenario | Error |
|---|---|
| opening a non-repository path | `NotARepository` |
| HEAD absent in empty repository | `EmptyRepository` or `NotFound { kind: Ref, name: "HEAD" }` |
| requested commit missing | `NotFound { kind: Commit, ... }` |
| object exists but is not commit | `NotACommit` |
| path absent at commit | `PathNotFound` |
| jj annotated tag | `UnsupportedBackendFeature { backend: Some(Jj), feature: "create_annotated_tag" }` |
| bare repo worktree status | existing empty result may remain if this is current contract |
| I/O failure | `Io` |
| gix collision-detecting hasher collision | `HashCollision` |
| unclassified gix/internal failure | `Backend` or `CorruptRepository` |

### 5.4 Migration guide

Add `docs/src/development/migration-v0.23-errors.md`:

The change is invisible to most call sites because `?` still works — the
difference is in the **error type a function names**, not in the `?`
operator. The "before" and "after" of the *call* look identical:

```rust
// both before and after:
let repo = repository(path)?;
```

What changes is any signature, type alias, or `match`/`downcast` that names
the error type:

Before:

```rust
use anyhow::Result;

fn inspect(path: &Path) -> Result<StatusDigest> {     // anyhow::Result
    let repo = endringer::repository::repository(path)?;
    repo.status_digest()
}

// matching had to inspect strings:
if err.to_string().contains("Could not find a git repository") { /* ... */ }
```

After:

```rust
fn inspect(path: &Path) -> endringer::Result<StatusDigest> {  // endringer::Result
    let repo = endringer::repository::repository(path)?;
    repo.status_digest()
}

// matching is now structural:
match repository(path) {
    Ok(repo) => { /* ... */ }
    Err(endringer::Error::NotARepository { .. }) => { /* show setup prompt */ }
    Err(err) => return Err(err),
}
```

Consumers that previously relied on `anyhow::Error`'s `.context()` chaining
or `.downcast_ref::<T>()` need the most attention; everything reachable
through plain `?` is mechanical.

### 5.5 The `remote_url` signature anomaly (decide here)

`Repository::remote_url(name) -> Option<String>` is today the **only**
fallible-looking public method that does not return `Result`: it collapses
both "no such remote" and any underlying config/I/O error into `None`,
silently discarding the error. Under a typed error model this is the one
place a real failure can vanish.

This RFC must pick one, because the choice is itself a (small) breaking
change best bundled into this wave:

- **Option 1 — keep `Option<String>`, document the collapse.** Lowest churn;
  acceptable because the richer `remotes()` API in RFC 011 returns
  `Result<Vec<RemoteInfo>>` and becomes the place to observe errors.
- **Option 2 — promote to `Result<Option<String>>`.** `Ok(None)` = no such
  remote; `Err(..)` = a real failure. Consistent with every other method and
  with the typed model.

**Recommended: Option 2**, taken in this same breaking release. `remote_url`
keeps its convenience role, errors stop disappearing, and the
absence-vs-failure distinction that the whole RFC is about is honoured even
for this method. RFC 003's "optional-absent default" for `remote_url` is then
restated as `Ok(None)` rather than `None`.

---

## 6. Internal design

### 6.1 Crate placement

Place the error type in `endringer-core`:

```text
crates/endringer-core/src/error.rs
```

Re-export from:

- `endringer-core::Error`, `endringer_core::Result`;
- `endringer::Error`, `endringer::Result`;
- `endringer_async::Error`, `endringer_async::Result`.

### 6.2 Mapping layer

Avoid leaking `gix` through public errors. Backend modules map gix errors to `Error`.

Example:

```rust
fn map_find_object_error(err: gix::object::find::existing::Error, id: &CommitId) -> Error {
    // Pseudo-code; exact gix types may differ.
    if is_not_found(&err) {
        Error::NotFound { kind: NotFoundKind::Commit, name: id.to_string() }
    } else if is_corrupt(&err) {
        Error::CorruptRepository { message: err.to_string() }
    } else {
        Error::Backend { message: err.to_string(), source: Some(Box::new(err)) }
    }
}
```

### 6.3 Transition strategy

Because this is breaking, do it in one concentrated release.

Possible intermediate step:

- internally introduce `Error` and `Result`;
- keep public signatures as `anyhow::Result` for one release;
- then switch public signatures.

However, dragging both models for too long increases maintenance cost. Preferred: one clear breaking pre-v1 release.

### 6.4 Async `JoinError`

`spawn_blocking(...).await` can fail independently of repository logic. Map that to:

```rust
Error::TaskJoin { message }
```

Do not collapse it into `Backend`.

---

## 7. Test plan

Add tests for:

- opening a non-repo path;
- finding a missing commit;
- file-at-commit missing path;
- jj annotated tag unsupported;
- invalid commit hex;
- async task join mapping if practical;
- branch upstream gone after RFC 005;
- error `Display` output is useful but tests match variants, not full strings.

Use pattern matching in tests:

```rust
assert!(matches!(err, Error::UnsupportedBackendFeature { backend: Some(BackendKind::Jj), feature: "create_annotated_tag" }));
```

---

## 8. Compatibility

This is source-breaking for consumers whose APIs mention `anyhow::Result` or `anyhow::Error` from `endringer` calls.

The migration is straightforward for normal `?`-using consumers, but it must be documented clearly.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Mapping gix errors is tedious | Start with high-value variants and use `Backend` for the rest. |
| Error enum becomes too large | Keep variants consumer-actionable. |
| Public enum becomes hard to extend | Mark enum `#[non_exhaustive]`. |
| Breaking change frustrates users | Do it before v1 and provide a migration guide. |

---

## 10. Acceptance criteria

- Public sync and async APIs return `endringer::Result<T>`.
- `Error` is `#[non_exhaustive]` unless the project explicitly rejects that.
- Common consumer cases are matchable without strings.
- Existing tests are migrated away from string matching where variants exist.
- Migration guide exists.
