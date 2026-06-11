# Release quality gates

This document defines what must be true before shipping a release.
It has three levels: patch, minor, and the stabilization discussion gate.

v1.0 is **not planned**. The stabilization gate defines the evidence required
before that conversation can even start. A gate being open does not imply a
schedule; it records what is still missing.

---

## Patch gate

Required before any patch release (`x.y.Z`):

- CHANGELOG entry for the version.
- `cargo test --workspace --lib --tests` passes with zero failures.
- No known archive-packaging error (verify-release-manifest.sh passes).
- Documentation updated for any changed behaviour.

---

## Minor gate

Required before any minor release (`x.Y.0`), in addition to the patch gate:

- RFC exists for every new public API item.
- Sync/async parity considered (new methods added to both, or explicitly deferred
  with a documented reason).
- Dependency impact reviewed (new or bumped dependencies justified).
- Typed error behaviour considered (new errors use the RFC 006 model).
- Migration note in CHANGELOG for any breaking 0.x change.

---

## Stabilization discussion gate

v1.0 discussion is blocked until **all** of the following are true.
See the [stabilization dashboard](./stabilization-dashboard.md) for current status.

| Gate item | RFC |
|---|---|
| Public contract consistency checks exist | RFC 002 |
| `VcsBackend` extension stance is decided | RFC 003 |
| Typed error model is implemented | RFC 006 |
| jj support is verified or explicitly experimental | RFC 007 |
| Path/platform robustness matrix has meaningful coverage | RFC 014 |
| Git CLI parity harness covers core operations | RFC 015 |
| Performance baseline exists for major reads | RFC 017 |
| No known stale handoff/docs contradictions remain | — |
| Maintainer explicitly approves opening v1 planning | — |

Meeting all gate items does not commit to v1 — it means the project has
enough evidence to have the conversation.
