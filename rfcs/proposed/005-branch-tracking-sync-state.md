# RFC 005 — Branch tracking and sync state

**Status.** Proposed  
**Priority.** P1  
**Target band.** v0.22.x  
**Breaking change.** No if implemented with new types/methods; adding fields to `BranchInfo` is deferred  
**Primary area.** Branch metadata / consumer UI support

---

## 1. Summary

Add branch tracking metadata and optional divergence data without changing `BranchInfo` yet.

This RFC builds on RFC 004 and gives branch-table consumers a one-call way to render upstream, gone, ahead, behind, and merged state.

---

## 2. Motivation

`BranchInfo` currently reports isolated facts: branch name, full ref name, and tip commit. Real VCS UIs need relationship data:

- does this branch track an upstream?
- is the upstream gone?
- how far has it diverged?
- has it already been merged into a target branch?

Consumers can compute this manually, but every consumer then repeats Git config parsing and graph logic.

---

## 3. Goals

- Add upstream tracking data for local branches.
- Add gone-upstream detection.
- Add optional divergence data using RFC 004.
- Provide a batch method for branch-table UIs.
- Avoid breaking public structs with public fields until the project deliberately chooses a breaking migration.

---

## 4. Non-goals

- No fetch or remote network check.
- No pruning of stale remote refs.
- No mutation of branch configuration.
- No jj-native branch/change semantics beyond the current git view.

---

## 5. External design

### 5.1 New types

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchTrackingInfo {
    pub branch: String,
    pub full_name: String,
    pub tip_commit_id: CommitId,
    pub upstream: Option<String>,
    pub upstream_gone: bool,
    pub ahead_behind: Option<AheadBehind>,
}
```

Meaning:

- `branch`: short local branch name;
- `full_name`: `refs/heads/<branch>`;
- `tip_commit_id`: branch tip;
- `upstream`: configured upstream full ref name if configured;
- `upstream_gone`: true if upstream is configured but not resolvable locally;
- `ahead_behind`: computed only when upstream exists and resolves.

### 5.2 New API

```rust
pub fn branch_tracking(&self, branch: &str) -> Result<BranchTrackingInfo>;

pub fn local_branch_tracking(&self) -> Result<Vec<BranchTrackingInfo>>;

pub fn is_merged_into(&self, branch: &str, target: &str) -> Result<bool>;
```

Sorting:

- `local_branch_tracking()` returns entries sorted the same way as
  `local_branches()`.
- **Resolved (was an open question):** in v0.19.2, `local_branches()` /
  `remote_branches()` have **no explicit endringer sort** — `branch/util.rs`
  pushes `BranchInfo` in `gix` reference-iteration order, which is ascending
  by full ref name in practice but is not a contract endringer states. This
  RFC should make the contract explicit: sort branch listings **ascending by
  full ref name** at the backend level (mirroring the `DiffSummary` and tag
  precedent of backend-enforced ordering) and document it, so consumers can
  rely on a stable order. Add the same explicit sort to `local_branches()`
  in the same change so the two stay aligned.

### 5.3 Why not add fields to `BranchInfo` now?

`BranchInfo` has public fields. Adding fields is a source-breaking change for consumers that construct test values. Since v1 is not planned yet, avoid unnecessary breakage and introduce a new read model.

A later v1-readiness migration may merge these fields into a renamed `BranchInfo` or keep both APIs.

---

## 6. Internal design

### 6.1 Git tracking resolution

Use Git config:

```text
branch.<name>.remote
branch.<name>.merge
```

Resolution rules:

| Config | Upstream full ref |
|---|---|
| `remote = origin`, `merge = refs/heads/main` | `refs/remotes/origin/main` |
| `remote = .`, `merge = refs/heads/main` | `refs/heads/main` |
| missing remote or merge | `None` |

Pseudo-code:

```rust
fn resolve_upstream(repo: &Repository, branch: &str) -> Result<Option<String>> {
    let remote = config_string(format!("branch.{branch}.remote"))?;
    let merge = config_string(format!("branch.{branch}.merge"))?;
    match (remote, merge) {
        (Some(remote), Some(merge)) if remote == "." => Ok(Some(merge)),
        (Some(remote), Some(merge)) => {
            let short = merge.strip_prefix("refs/heads/").unwrap_or(&merge);
            Ok(Some(format!("refs/remotes/{remote}/{short}")))
        }
        _ => Ok(None),
    }
}
```

### 6.2 Batch method performance

Naively computing ahead/behind for every branch may perform many graph walks. Initial implementation may be simple; however, document cost.

Possible optimization later:

- cache commit reachability sets per upstream during one method call only;
- group branches by upstream;
- reuse merge-base results within the call.

No persistent cache is introduced.

### 6.3 `is_merged_into`

Resolve both branch names to commit IDs and call:

```rust
is_ancestor(branch_tip, target_tip)
```

This method exists to prevent consumers from reversing the arguments.

### 6.4 jj behavior

Until RFC 007 verifies jj branch/upstream behavior, jj may:

- delegate when the underlying git refs/config are available;
- return unsupported for branch tracking in native jj repositories;
- document the limitation.

---

## 7. Test plan

- branch with origin upstream;
- branch without upstream;
- branch whose upstream remote-tracking ref is deleted locally;
- branch tracking a differently named remote branch;
- branch tracking local branch with `remote = .`;
- `is_merged_into` positive and negative cases;
- batch method returns all local branches;
- async wrapper parity.

After RFC 015, compare upstream/gone behavior with `git branch -vv` and config-derived expectations where stable.

---

## 8. Compatibility

No breaking change if implemented as new types/methods.

A later breaking migration may rename `BranchInfo.last_commit_*` to `tip_commit_*` and may choose to merge tracking fields into branch info. That is deliberately out of this RFC.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Batch divergence is expensive | Keep method explicit and document cost. |
| Git config resolution is subtle | Add fixtures for non-default upstream names and local upstreams. |
| jj branch semantics differ | Gate jj claims behind RFC 007 verification. |

---

## 10. Acceptance criteria

- `BranchTrackingInfo` is public and re-exported.
- `branch_tracking`, `local_branch_tracking`, and `is_merged_into` are exposed in sync and async APIs.
- Git tests cover no-upstream, gone-upstream, divergent, and merged cases.
- No existing public struct gains fields in this RFC.
