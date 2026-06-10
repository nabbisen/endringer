# RFC 019 — Submodule read model and submodule status summary

**Status.** Proposed  
**Priority.** P3  
**Target band.** v0.29.x  
**Breaking change.** Adds API  
**Primary area.** Repository metadata / submodules

---

## 1. Summary

Extend the read-only submodule surface from simple `.gitmodules` metadata to a more useful submodule summary, while preserving the rule that endringer does not initialize, update, fetch, or mutate submodules.

---

## 2. Motivation

Current `SubmoduleInfo` exposes name, path, and URL. That is enough for inventory, but VCS-aware UIs often need to show whether a submodule is initialized, whether its checked-out commit differs from the superproject gitlink, and whether it has dirty state.

Submodule mutation is out of scope, but reading submodule state is in scope.

---

## 3. Goals

- Keep existing `submodules()` behavior stable.
- Add a richer summary method rather than overloading simple metadata.
- Report initialization/presence state.
- Report expected gitlink commit where available.
- Optionally report checked-out HEAD and dirty marker.
- Avoid recursive deep scans by default.

---

## 4. Non-goals

- Do not run `git submodule update`.
- Do not fetch submodule remotes.
- Do not recursively scan nested submodules unless a future RFC adds an explicit option.
- Do not make submodule dirty detection expensive by default.

---

## 5. External design

### 5.1 New types

```rust
pub struct SubmoduleSummary {
    pub name: String,
    pub path: PathBuf,
    pub url: Option<String>,
    pub expected_commit_id: Option<CommitId>,
    pub checked_out_commit_id: Option<CommitId>,
    pub state: SubmoduleState,
    pub is_dirty: Option<bool>,
}

pub enum SubmoduleState {
    Registered,
    Initialized,
    MissingWorktree,
    MissingGitDir,
    Detached,
    Unknown,
}
```

`is_dirty` is `Option<bool>` because dirty detection may be skipped or unsupported.

### 5.2 New methods

```rust
Repository::submodule_summaries() -> Result<Vec<SubmoduleSummary>>
```

The existing `submodules()` remains the cheap metadata call.

### 5.3 Cost model

`submodule_summaries()` may open submodule repositories locally, but it must not recurse by default. It should sort by path, matching the existing `submodules()` contract.

---

## 6. Internal design

### 6.1 Git backend

Read `.gitmodules` and index gitlink entries to determine expected commits. For initialized submodules, discover the nested repository from the submodule path and read HEAD.

Dirty detection should be conservative:

- first implementation may set `is_dirty: None`;
- later implementation may call nested endringer status reads;
- any failure to inspect nested state should not fail the entire superproject call unless the metadata itself is corrupt.

### 6.2 jj backend

The jj backend delegates to the git view. Document any jj-specific uncertainty until RFC 007 verification covers submodules.

### 6.3 Errors

Use typed errors after RFC 006 for corrupt submodule metadata. Missing submodule working directories should be data states, not hard errors.

---

## 7. Tests and verification

- Fixture with registered but uninitialized submodule.
- Fixture with initialized clean submodule.
- Fixture where submodule HEAD differs from superproject gitlink.
- Fixture where submodule worktree is missing.
- Sorting by path.
- No network access.

---

## 8. Rollout plan

1. Keep `submodules()` unchanged.
2. Add `SubmoduleSummary` and `submodule_summaries()`.
3. Start with `is_dirty: None` if necessary.
4. Add optional dirty detection later only if cost and semantics are clear.

---

## 9. Risks and mitigations

**Risk: submodule semantics are complex.** Keep the first summary conservative.

**Risk: expensive nested status calls.** Do not perform recursive or dirty checks by default unless explicitly designed.

**Risk: turning missing submodules into errors.** Model them as states.

---

## 10. Definition of done

- Simple metadata API remains unchanged.
- Rich summary API exists and is documented as costlier.
- Tests cover initialized/uninitialized/missing cases.
- No submodule mutation or network operation is introduced.
