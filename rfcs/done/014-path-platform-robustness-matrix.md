# RFC 014 — Path and platform robustness matrix

**Status.** Implemented (v0.32.0)  
**Priority.** P2  
**Target band.** v0.27.x+; selected tests may land earlier  
**Breaking change.** No  
**Primary area.** Cross-platform testing

---

## 1. Summary

Define and implement a path/platform test matrix for `endringer`.

This RFC does not add a user-facing feature. It increases confidence that repository reads behave correctly across Linux, macOS, Windows, unusual paths, symlinks, executable bits, bare repositories, worktrees, submodules, and nested repositories.

---

## 2. Motivation

A VCS library is exposed to repositories created by many tools on many filesystems. Simple ASCII paths and normal worktrees are not enough.

Because `endringer` returns `PathBuf` and owned values, path correctness is part of the API contract.

---

## 3. Goals

- Create an explicit test matrix.
- Add tests for high-risk path/platform behavior.
- Document unsupported cases honestly.
- Avoid converting paths through UTF-8 unless required.

---

## 4. Non-goals

- No UI path formatting.
- No path normalization policy for consumers.
- No repository mutation outside test setup.
- No support guarantee for filesystems the CI cannot exercise.

---

## 5. External design

Add documentation:

```text
docs/src/development/platform-matrix.md
```

Example table:

| Case | Linux | macOS | Windows | Expected behavior |
|---|---:|---:|---:|---|
| spaces in paths | yes | yes | yes | preserved in `PathBuf` |
| Unicode paths | yes | yes | yes | preserved |
| non-UTF-8 paths | yes | n/a/limited | n/a | no panic; return `PathBuf` |
| symlink | yes | yes | limited | tree/status classify or document |
| executable bit | yes | yes | no | mode changes tested where supported |
| case-only rename | fs-dependent | fs-dependent | fs-dependent | documented |
| bare repository | yes | yes | yes | worktree reads empty/unsupported per contract |
| linked worktree | yes | yes | yes | linked worktrees listed |
| submodule | yes | yes | yes | submodule metadata listed |
| nested git repo | yes | yes | yes | behavior documented |

---

## 6. Internal design

### 6.1 Test helper conventions

Extend fixture support with platform-aware helpers:

```rust
pub fn write_file_bytes(path: &Path, bytes: &[u8]);
pub fn create_symlink(target: &Path, link: &Path) -> TestOutcome;
pub fn set_executable(path: &Path) -> TestOutcome;
pub fn supports_non_utf8_paths() -> bool;
```

Use `#[cfg(unix)]` for non-UTF-8 and executable-bit tests.

### 6.2 Non-UTF-8 paths

On Unix:

```rust
use std::os::unix::ffi::OsStringExt;
let name = OsString::from_vec(vec![0xff, b'.', b't', b'x', b't']);
```

Ensure APIs return `PathBuf` and do not panic. If a backend operation cannot handle the path, return a typed `NonUtf8Path` after RFC 006.

### 6.3 Windows separators

Tests should assert logical path components, not hardcoded `/`, unless the API explicitly promises forward-slashed paths. The handoff says paths are repo-root-relative, forward-slashed for some APIs; reconcile this contract in RFC 002 before platform assertions.

### 6.4 Case sensitivity

Do not pretend case behavior is uniform. Tests should detect filesystem behavior or avoid making universal assertions.

---

## 7. Test plan

Add integration tests for:

- path with spaces;
- Unicode path;
- non-UTF-8 path on Unix;
- symlink file;
- executable-bit modification;
- bare repo behavior for status/diff/history;
- linked worktree path handling;
- submodule path handling;
- nested repository ignored/tracked behavior.

Run at least a subset on Linux, macOS, and Windows CI.

---

## 8. Compatibility

No public API changes.

Tests may reveal bugs that require behavior fixes. Those fixes should be documented in changelog.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Platform tests are flaky | Gate filesystem-specific expectations behind capability detection. |
| Non-UTF-8 paths are hard on Windows | Restrict non-UTF-8 tests to Unix. |
| CI cost increases | Add matrix gradually and keep fixtures small. |

---

## 10. Acceptance criteria

- Platform matrix doc exists.
- Linux CI covers non-UTF-8 paths.
- Windows CI covers separator/path behavior.
- macOS CI covers symlink/executable behavior where possible.
- No path test relies on consumer-side string parsing unless that is the documented contract.
