# RFC 024 — Empty, bare, detached, and unusual repository semantics

**Status.** Implemented (v0.29.0)  
**Priority.** P2  
**Target band.** v0.28.x  
**Breaking change.** May clarify errors; should avoid unnecessary breakage  
**Primary area.** Repository state semantics

---

## 1. Summary

Define and test how endringer behaves for empty repositories, bare repositories, detached HEAD, unborn branches, missing HEAD, and repositories with unusual but valid states.

---

## 2. Motivation

Real tools often point libraries at repositories that are not clean, normal, non-bare, branch-attached working trees. A dependable VCS library should have explicit semantics for these cases rather than accidental behavior.

The current `StatusDigest` uses `"(detached)"` for detached branches. That may be adequate, but other states such as unborn HEAD or bare repositories need stronger contracts.

---

## 3. Goals

- Document state semantics for empty/unborn repositories.
- Document bare repository support per method.
- Replace magic strings with typed state where a new API is feasible.
- Preserve current compatibility unless a breaking band is chosen.
- Add fixtures for unusual states.

---

## 4. Non-goals

- Do not make every method work on every repository shape.
- Do not mutate repositories to normalize them.
- Do not run git repair commands.

---

## 5. External design

### 5.1 Head state type

`HeadState` is **owned by RFC 009** and defined there as:

```rust
#[non_exhaustive]
pub enum HeadState {
    Attached { branch: String, full_name: String, commit_id: CommitId },
    Detached { commit_id: CommitId },
    Unborn { branch: Option<String> },
    Missing,
}
```

This RFC reuses that definition rather than redeclaring it. (An earlier draft
here defined `Attached { branch, commit_id }` without `full_name`, conflicting
with RFC 009's `Branch { name, full_name }`; the unified form above — carrying
short name, full ref name, and commit id — resolves both.)

Do not necessarily replace `StatusDigest.current_branch` immediately. Instead,
add richer repository/head info first via RFC 009.

### 5.2 Method behavior matrix

Create a docs table:

| Method | Empty/unborn | Bare | Detached |
|---|---|---|---|
| `status_digest` | returns typed empty/unborn state or error | supported without worktree fields? | supported |
| `worktree_status` | empty status | unsupported/empty documented | supported |
| `list_commits` | empty vec | supported | supported |
| `local_branches` | empty or unborn branch documented | supported | supported |
| `file_at_commit` | not found | supported if commit specified | supported |

### 5.3 Magic string migration

The existing `"(detached)"` string can remain for compatibility in `StatusDigest` and `WorktreeInfo`, but new APIs should use typed `HeadState`.

---

## 6. Internal design

### 6.1 Fixtures

Add fixtures for:

- `git init` with no commits;
- detached HEAD;
- bare repository;
- repository with unborn branch;
- missing/corrupt HEAD if safe to construct;
- linked worktree detached HEAD.

### 6.2 Errors

After RFC 006, prefer typed variants:

- `EmptyRepository`;
- `BareRepositoryUnsupported { operation }`;
- `NotFound { kind: Ref, name: "HEAD" }` only when semantically missing rather than unborn.

### 6.3 Backward compatibility

Do not change existing methods from string to enum without a planned breaking band. Add new richer methods first.

---

## 7. Tests and verification

- Integration tests for each state.
- Method matrix validated at least for major methods.
- Docs include exact behavior.
- Existing consumers are not broken unless intentionally bundled with a breaking release.

---

## 8. Rollout plan

1. Document current behavior by test.
2. Add `HeadState` via repository info/capabilities.
3. Improve typed errors after RFC 006.
4. Consider replacing magic strings only in a later breaking cleanup.

---

## 9. Risks and mitigations

**Risk: over-normalizing unusual states.** Report states honestly.

**Risk: breaking existing `current_branch` users.** Add typed alternatives before replacement.

**Risk: bare repository semantics are method-specific.** Use a behavior matrix.

---

## 10. Definition of done

- Empty, bare, detached, and unborn repository behavior is documented.
- Fixtures cover major unusual states.
- Typed `HeadState` exists or is explicitly assigned to RFC 009.
- No accidental behavior remains undocumented.
