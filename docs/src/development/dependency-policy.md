# Dependency and feature policy

This document defines how endringer's five-crate workspace manages
dependencies, feature flags, and public type exposure. It is a standing
policy, not a one-time decision — future RFCs that add dependencies or
feature flags must reference it.

## Crate responsibilities

| Crate | Responsibility | Dependency rules |
|---|---|---|
| `endringer-core` | Public types, `VcsBackend` trait, error model | No backend implementation deps; no `tokio`; no `gix` |
| `endringer-git` | Git backend (`gix`-based) | May depend on `gix`; `gix` types must not appear in public API |
| `endringer-jj` | jj backend | May reuse `endringer-git`; no runtime `jj` binary requirement |
| `endringer` | Façade and re-exports | Depends on core/git/jj; no `tokio` |
| `endringer-async` | Async wrapper | Only crate that may depend on `tokio`; mirrors sync API |

## Feature flag rules

Feature flags are allowed only when one of the following is true:

1. A dependency is large and not needed by all users.
2. An optional backend materially affects compile time.
3. An unstable extension must be guarded before stabilization.
4. Test or fixture support must not affect production builds.

Feature flags must not change semantic contracts silently. If a method is
unavailable because a feature flag is absent, this must be reflected by crate
selection or a typed `UnsupportedBackendFeature` error — not by surprising
runtime behaviour.

**Current stance (pre-v1.0):** feature flags within existing crates are
deferred. Crate-level dependency control (separate crates, each addable
individually in `Cargo.toml`) is preferred over in-crate flags. This avoids
the `VcsBackend` trait fragmentation problem described in the handoff.

## Public dependency rule

Public types must not contain third-party implementation types unless an RFC
explicitly approves it. The current rule: `gix` types must never cross the
public API boundary. Downstream consumers must have zero compile-time
dependency on `gix`.

This is enforced at the architecture level:
- `CommitId` hides `gix::ObjectId` behind a newtype.
- `VcsBackend` is the only seam; backend crates are not in `endringer`'s
  public dependency tree (they are implementation details).

## When a new crate is justified

Add a new crate to the workspace when:
- a significant new backend is needed (e.g. a new VCS);
- a feature would pull a large new dependency that sync users must not pay for;
- a separation of build units gives meaningful compile-time or binary-size wins.

Do not add micro-crates for minor functionality splits. The current five-crate
layout is the right size for the current scope.

## Dependency impact in RFCs

Every RFC that adds or modifies dependencies includes a short section:

```markdown
## Dependency impact

- New public dependencies: none
- New private dependencies: …
- Feature flags: none / deferred
- Async impact: none / async mirror added
```

## Runtime binary policy

endringer never invokes `git`, `jj`, or any other external binary at
runtime. The `git` and `jj` CLIs may only appear in test fixture setup
code. This is a hard invariant, not a guideline.

If a future RFC proposes runtime CLI invocation, it requires an explicit
justification section and a named person responsible for maintaining
the integration.
