# RFC 009 — Repository information and capability discovery

**Status.** Proposed  
**Priority.** P2  
**Target band.** v0.22.x  
**Breaking change.** No  
**Primary area.** Repository metadata

---

## 1. Summary

Add a lightweight `repository_info()` API and a capability model so consumers can inspect repository kind, object format, worktree availability, and backend-supported features without trial-and-error calls.

---

## 2. Motivation

A consumer often needs cheap facts before choosing which UI/actions to show:

- is this Git or jj?
- is there a working tree?
- is this bare?
- is the object format SHA-1 or SHA-256?
- are tag writes supported?
- are operation-state reads supported?

Currently, callers must infer this by calling individual methods and interpreting errors or empty results.

---

## 3. Goals

- Provide a single lightweight metadata call.
- Avoid exposing `gix` types.
- Represent backend capabilities explicitly.
- Support future APIs without forcing consumers to probe by error.

---

## 4. Non-goals

- No runtime benchmarking.
- No remote/network capability discovery.
- No guarantee that capabilities cannot change after repository mutation by another process.
- No scheduling or refresh policy.

---

## 5. External design

### 5.1 New types

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryInfo {
    pub backend: BackendKind,
    pub repo_name: String,
    pub workdir: Option<PathBuf>,
    pub vcs_dir: PathBuf,
    pub is_bare: bool,
    pub object_format: ObjectFormat,
    pub head: HeadState,
    pub capabilities: RepositoryCapabilities,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObjectFormat {
    Sha1,
    Sha256,
    /// An object format gix reported that endringer does not model; the
    /// string carries the raw format name for diagnostics.
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HeadState {
    /// HEAD points at a branch that has at least one commit.
    Attached { branch: String, full_name: String, commit_id: CommitId },
    /// HEAD is detached at a specific commit.
    Detached { commit_id: CommitId },
    /// HEAD names a branch that has no commits yet (fresh `git init`).
    Unborn { branch: Option<String> },
    /// HEAD reference is absent or unreadable.
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryCapabilities {
    pub working_tree: bool,
    pub tag_create_lightweight: bool,
    pub tag_create_annotated: bool,
    pub tag_delete: bool,
    pub branch_tracking: bool,
    pub operation_state: bool,
    pub conflict_state: bool,
    pub jj_native_state: bool,
}
```

### 5.2 API

```rust
pub fn repository_info(&self) -> Result<RepositoryInfo>;
```

### 5.3 Capability meaning

Capabilities are not permissions and not policy. They state whether the backend/API combination claims to implement a feature faithfully for this repository.

Example:

- Git backend in normal worktree:
  - `working_tree = true`
  - `tag_create_annotated = true`
  - `operation_state = true` after RFC 008
- jj backend:
  - `tag_create_lightweight = true` if verified
  - `tag_create_annotated = false`
  - `jj_native_state = false` until a future jj-native RFC

---

## 6. Internal design

### 6.1 Trait method

Add:

```rust
fn repository_info(&self) -> Result<RepositoryInfo>;
```

If RFC 003 has landed, this should likely be required, because a fake default would be misleading. However, a temporary default can return `UnsupportedBackendFeature`.

### 6.2 Git implementation

Gather:

- repo name from workdir path or git dir;
- workdir from gix repository metadata;
- git dir path;
- bare flag;
- object format from repository hash kind where available;
- HEAD state from ref resolution;
- capabilities based on backend implementation and repository kind.

### 6.3 jj implementation

Use `JjBackend` project root detection and underlying git store path.

Important: `vcs_dir` should point to the VCS metadata directory meaningful for the backend, not expose confusing store internals unless documented.

Possible fields if needed later:

```rust
pub backend_store_dir: PathBuf
```

Do not add it initially unless consumers need it.

---

## 7. Test plan

- normal Git repository;
- bare Git repository;
- detached HEAD;
- unborn branch / empty repository if supported;
- jj colocated layout;
- jj native layout after RFC 007;
- capability flags for jj annotated tag false;
- async wrapper parity.

---

## 8. Compatibility

Adds public types and methods.

No existing API changes.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Capabilities become stale after external mutation | Document that info is a fresh read, not a subscription. |
| Too many capability booleans | Keep only consumer-actionable flags. |
| Exposing paths leaks implementation detail | Use neutral names and document meaning. |

---

## 10. Acceptance criteria

- `RepositoryInfo`, `ObjectFormat`, `HeadState`, and `RepositoryCapabilities` are public.
- `repository_info()` exists in sync and async APIs.
- Git and jj implementations return truthful capabilities.
- Bare/detached/unborn states are tested or explicitly deferred.
