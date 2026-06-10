# RFC 002 — Public contract consistency and documentation tests

**Status.** Proposed  
**Priority.** P0  
**Target band.** v0.20.x  
**Breaking change.** No  
**Primary area.** Documentation / test quality

---

## 1. Summary

Add a lightweight contract-consistency discipline so README, mdBook, rustdoc, handoff documents, and actual behavior describe the same public API semantics.

The immediate inconsistencies to fix are:

- rustdoc says `worktree_status().untracked` does not apply gitignore, while implementation/tests and mdBook say it does;
- rustdoc says jj `create_annotated_tag` falls back to a lightweight tag, while implementation and backend docs say it returns an error;
- roadmap and handoff method counts drift.

---

## 2. Motivation

For a library like `endringer`, documentation is part of the public contract. Consumers will make design decisions from rustdoc and mdBook, not only from tests.

Incorrect docs are especially harmful here because `endringer` is a boundary library. If a consumer believes ignored files are reported, it may implement filtering unnecessarily. If it believes jj annotated tags silently become lightweight tags, it may miss an error path.

This RFC treats docs as testable artifacts.

---

## 3. Goals

- Fix known contract drift.
- Add a small repeatable consistency check.
- Make future doc changes easier to review.
- Keep the check cheap enough to run in normal CI.

---

## 4. Non-goals

- Do not solve rustdoc/cargo doctest environment issues in this RFC.
- Do not require full mdBook rendering in every CI job.
- Do not introduce a heavy documentation linter.
- Do not change public API semantics.

---

## 5. External design

### 5.1 Contract statements

Create a file:

```text
docs/src/reference/contract.md
```

It records high-value behavioral contracts in one place.

Example entries:

```markdown
## Working-tree status

- `worktree_status().untracked` applies active git ignore rules when the backend can build the exclude stack.
- If exclude-stack construction fails, the git backend degrades to reporting untracked files without filtering.
- Bare repositories return an empty `WorktreeStatus`.

## jj tags

- `create_tag` creates lightweight tags on jj repositories.
- `create_annotated_tag` returns an unsupported-feature error on jj repositories.
- jj does not support annotated tags through the current endringer API.
```

### 5.2 Rustdoc updates

Update rustdoc on:

- `WorktreeStatus`;
- `Repository::worktree_status`;
- `Repository::create_annotated_tag`;
- `JjBackend::create_annotated_tag` module comments;
- any docs that still imply tag annotations are unavailable after v0.18.

### 5.3 Consistency check command

Introduce:

```sh
scripts/check-public-contract.sh
```

Initial checks may be simple grep-based guards:

```sh
#!/usr/bin/env sh
set -eu

# Stale claims that must not reappear.
! grep -R "gitignore rules are not applied" crates docs README.md
! grep -R "falls back to a lightweight tag" crates docs README.md
! grep -R "tag objects themselves .* not exposed" docs README.md
```

If `xtask` is adopted, move the logic there later.

### 5.4 Documentation map

Add a small table to `docs/src/development/architecture.md`:

| Public claim | Source of truth | Tests |
|---|---|---|
| gitignore-aware untracked files | `status.rs` + contract doc | `git_status.rs::gitignored_file_not_in_untracked` |
| jj annotated tag error | `endringer-jj` backend | jj backend tests |
| sorted diff paths | `diff.rs` + type docs | diff integration tests |
| no public `gix` types | public API review | compile/API check |

---

## 6. Internal design

No runtime internal design changes are required.

The only optional code change is to add a small `cargo xtask public-contract` command later.

Pseudo-code for an xtask variant:

```rust
fn check_public_contract(repo: &Path) -> Result<()> {
    forbid(repo, "gitignore rules are not applied")?;
    forbid(repo, "falls back to a lightweight tag")?;
    forbid(repo, "tag objects themselves")?;
    require(repo.join("docs/src/reference/contract.md"))?;
    Ok(())
}
```

---

## 7. Test plan

- Run the script in CI.
- Add one test that intentionally searches for stale phrases only in source-controlled docs/code.
- Keep behavioral tests separate; this RFC only catches known stale public statements.

---

## 8. Compatibility

No public API changes.

Consumers only see corrected documentation.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Grep checks are crude | Use them only for known stale claims; do not overgeneralize. |
| Check becomes annoying during refactors | Keep the forbidden phrase list small and high-signal. |
| Docs duplicate too much detail | Use `contract.md` as the stable contract and link to it from detailed pages. |

---

## 10. Acceptance criteria

- Stale gitignore and jj annotated-tag claims are fixed.
- `docs/src/reference/contract.md` exists.
- CI or local release checks run `scripts/check-public-contract.sh` or equivalent.
- README, mdBook, rustdoc, and handoff no longer disagree on the corrected items.
