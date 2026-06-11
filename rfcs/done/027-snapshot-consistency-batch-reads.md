# RFC 027 — Snapshot consistency and batch read APIs

**Status.** Implemented (v0.32.0)  
**Priority.** P3  
**Target band.** v0.30.x or later  
**Breaking change.** Adds API; no change to existing no-cache model  
**Primary area.** Read lifecycle / API ergonomics

---

## 1. Summary

Explore optional batch read APIs that provide a more consistent view across related reads without introducing persistent caching or background refresh.

This RFC is intentionally conservative: it does not replace the current no-state lifecycle.

---

## 2. Motivation

The handoff states that no state is held between calls: each method reads fresh on-disk state and returns owned data. That is a good and simple contract.

However, a consumer may want `status_digest`, branch info, and operation state from approximately the same repository moment. If the repository changes between calls, the consumer can observe a mixed view. Most widgets tolerate this, but richer UIs may benefit from a batch read that collects related data in one backend view.

---

## 3. Goals

- Preserve current method behavior: every call sees current disk state.
- Add optional batch APIs for common UI snapshots.
- Avoid persistent caches and invalidation semantics.
- Avoid long-lived borrowed snapshots.
- Keep results owned.

---

## 4. Non-goals

- Do not add a cache.
- Do not add file watching.
- Do not promise transactional consistency against concurrent external writes.
- Do not expose gix thread-local views or lifetimes.

---

## 5. External design

### 5.1 New snapshot result type

```rust
pub struct RepositorySnapshot {
    pub info: RepositoryInfo,
    pub status_digest: Option<StatusDigest>,
    pub operation_state: Option<OperationState>,
    pub local_branches: Option<Vec<BranchInfo>>,
    pub tags: Option<Vec<TagInfo>>,
}
```

### 5.2 Query object

```rust
pub struct SnapshotRequest {
    pub include_status_digest: bool,
    pub include_operation_state: bool,
    pub include_local_branches: bool,
    pub include_tags: bool,
}

Repository::snapshot(request: SnapshotRequest) -> Result<RepositorySnapshot>
```

### 5.3 Semantics

The snapshot is **not** a persistent object-store snapshot. It is a batch of reads executed within one method call, using one thread-local backend view where possible.

Documentation must say:

> A snapshot reduces inter-call drift for related UI data, but it is not an atomic transaction against concurrent repository mutation.

---

## 6. Internal design

### 6.1 Git backend

Use a single `to_thread_local()` within the backend method and call internal helper functions with that view. Some existing module functions may need refactoring to accept an explicit thread-local repo rather than creating their own.

### 6.2 Trait impact

Add a default implementation on `VcsBackend` that calls existing methods sequentially. Git backend can override for efficiency/consistency.

### 6.3 Async

Async wrapper treats snapshot as one blocking task.

---

## 7. Tests and verification

- Snapshot returns same values as individual calls on a stable fixture.
- Git override uses one thread-local view where possible.
- Default trait implementation works for custom backends.
- Documentation states non-atomic semantics.

---

## 8. Rollout plan

1. Do not start until RFC 003 default impls exist.
2. Add documentation clarifying current no-state lifecycle.
3. Add default trait method.
4. Add Git optimized implementation only if justified.

---

## 9. Risks and mitigations

**Risk: consumers think snapshot is transactional.** Use explicit naming/docs.

**Risk: API becomes a dumping ground.** Keep request fields limited to common UI needs.

**Risk: refactor cost is high.** Start with default method calling existing APIs.

---

## 10. Definition of done

- Current no-cache lifecycle remains documented.
- Optional `snapshot` API exists or is explicitly deferred.
- Default implementation exists.
- Docs clearly distinguish batch consistency from atomic snapshots.
