# RFC 030 — Release quality gates and stabilization dashboard

**Status.** Implemented (v0.28.0)  
**Priority.** P1  
**Target band.** v0.28.x and ongoing  
**Breaking change.** No  
**Primary area.** Release management / quality assurance

---

## 1. Summary

Define explicit quality gates for post-0.19.2 releases and a stabilization dashboard that shows why v1.0 is not yet planned and what evidence would eventually make it discussable.

---

## 2. Motivation

The user has clarified that v1 is not planned yet and that more stabilization and testing are required. The project therefore needs a roadmap mechanism that prevents accidental v1 pressure while still showing progress.

This RFC turns stabilization into visible gates rather than vague caution.

---

## 3. Goals

- Add release gates for normal 0.x releases.
- Add stricter “v1 discussion gates” without scheduling v1.
- Track documentation consistency, API stability, typed errors, jj verification, platform coverage, and performance baselines.
- Make quality status visible in ROADMAP or a dedicated dashboard.

---

## 4. Non-goals

- Do not set a v1 date.
- Do not require perfection before every 0.x release.
- Do not block small patch releases on long-term v1 gates.
- Do not hide known gaps.

---

## 5. External design

### 5.1 Release gate levels

Define three gate levels:

| Gate | Meaning |
|---|---|
| Patch gate | Safe bugfix/documentation release |
| Minor gate | New API or behavior release |
| Stabilization gate | Evidence required before v1 discussion |

### 5.2 Patch gate

Required:

- changelog entry;
- `cargo test --workspace --lib --tests` passes;
- no known archive packaging error;
- docs updated for changed behavior.

### 5.3 Minor gate

Required in addition:

- RFC exists for new public API;
- sync/async parity considered;
- dependency impact reviewed;
- typed error behavior considered if RFC 006 has landed;
- migration note for breaking 0.x changes.

### 5.4 Stabilization discussion gate

v1 discussion remains blocked until all are true:

- public contract consistency checks exist;
- `VcsBackend` extension stance is decided;
- typed error model is implemented or explicitly rejected with rationale;
- jj support is verified or clearly marked experimental;
- path/platform matrix has meaningful coverage;
- Git CLI parity harness covers core operations;
- performance baseline exists for major reads;
- no known stale handoff/docs contradictions remain;
- maintainer explicitly approves opening v1 planning.

### 5.5 Dashboard

Add `docs/src/development/stabilization-dashboard.md` or a ROADMAP section:

```markdown
| Gate item | Status | Evidence | Blocking notes |
|---|---|---|---|
| Typed errors | Open | RFC 006 | ... |
| jj verification | Open | RFC 007 | ... |
```

---

## 6. Internal design

### 6.1 `xtask release-check`

Optional future helper:

```sh
cargo xtask release-check --level patch
cargo xtask release-check --level minor
```

Initial version may simply print a checklist. Later versions can verify:

- changelog contains version;
- ROADMAP updated;
- package archive includes `rfcs/`;
- tests pass;
- docs build if mdBook is available.

### 6.2 CI integration

Do not require full stabilization gate in CI. CI should enforce patch/minor mechanics; stabilization dashboard is a maintainer review tool.

---

## 7. Tests and verification

- Manual release checklist exists.
- Optional `xtask release-check` exists or is explicitly deferred.
- A simulated release archive includes expected docs/RFC files.
- Stabilization dashboard is updated when RFCs land.

---

## 8. Rollout plan

1. Add release-gate document.
2. Add stabilization dashboard.
3. Add archive-manifest check from RFC 001.
4. Add optional `xtask release-check` later.

---

## 9. Risks and mitigations

**Risk: gates slow down necessary bugfixes.** Separate patch gate from stabilization gate.

**Risk: dashboard becomes stale.** Make updates part of RFC completion.

**Risk: v1 pressure returns.** Keep v1 as a discussion gate requiring explicit maintainer approval.

---

## 10. Definition of done

- Release gate document exists.
- Stabilization dashboard exists.
- v1 remains explicitly unplanned.
- Maintainers have a clear checklist for 0.x releases and eventual v1 discussion readiness.
