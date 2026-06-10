# RFC 029 — Documentation cookbook and consumer examples

**Status.** Implemented (v0.30.0)  
**Priority.** P2  
**Target band.** v0.28.x  
**Breaking change.** No  
**Primary area.** Documentation / adoption

---

## 1. Summary

Add a practical cookbook of consumer workflows: status widget, branch table, commit browser, tag management, jj read path, async multi-repo scan, and write-then-read boundary pattern.

---

## 2. Motivation

The handoff describes canonical app developer workflows, but these should be converted into maintained mdBook pages and compile-checked examples. endringer’s value is not just its raw API; it is the clear boundary it gives application developers.

A cookbook helps prevent consumers from building the wrong things into endringer or misunderstanding what the library owns.

---

## 3. Goals

- Add concise cookbook pages for common workflows.
- Include compile-checked examples where feasible.
- Demonstrate the read-first boundary repeatedly.
- Show how consumers should shell out for writes and then re-read.
- Show async semaphore pattern from RFC 018.

---

## 4. Non-goals

- Do not turn docs into a full Git tutorial.
- Do not include UI framework-specific code.
- Do not include network/write implementation recipes beyond boundary examples.
- Do not require doctests if the rustdoc/cargo mismatch remains unresolved; use examples/tests instead.

---

## 5. External design

### 5.1 Cookbook pages

Add pages under `docs/src/cookbook/`:

1. `status-widget.md`
2. `branch-table.md`
3. `commit-history-browser.md`
4. `tag-management.md`
5. `jj-repositories.md`
6. `async-multi-repo-scan.md`
7. `write-then-read-boundary.md`
8. `custom-backend.md`

### 5.2 Example style

Each page should include:

- when to use this pattern;
- API calls involved;
- minimal Rust example;
- cost notes;
- boundary notes;
- failure handling notes after typed errors land.

### 5.3 Compile checking

Because doctests may be unreliable in the current environment, examples can be placed in `examples/` or integration tests that are compiled by CI.

---

## 6. Internal design

### 6.1 Docs structure

Update `docs/src/SUMMARY.md` with a cookbook section. Remove or replace the legacy `docs/README.md` per RFC 001.

### 6.2 Example maintenance

For each cookbook page, provide a corresponding `examples/*.rs` or integration compile test when practical.

### 6.3 Error examples

After RFC 006, update cookbook examples to match on typed errors where useful.

---

## 7. Tests and verification

- `cargo test --workspace --lib --tests` remains green.
- Example code compiles through examples or integration tests.
- Cookbook does not depend on external git/jj binaries at runtime except fixture setup in tests.
- Docs mention that scheduling/persistence/UI are consumer-owned.

---

## 8. Rollout plan

1. Add cookbook structure.
2. Port handoff workflow descriptions into docs.
3. Add compile-checked examples incrementally.
4. Update examples as APIs from RFC 004–018 land.

---

## 9. Risks and mitigations

**Risk: docs drift from code.** Compile-check examples and add public contract consistency checks from RFC 002.

**Risk: examples overstep boundary.** Each page should include a boundary note.

**Risk: too much doc volume.** Keep examples short and practical.

---

## 10. Definition of done

- Cookbook pages exist.
- Core examples compile.
- Boundary rule is visible in docs.
- Handoff workflow content is no longer the only place where these patterns live.
