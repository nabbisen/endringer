# RFC 016 — Crate, feature, and dependency policy

**Status.** Implemented (v0.31.0)  
**Priority.** P2  
**Target band.** v0.28.x  
**Breaking change.** No, unless public dependency expectations are changed  
**Primary area.** Packaging / dependency control

---

## 1. Summary

Define a written policy for how `endringer` uses its five-crate workspace, optional dependencies, feature flags, and public dependency exposure.

The project currently uses crate separation rather than in-crate feature flags as the main dependency-control mechanism. This RFC keeps that stance, but makes it explicit and reviewable so future growth does not accidentally make the façade crate heavy or pull async/test-only dependencies into sync consumers.

---

## 2. Motivation

A VCS library is often embedded into GUI tools, status bars, release helpers, CI helpers, and code-review services. These consumers care about compile time, dependency surface, binary size, and runtime assumptions.

The handoff states that async is optional and separate, that sync users should pay nothing for tokio, and that `gix` must remain internal. As new APIs are added, the project needs a repeatable policy for deciding:

- when a new crate is justified;
- when a feature flag is justified;
- which crates may depend on `tokio`;
- whether test-only CLI tooling may be present;
- whether any public API type can contain a third-party type.

---

## 3. Goals

- Preserve the current five-crate architecture unless a real reason appears.
- Keep `endringer-core` free of backend-heavy dependencies.
- Keep `endringer` as the ergonomic façade.
- Keep `endringer-async` as the only tokio-dependent public crate.
- Document when feature flags are allowed.
- Document that `gix` and jj implementation dependencies remain private.
- Define dependency-review criteria for future RFCs.

---

## 4. Non-goals

- Do not collapse the workspace into one crate.
- Do not introduce feature flags merely because they are fashionable.
- Do not make `git` or `jj` runtime dependencies.
- Do not expose `gix`, `jj`, or CLI-specific types in public structs.
- Do not solve compile-time optimization with premature micro-crates.

---

## 5. External design

### 5.1 Policy document

Add `docs/src/development/dependency-policy.md` and reference it from the README and mdBook.

The document should state:

| Crate | Responsibility | Dependency policy |
|---|---|---|
| `endringer-core` | Public types, trait, error model | No backend implementation dependencies; no tokio |
| `endringer-git` | Git backend | May depend on `gix`; no public `gix` exposure |
| `endringer-jj` | jj backend | May reuse `endringer-git`; no runtime `jj` requirement |
| `endringer` | Facade and re-exports | Depends on core/git/jj; no tokio |
| `endringer-async` | Async wrapper | May depend on tokio; mirrors sync API |

### 5.2 Feature flag rules

Feature flags are allowed only when one of the following is true:

1. a dependency is large and not needed by all users;
2. a backend is optional in a way that materially affects compile time;
3. an unstable extension must be guarded before stabilization;
4. test or fixture support must not affect production builds.

Feature flags must not change semantic contracts silently. If a method is unavailable due to a feature, this must be reflected by crate selection or a typed `UnsupportedBackendFeature` error, not by surprising runtime behavior.

### 5.3 Public dependency rule

Public types must not contain third-party implementation types unless an RFC explicitly approves it. This preserves the current rule that `gix` remains internal.

### 5.4 RFC template addition

Update the RFC lifecycle/template so every RFC has a short dependency-impact section:

```markdown
## Dependency impact

- New public dependencies: none
- New private dependencies: ...
- Feature flags: ...
- Async impact: none / mirrors sync method
```

---

## 6. Internal design

No major code changes are required. The implementation is documentation and review-process work.

If desired, add a small CI script that checks forbidden dependencies in `endringer-core` and `endringer`:

- `endringer-core` must not depend on `gix` or `tokio`;
- `endringer` must not depend on `tokio`;
- `endringer-async` may depend on `tokio`.

A simple `cargo metadata`-based `xtask` can enforce this later.

---

## 7. Tests and verification

- Add a documentation test or CI check that `endringer-core` has no `gix` dependency.
- Add a CI check that `endringer` has no `tokio` dependency.
- Review all current Cargo manifests and document intentional dependencies.
- Confirm that examples using the sync façade compile without enabling async crates.

---

## 8. Rollout plan

1. Land the policy document.
2. Add the RFC-template dependency-impact section.
3. Add optional CI dependency checks.
4. Revisit only if a future RFC proposes major feature flags or crate splits.

---

## 9. Risks and mitigations

**Risk: policy becomes bureaucracy.** Keep the document short and practical.

**Risk: feature flags fragment behavior.** Prefer crate separation and typed unsupported errors over semantic feature flags.

**Risk: compile time remains high because `gix` is inherently large.** This RFC cannot remove that cost, but it prevents accidental new costs.

---

## 10. Definition of done

- Dependency policy exists in mdBook.
- RFC template includes dependency-impact review.
- Current crate dependency responsibilities are documented.
- Optional dependency CI check exists or is explicitly deferred.
