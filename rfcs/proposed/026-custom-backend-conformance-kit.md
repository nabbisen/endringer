# RFC 026 — Custom backend conformance kit

**Status.** Proposed  
**Priority.** P3  
**Target band.** v0.30.x  
**Breaking change.** No  
**Primary area.** Backend extension / testing

---

## 1. Summary

Provide a conformance test kit for custom `VcsBackend` implementors so they can validate behavior against endringer’s public contracts without copying internal tests.

---

## 2. Motivation

The handoff allows consumers to implement custom backends, but also says the trait is not yet stabilized. If endringer keeps `VcsBackend` as a public extension point, custom backend authors need a way to test their implementations.

Default implementations reduce breakage, but conformance tests reduce semantic drift.

---

## 3. Goals

- Provide reusable tests for backend implementors.
- Define behavioral contracts for each trait method.
- Allow custom backends to skip unsupported optional features explicitly.
- Keep internal gix fixture details separate from public conformance expectations.

---

## 4. Non-goals

- Do not freeze `VcsBackend` prematurely.
- Do not require every custom backend to support every method if the extension stance allows optional features.
- Do not expose private git backend internals.

---

## 5. External design

### 5.1 New dev-support module or crate

Options:

- `endringer-testkit` crate;
- `endringer-core::testkit` behind `#[cfg(any(test, feature = "testkit"))]`;
- documentation-only checklist first.

Recommended: start with a separate `endringer-testkit` crate only when there is real demand. Before that, add documentation and internal helpers.

### 5.2 Conformance trait

A test kit could accept a factory:

```rust
pub trait BackendFixtureFactory {
    type Backend: VcsBackend;

    fn clean_repo(&self) -> Repository;
    fn dirty_repo(&self) -> Repository;
    fn repo_with_branches(&self) -> Repository;
    fn repo_with_tags(&self) -> Repository;
}
```

### 5.3 Contract groups

Tests should be grouped so unsupported areas can be skipped:

- identity and commit lookup;
- status;
- branches;
- tags;
- graph;
- diff/content;
- metadata;
- optional worktree/stash/submodule areas.

---

## 6. Internal design

### 6.1 Internal extraction

Start by extracting shared expectations from current integration tests into named helper functions. Do not make them public until the trait stability stance is decided.

### 6.2 Optional feature behavior

After RFC 006, unsupported optional methods should return `UnsupportedBackendFeature`, allowing conformance tests to distinguish unsupported from incorrect.

### 6.3 Documentation

Add a page:

- how to implement `VcsBackend`;
- which methods are essential;
- which methods may use default impls;
- how to test a custom backend.

---

## 7. Tests and verification

- Internal git backend passes all conformance groups.
- jj backend passes groups appropriate to its verified stance.
- A toy in-memory backend can pass a minimal subset if useful.
- Default impl behavior is tested.

---

## 8. Rollout plan

1. Decide extension stance in RFC 003.
2. Document backend contracts.
3. Extract internal conformance helpers.
4. Publish a testkit crate only if maintainers want a supported extension ecosystem.

---

## 9. Risks and mitigations

**Risk: testkit freezes the trait accidentally.** Do not publish as stable until the extension stance is clear.

**Risk: too much maintenance overhead.** Start internal and documentation-first.

**Risk: custom backends rely on weak defaults.** Conformance tests should make unsupported behavior explicit.

---

## 10. Definition of done

- Backend contract documentation exists.
- Internal conformance helpers exist.
- Git backend passes all relevant groups.
- Public testkit decision is recorded.
