# RFC 015 — Git CLI parity test harness

**Status.** Implemented (v0.32.0)  
**Priority.** P2  
**Target band.** v0.27.x+; selected parity tests may land earlier  
**Breaking change.** No  
**Primary area.** Behavioral verification

---

## 1. Summary

Add a test-only Git CLI parity harness that compares selected `endringer` reads against stable Git CLI outputs.

This does not change the runtime promise: `git` is not required by the library at runtime. Git remains a fixture/test dependency only.

---

## 2. Motivation

`endringer` wraps `gix` and intentionally avoids spawning `git` at runtime. However, Git CLI behavior is still the practical reference for many semantics:

- status categories;
- ahead/behind counts;
- merge-base;
- branch upstream configuration;
- tag listing;
- blame line attribution.

Parity tests help distinguish implementation bugs from intentional design differences.

---

## 3. Goals

- Compare high-value read APIs to Git CLI behavior in tests.
- Keep parity tests deterministic and fixture-isolated.
- Use machine-readable Git output where possible.
- Document intentional deviations.

---

## 4. Non-goals

- No runtime use of `git`.
- No parsing of human-friendly localized output.
- No attempt to match every Git porcelain detail.
- No replacement for direct integration tests.

---

## 5. External design

No public API changes.

Add developer documentation:

```text
docs/src/development/git-cli-parity.md
```

It explains:

- why Git CLI is allowed in tests;
- which commands are used;
- which behaviors intentionally differ;
- how to update parity expectations.

---

## 6. Internal design

### 6.1 Harness module

Add:

```text
crates/endringer/tests/support/git_cli.rs
```

Helpers:

```rust
pub fn git_output(repo: &Path, args: &[&str]) -> String;
pub fn git_lines(repo: &Path, args: &[&str]) -> Vec<String>;
pub fn git_status_porcelain_v2(repo: &Path) -> Vec<PorcelainStatusEntry>;
pub fn git_rev_list_left_right_count(repo: &Path, left: &str, right: &str) -> (usize, usize);
```

Use the same environment isolation as existing fixtures:

- `GIT_CONFIG_NOSYSTEM=1`;
- `GIT_CONFIG_GLOBAL=/dev/null`;
- `GIT_EDITOR=true`;
- `GIT_TERMINAL_PROMPT=0`;
- null stdin.

### 6.2 Commands

Prefer stable, parseable commands:

| API | Git command |
|---|---|
| `worktree_status` | `git status --porcelain=v2 -z` |
| `ahead_behind` | `git rev-list --left-right --count A...B` |
| `merge_base` | `git merge-base A B` |
| `is_ancestor` | `git merge-base --is-ancestor A B` |
| branches | `git for-each-ref --format=... refs/heads refs/remotes` |
| tags | `git for-each-ref --format=... refs/tags` |
| blame | `git blame --line-porcelain` |

### 6.3 Intentional deviations file

Add:

```text
tests/parity/KNOWN-DEVIATIONS.md
```

Examples:

- simple `ChangeKind` may collapse mode/type changes;
- ignored files are not returned unless API says so;
- jj backend is not covered by Git CLI parity except its git-store view.

---

## 7. Test plan

Initial parity tests:

1. `merge_base` and `is_ancestor` on simple graph;
2. ahead/behind after RFC 004;
3. worktree status with staged/unstaged/untracked/ignored files;
4. tag listing with lightweight and annotated tags;
5. branch listing with local and remote refs;
6. blame on a simple two-commit file.

Each parity test should have an integration fixture that already tests the same feature directly. Parity is an additional confidence layer, not the only test.

---

## 8. Compatibility

No public API changes.

CI already needs Git for fixture creation. This RFC formalizes that dependency for parity checks.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Git CLI output changes | Use porcelain/plumbing formats and pin minimum Git version in CI docs. |
| Locale affects output | Use machine-readable output and avoid human text. |
| Parity tests become too broad | Start with high-value APIs only. |
| Tests make runtime promise confusing | Document clearly: test dependency only. |

---

## 10. Acceptance criteria

- `git_cli` test helper exists.
- At least three APIs have parity tests.
- Parity docs explain test-only use of Git CLI.
- Known deviations are documented.
