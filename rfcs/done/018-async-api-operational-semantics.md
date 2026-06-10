# RFC 018 — Async API operational semantics and cancellation expectations

**Status.** Implemented (v0.31.0)  
**Priority.** P2  
**Target band.** v0.28.x  
**Breaking change.** No  
**Primary area.** Async API / documentation / test coverage

---

## 1. Summary

Clarify and test the operational semantics of `endringer-async`, especially its `spawn_blocking` behavior, cancellation expectations, error mapping, and API parity with the sync façade.

This RFC keeps async optional and separate. It does not introduce a native async backend.

---

## 2. Motivation

The handoff states that async users opt in via a separate crate and that `AsyncRepository` mirrors every method through `tokio::task::spawn_blocking`. This is a practical design because repository reads are filesystem/blocking operations.

However, consumers need to understand exactly what they get:

- cancelling an async task may not stop an already-running blocking operation;
- many concurrent calls may consume tokio blocking threads;
- async errors should preserve the same public error model as sync errors;
- sync and async APIs should remain in parity.

---

## 3. Goals

- Document `endringer-async` as a blocking-work wrapper, not a non-blocking filesystem API.
- Define cancellation expectations.
- Ensure every sync method has an async mirror or an explicit reason not to.
- Map `JoinError` into the typed error model from RFC 006.
- Provide examples with consumer-owned semaphores for multi-repo scans.

---

## 4. Non-goals

- Do not introduce a custom runtime.
- Do not hide scheduling policy inside endringer.
- Do not add internal global concurrency limits.
- Do not promise cancellation of already-running filesystem work.

---

## 5. External design

### 5.1 Documentation contract

Add an async page to mdBook:

```text
AsyncRepository is a convenience wrapper around blocking repository reads.
It is suitable for async applications, but it does not make filesystem and object-store access cancellable once the blocking task has started.
```

### 5.2 Example pattern

Show the recommended consumer-owned concurrency limiter:

```rust
let limiter = std::sync::Arc::new(tokio::sync::Semaphore::new(8));

for repo in repos {
    let permit = limiter.clone().acquire_owned().await.unwrap();
    tokio::spawn(async move {
        let _permit = permit;
        let digest = repo.status_digest().await?;
        Ok::<_, endringer::Error>(digest)
    });
}
```

### 5.3 API parity checklist

Add a maintained checklist table in `endringer-async` docs:

| Sync method | Async method | Status |
|---|---|---|
| `status_digest` | `status_digest` | mirrored |
| `worktree_status` | `worktree_status` | mirrored |
| ... | ... | ... |

### 5.4 Cancellation statement

If an async future is dropped before the blocking task starts, tokio may avoid running it. If it has started, the underlying read usually continues until completion. This is tokio behavior and should be documented honestly.

---

## 6. Internal design

### 6.1 Macro/helper for parity

Consider a local macro or checklist-driven test to reduce drift between sync and async APIs. Avoid clever code generation if it makes rustdoc worse.

### 6.2 Error mapping

After RFC 006:

- sync backend errors map to `Error`;
- tokio join failures map to `Error::TaskJoin { message }`;
- panics in blocking tasks should not be normalized as normal repository errors.

### 6.3 No shared mutable state

`AsyncRepository` should remain a cloneable wrapper around the same repository handle model. It should not introduce caches or mutex-based serialization.

---

## 7. Tests and verification

- Unit or integration tests confirm async mirrors return the same values as sync on the same fixture.
- Compile tests/examples confirm sync users do not need tokio.
- Error mapping tests cover a synthetic join failure where practical.
- Documentation includes cancellation semantics.

---

## 8. Rollout plan

1. Add docs and examples first.
2. Add parity tests.
3. Integrate typed error mapping after RFC 006 lands.
4. Require future RFCs adding sync methods to update async parity checklist.

---

## 9. Risks and mitigations

**Risk: consumers assume cancellation is stronger than it is.** Mitigate through explicit docs.

**Risk: parity drift.** Mitigate with checklist and tests.

**Risk: accidental scheduling policy.** Keep semaphores and prioritization in examples, not library internals.

---

## 10. Definition of done

- Async semantics are documented.
- Sync/async parity checklist exists.
- Async examples demonstrate consumer-owned concurrency limits.
- Join errors have a typed mapping after RFC 006.
