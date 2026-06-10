# RFC 025 — Security and resource-exhaustion hardening for local repository reads

**Status.** Implemented (v0.31.0)  
**Priority.** P2  
**Target band.** v0.29.x  
**Breaking change.** No  
**Primary area.** Robustness / security posture

---

## 1. Summary

Define a security posture for reading untrusted or semi-trusted local repositories, focusing on resource exhaustion, path robustness, symlink handling, corrupt object data, and avoidance of unexpected external execution.

---

## 2. Motivation

Although endringer is not a network service and does not execute hooks or external binaries at runtime, consumers may point it at repositories from untrusted sources. A robust library should avoid accidental external execution, avoid unbounded reads where possible, and fail safely on corrupt or malicious repository data.

This RFC complements path/platform testing and CLI parity, but its focus is security and defensive limits.

---

## 3. Goals

- State the threat model for local repository reads.
- Confirm no runtime hook/CLI execution.
- Add limits or documented cost controls where APIs can produce huge outputs.
- Avoid path traversal surprises in APIs returning paths or file bytes.
- Map corrupt data into typed errors after RFC 006.
- Add fuzz/property tests where practical.

---

## 4. Non-goals

- Do not attempt to sandbox gix.
- Do not add a permission system.
- Do not make endringer responsible for scanning malware in repository files.
- Do not add network credential handling.

---

## 5. External design

### 5.1 Security posture document

Add `docs/src/security.md`:

- runtime does not call `git` or `jj` binaries;
- tests may call CLI fixtures only;
- hooks are not executed by endringer;
- repository contents may be malicious or huge;
- callers should apply their own timeouts/concurrency limits at orchestration level;
- file contents returned by `file_at_commit` are raw bytes and may be large.

### 5.2 Resource controls

Prefer bounded APIs for potentially large outputs:

- RFC 012: bounded history queries;
- RFC 010: tree listing should support limits or documented cost;
- future large file reads may need a size-aware metadata method before byte loading.

### 5.3 Path rules

All returned paths should be repository-root-relative where applicable. Absolute paths should appear only for repository/worktree locations.

Document whether returned paths can contain `..` or platform-specific separators. Prefer normalized relative `PathBuf` values that do not escape the repository root.

### 5.4 Corrupt data handling

Malformed refs, objects, index entries, or administrative files should map to typed errors or conservative unknown states rather than panics.

---

## 6. Internal design

### 6.1 Audit points

Review code paths for:

- `unwrap`/`expect` on repository data;
- path joining with untrusted path components;
- reading whole files without size awareness;
- recursion over directories or trees;
- silent fallback behavior that may hide corruption.

### 6.2 Fuzz/property testing

Candidate fuzz targets:

- `CommitId::from_hex`;
- ref-name parsing wrappers;
- path normalization helpers;
- status entry classification from synthetic inputs where separable.

### 6.3 No external execution guarantee

Add tests or static checks only if practical. At minimum, document that runtime library code must not spawn commands.

---

## 7. Tests and verification

- Grep/audit for command spawning outside tests/fixtures.
- Tests for path normalization edge cases.
- Tests for invalid commit hex and invalid path inputs.
- Fuzz target for `CommitId::from_hex` if fuzz infrastructure is accepted.
- Corrupt fixture tests where feasible and safe.

---

## 8. Rollout plan

1. Add security posture documentation.
2. Audit runtime code for external command execution and unsafe unwraps.
3. Add path/resource tests.
4. Add optional fuzz targets.
5. Revisit resource controls after RFC 012 and RFC 010.

---

## 9. Risks and mitigations

**Risk: security posture becomes overclaiming.** Be precise: endringer reduces risk by not executing commands, but it is not a sandbox.

**Risk: hard limits break legitimate large repos.** Prefer bounded alternative APIs and documentation over arbitrary global limits.

**Risk: corrupt fixtures are hard to maintain.** Keep them minimal and targeted.

---

## 10. Definition of done

- Security posture page exists.
- Runtime no-external-command guarantee is documented.
- Path/resource audit is complete.
- Tests cover representative hardening cases.
