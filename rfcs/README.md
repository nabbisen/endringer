# endringer — RFC index

This directory contains design RFCs for `endringer`. The lifecycle policy is
defined in [`000-rfc-lifecycle-policy.md`](./000-rfc-lifecycle-policy.md).

**Folder is the source of truth for state.** An RFC's location determines its
state; the `Status` field inside the file must match.

---

## Proposed

Open for review and discussion. Implementation may not yet have begun.

| ID | Title | Priority | Target band |
|----|-------|----------|-------------|
| [010](./proposed/010-point-in-time-reads-tree-snapshots.md) | Point-in-time reads and tree snapshots | P2 | v0.26.x |
| [011](./proposed/011-remote-reference-inventory.md) | Remote and reference inventory | P3 | v0.27.x+ |
| [013](./proposed/013-rich-status-model.md) | Rich status model | P3 | v0.27.x+ |
| [014](./proposed/014-path-platform-robustness-matrix.md) | Path and platform robustness matrix | P2 | v0.27.x+ |
| [015](./proposed/015-git-cli-parity-test-harness.md) | Git CLI parity test harness | P2 | v0.27.x+ |
| [016](./proposed/016-crate-feature-dependency-policy.md) | Crate, feature, and dependency policy | P2 | v0.28.x |
| [017](./proposed/017-performance-benchmarking-large-repo-profiling.md) | Performance benchmarking and large-repository profiling | P2 | v0.28.x |
| [018](./proposed/018-async-api-operational-semantics.md) | Async API operational semantics and cancellation expectations | P2 | v0.28.x |
| [023](./proposed/023-object-format-sha256-validation.md) | Object format and SHA-256 repository validation | P2 | v0.28.x |
| [025](./proposed/025-security-resource-hardening.md) | Security and resource-exhaustion hardening | P2 | v0.29.x |
| [026](./proposed/026-custom-backend-conformance-kit.md) | Custom backend conformance kit | P3 | v0.30.x |
| [027](./proposed/027-snapshot-consistency-batch-reads.md) | Snapshot consistency and batch read APIs | P3 | v0.30.x+ |
| [028](./proposed/028-rename-copy-detection.md) | Rename and copy detection | P3 | v0.30.x+ |

---

## Implemented

RFCs whose work has shipped. Moved here from `proposed/`.

| ID | Title | Shipped in |
|----|-------|------------|
| [001](./done/001-handoff-archive-release-integrity.md) | Handoff, archive, and release-manifest integrity | v0.20.0 |
| [002](./done/002-public-contract-consistency.md) | Public contract consistency and documentation tests | v0.20.0 |
| [003](./done/003-vcsbackend-default-impls-extension-stance.md) | `VcsBackend` default implementations and extension stance | v0.21.0 |
| [031](./done/031-object-identity-foundation.md) | Object identity foundation (`ObjectId`) | v0.21.0 |
| [004](./done/004-ahead-behind-graph-computation.md) | Ahead/behind graph computation | v0.21.0 |
| [005](./done/005-branch-tracking-sync-state.md) | Branch tracking and sync state | v0.22.0 |
| [009](./done/009-repository-info-capabilities.md) | Repository information and capability discovery | v0.22.0 |
| [006](./done/006-typed-public-error-model.md) | Typed public error model | v0.23.0 |
| [007](./done/007-jj-real-repository-verification.md) | jj real-repository verification | v0.24.0 |
| [008](./done/008-read-side-operation-conflict-state.md) | Read-side operation and conflict state | v0.25.0 |
| [010](./done/010-point-in-time-reads-tree-snapshots.md) | Point-in-time reads and tree snapshots | v0.26.0 |
| [011](./done/011-remote-reference-inventory.md) | Remote and reference inventory | v0.27.0 |
| [022](./done/022-tag-api-refinement.md) | Tag API refinement and annotation completeness | v0.28.0 |
| [030](./done/030-release-quality-gates.md) | Release quality gates and stabilization dashboard | v0.28.0 |
| [012](./done/012-bounded-history-queries.md) | Bounded history queries | v0.29.0 |
| [024](./done/024-empty-bare-detached-semantics.md) | Empty, bare, detached, and unusual repository semantics | v0.29.0 |
| [019](./done/019-submodule-read-model.md) | Submodule read model and status summary | v0.30.0 |
| [020](./done/020-stash-detail-and-diff-reads.md) | Stash detail and diff reads | v0.30.0 |
| [021](./done/021-worktree-detail-and-safety.md) | Linked worktree detail and safety metadata | v0.30.0 |
| [029](./done/029-documentation-cookbook-examples.md) | Documentation cookbook and consumer examples | v0.30.0 |

---

## Archive

Withdrawn or superseded RFCs.

*(none yet)*