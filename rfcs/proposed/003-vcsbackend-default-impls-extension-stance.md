# RFC 003 — `VcsBackend` default implementations and extension stance

**Status.** Proposed  
**Priority.** P0  
**Target band.** v0.21.x  
**Breaking change.** Possibly minor for custom backend authors if method semantics are tightened  
**Primary area.** Public trait stability

---

## 1. Summary

Classify `VcsBackend` methods and add default implementations where a backend can reasonably opt out. This reduces the breakage cost of future read-side APIs such as ahead/behind, operation state, tree reads, and remote/ref inventory.

This RFC also requires an explicit extension stance: either `VcsBackend` is a stable public extension point, or `Repository` is stable while direct third-party `VcsBackend` implementation remains pre-v1 unstable.

---

## 2. Motivation

`VcsBackend` is public because consumers may inject custom backends. Every newly required trait method is therefore a breaking change. The handoff already notes that the trait is not yet stable.

Without default implementations, every useful feature added to the library makes custom backends harder to maintain. Installing defaults early changes this cost model.

---

## 3. Goals

- Make future trait additions less disruptive.
- Classify existing methods by whether a safe default exists.
- Avoid silently lying about backend capability.
- Document the trait's stability stance before more public APIs are added.

---

## 4. Non-goals

- Do not freeze `VcsBackend` for v1.0 in this RFC.
- Do not design the full typed error model; RFC 006 covers that.
- Do not remove custom backend support.
- Do not add ahead/behind here; RFC 004 covers it.

---

## 5. External design

### 5.1 Method categories

Classify methods as follows.

#### Required core methods

A backend must implement these because default behavior would be misleading:

- `status_digest`
- `local_branches`
- `remote_branches`
- `list_commits`
- `list_commits_sorted`
- `log_since`
- `find_commit`
- `list_tags`
- `list_tags_sorted`
- `diff`
- `is_dirty`
- `merge_base`
- `is_ancestor`
- `blame`
- `worktree_status`
- `file_at_commit`

#### Optional-empty methods

Returning an empty result is semantically valid if the backend has no such data:

- `submodules` → `Ok(vec![])`
- `stash_entries` → `Ok(vec![])`
- `worktrees` → `Ok(vec![])`

#### Optional-absent method

- `remote_url(name)` → `None`

#### Write-side exception methods

Tags are the only write-side operations in scope. A custom backend may not support them.

- `create_tag`
- `create_annotated_tag`
- `delete_tag`

These should default to an unsupported-feature error rather than silently doing nothing.

### 5.2 Temporary error before RFC 006

Before typed errors land, unsupported defaults may use `anyhow::bail!`:

```rust
fn create_tag(&self, name: &str) -> anyhow::Result<()> {
    anyhow::bail!("backend does not support create_tag({name:?})")
}
```

After RFC 006, replace this with:

```rust
Err(Error::UnsupportedBackendFeature {
    backend: self.backend_kind_if_available(),
    feature: "create_tag",
})
```

Because `VcsBackend` currently has no `backend_kind()` method, the first typed-error version may omit backend kind or accept it at the façade layer.

### 5.3 Extension stance text

Add to rustdoc:

```rust
/// Public backend extension trait.
///
/// Before v1.0, this trait is implementable but not fully stable. New methods
/// may still be added, but new methods should provide default implementations
/// whenever a truthful default exists.
///
/// Consumers that only use `Repository` should receive stronger stability than
/// consumers implementing `VcsBackend` directly.
```

This stance avoids pretending the trait is fully frozen while still reducing churn.

---

## 6. Internal design

### 6.1 Trait defaults

Add defaults directly to `endringer-core/src/backend.rs`.

Example:

```rust
fn remote_url(&self, _name: &str) -> Option<String> {
    None
}

fn submodules(&self) -> Result<Vec<SubmoduleInfo>> {
    Ok(Vec::new())
}

fn stash_entries(&self) -> Result<Vec<StashEntry>> {
    Ok(Vec::new())
}

fn worktrees(&self) -> Result<Vec<WorktreeInfo>> {
    Ok(Vec::new())
}

fn create_tag(&self, name: &str) -> Result<()> {
    anyhow::bail!("backend does not support lightweight tag creation: {name}")
}
```

### 6.2 Future default style

For every future trait method, the RFC introducing it must say which default category applies:

- **required:** no default;
- **empty:** `Ok(vec![])` / `None` because absence is meaningful;
- **unsupported:** error because an empty result would mislead;
- **derived:** implemented in terms of existing trait methods.

Example derived method after RFC 004:

```rust
fn is_merged_into(&self, branch_tip: &CommitId, target: &CommitId) -> Result<bool> {
    self.is_ancestor(branch_tip, target)
}
```

---

## 7. Test plan

- Add a test-only minimal backend that implements only required methods.
- Verify it compiles without implementing optional methods.
- Verify optional-empty defaults return empty values.
- Verify tag defaults return errors.
- Verify `Repository::with_backend` works with the minimal backend.

Example test concept:

```rust
struct MinimalBackend;

impl VcsBackend for MinimalBackend {
    fn status_digest(&self) -> Result<StatusDigest> { todo!() }
    // required methods only
}

#[test]
fn optional_methods_have_defaults() {
    let backend = MinimalBackend;
    assert_eq!(backend.submodules().unwrap(), Vec::new());
    assert_eq!(backend.remote_url("origin"), None);
    assert!(backend.create_tag("v0").is_err());
}
```

---

## 8. Compatibility

Adding defaults is source-compatible for existing backend implementors.

If error messages change for custom backends relying on default methods later, that is acceptable before v1 but should be recorded.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Empty defaults hide unsupported features | Use empty defaults only where empty is a true repository state. |
| Trait still changes too often | Require every future RFC to specify default category. |
| Unsupported errors are untyped until RFC 006 | Use clear messages temporarily and replace with typed errors later. |

---

## 10. Acceptance criteria

- `submodules`, `stash_entries`, `worktrees`, and `remote_url` have defaults.
- Tag write methods have unsupported defaults.
- Rustdoc states the pre-v1 extension stance.
- A minimal custom backend test demonstrates reduced implementation burden.
