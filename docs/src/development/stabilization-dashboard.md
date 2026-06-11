# Stabilization dashboard

Tracks the evidence required before a v1.0 discussion can start.
Updated as RFCs land. See [release-gates.md](./release-gates.md) for definitions.

v1.0 is **not planned**.

---

## Gate status

| Gate item | Status | RFC | Evidence / notes |
|---|---|---|---|
| Public contract consistency checks | ✅ Done | RFC 002 | `scripts/check-public-contract.sh` passes on every release |
| `VcsBackend` extension stance | ✅ Done | RFC 003 | Default-impl policy documented; trait not yet stable pre-v1 |
| Typed error model | ✅ Done | RFC 006 | `endringer::Error` enum; all public methods return `Result<T>` |
| jj support verified | ✅ Done | RFC 007 | `JjFixture`; git-store-view stance; support boundary documented |
| Path/platform robustness matrix | ✅ Done | RFC 014 | `git_platform.rs`; platform-matrix.md; 8 tests |
| Git CLI parity harness | ✅ Done | RFC 015 | `git_cli.rs`, `git_cli_parity.rs`; 6 parity tests; known-deviations.md |
| Performance baseline | ✅ Done | RFC 017 | Criterion benchmarks; performance.md; classification table |
| No stale docs contradictions | ✅ Done | — | v0.33.0 docs audit: all pages verified against codebase |
| Maintainer v1 approval | 🔲 Open | — | Explicit gate; not triggered by code alone |

---

## Read surface completeness

The library's read surface is considered feature-complete for 0.x. All
originally scoped APIs are implemented and tested:

| Area | Version | RFC |
|---|---|---|
| Ahead/behind computation | v0.21.0 | RFC 004 |
| Branch tracking | v0.22.0 | RFC 005 |
| Repository info and capabilities | v0.22.0 | RFC 009 |
| Typed error model | v0.23.0 | RFC 006 |
| jj real-repository tests | v0.24.0 | RFC 007 |
| Operation and conflict state | v0.25.0 | RFC 008 |
| Point-in-time reads and tree snapshots | v0.26.0 | RFC 010 |
| Remote and reference inventory | v0.27.0 | RFC 011 |
| Bounded history queries | v0.29.0 | RFC 012 |
| Unusual repository semantics | v0.29.0 | RFC 024 |

---

## Open advancement themes

Five strategic directions remain under consideration for 0.x advancement.
None are required for the current read-surface feature set; all are
prerequisites or parallel tracks toward eventual v1 readiness.

See `rfcs/proposed/` for the full list (012–029).

---

*Last updated: v0.33.0*
