# RFC 001 — Handoff, archive, and release-manifest integrity

**Status.** Implemented (v0.20.0)  
**Priority.** P0  
**Target band.** v0.20.x  
**Breaking change.** No  
**Primary area.** Release hygiene / maintainer handoff

---

## 1. Summary

Make the release artifact, handoff documents, RFC directory, roadmap, changelog, and documentation tree consistent with one another.

The immediate trigger is that the v0.19.2 handoff says RFC files exist under `rfcs/`, while the inspected source archive does not appear to contain an `rfcs/` directory. The handoff also records a legacy `docs/README.md` cleanup item. These issues do not affect runtime behavior, but they directly affect maintainability and takeover safety.

This RFC introduces a small release-manifest discipline and a package verification check so future handoffs do not overclaim archive contents.

---

## 2. Motivation

`endringer` is intended for downstream consumers that may take a deep dependency on it. Those consumers need to trust not only the code but also the project record:

- what version they received;
- which RFCs define design intent;
- which docs are current;
- which files are legacy;
- whether the release archive contains everything referenced by the handoff.

A maintainer handoff that references absent RFCs causes uncertainty: is the source archive incomplete, is the handoff stale, or has the RFC process not been adopted yet?

Fixing this early is cheaper than fixing it after more RFCs accumulate.

---

## 3. Goals

- Ensure the release archive contains every planning file referenced by the handoff.
- Add a machine-readable or easy-to-audit release manifest.
- Remove or explicitly archive stale single-file documentation superseded by mdBook docs.
- Document the expected artifact layout.
- Add a CI or local check that fails before packaging if required planning files are missing.

---

## 4. Non-goals

- No source API changes.
- No changes to `gix` usage.
- No changes to runtime behavior.
- No new feature implementation.
- No v1.0 planning.

---

## 5. External design

### 5.1 Release manifest

Add a file at the repository root:

```text
RELEASE-MANIFEST.md
```

The manifest is human-readable Markdown and records the expected release contents.

Minimum sections:

```markdown
# endringer release manifest

**Version.** vX.Y.Z
**Archive shape.** The release archive extracts into a single `endringer-X.Y.Z/` directory.

## Required root files

- Cargo.toml
- Cargo.lock
- README.md
- CHANGELOG.md
- ROADMAP.md
- LICENSE
- NOTICE
- TERMS_OF_USE.md

## Required directories

- crates/
- docs/src/
- rfcs/

## Excluded files

- target/
- .git/
- editor backup files
- temporary test repositories

## Verification command

```sh
cargo xtask verify-release-manifest --version X.Y.Z
```
```

If the project does not want to introduce `xtask` yet, the verification command may initially be a shell script:

```text
scripts/verify-release-manifest.sh
```

The interface should be stable enough that maintainers can run it during release.

### 5.2 RFC directory baseline

Add or restore:

```text
rfcs/
  000-rfc-lifecycle-policy.md
  proposed/
  done/
  archive/
```

If RFC 001 already exists in a private/local tree, include it. If not, the handoff must be corrected and a fresh RFC 001 should be created under the actual project tree.

### 5.2.1 RFC numbering reconciliation (must resolve before adoption)

There is a **numbering collision** between the v0.19.2 handoff and this
planning package, and it must be resolved explicitly because the RFC
lifecycle policy (RFC 000) states that numbers are assigned once and
**never reused or renumbered**:

- The handoff documents (development-instructions §3.7, future-plans §4.1)
  repeatedly reference *"RFC 001 (ahead/behind), currently in `proposed/`"*.
- This planning package assigns **RFC 001 = handoff/archive integrity** and
  **RFC 004 = ahead/behind**.

Resolution (recommended): the inspected v0.19.2 archive contains **no
`rfcs/` directory at all**, so no RFC number was ever actually committed to
the canonical tree. The handoff's "RFC 001 = ahead/behind" is therefore a
*pre-adoption aspirational reference*, not a committed assignment. The
lifecycle policy's "never renumber" rule protects numbers that already have
external references in a shipped tree; it does not bind references that only
exist in not-yet-adopted handoff prose.

Therefore this package's numbering (001 = integrity … 004 = ahead/behind) is
adopted as the **canonical first assignment**, and the acceptance criteria
below require correcting every stale handoff reference so that ahead/behind
is unambiguously RFC 004. Once this package's `rfcs/` tree is committed,
these numbers become permanent under RFC 000.

(Alternative, rejected: preserve `001 = ahead/behind` and renumber this
package's 30 files starting at 002. Rejected because it churns 30 files and
their dense cross-references for the sake of prose that the same release is
already correcting.)

### 5.3 Documentation cleanup

Remove current `docs/README.md` if it is superseded by `docs/src/`.

If historical retention is desired, move it to:

```text
docs/legacy/README-pre-mdbook.md
```

and add a banner at the top:

```markdown
> Legacy document. This file is not part of the current public documentation.
> Current docs live under `docs/src/`.
```

Preferred option: delete it from the active release artifact.

---

## 6. Internal design

### 6.1 Verification script behavior

The check should validate:

1. root files exist;
2. required directories exist;
3. `rfcs/000-rfc-lifecycle-policy.md` exists;
4. every RFC the handoff references resolves to an existing file under
   `rfcs/` at its **canonical** number — in particular ahead/behind is
   `rfcs/proposed/004-ahead-behind-graph-computation.md`, not a legacy
   `001-ahead-behind.md` (see §5.2.1);
5. no `target/` directory is included in the archive;
6. no `.git/` directory is included;
7. archive root name matches version;
8. the `ROADMAP.md` release-history table has no duplicate version rows
   (the v0.19.2 tree currently lists `v0.15.0` twice — this check catches
   that class of defect).

Example pseudo-code:

```rust
struct ManifestRule {
    path: &'static str,
    kind: EntryKind,
    required: bool,
}

enum EntryKind {
    File,
    Directory,
}

fn verify_release_tree(root: &Path, version: &str) -> Result<()> {
    require_file(root.join("Cargo.toml"))?;
    require_file(root.join("CHANGELOG.md"))?;
    require_file(root.join("ROADMAP.md"))?;
    require_dir(root.join("crates"))?;
    require_dir(root.join("docs/src"))?;
    require_dir(root.join("rfcs"))?;
    reject_path(root.join("target"))?;
    reject_path(root.join(".git"))?;
    verify_version_in_cargo(root, version)?;
    Ok(())
}
```

A shell-script first version is acceptable:

```sh
#!/usr/bin/env sh
set -eu
root="${1:-.}"
test -f "$root/Cargo.toml"
test -f "$root/CHANGELOG.md"
test -f "$root/ROADMAP.md"
test -d "$root/crates"
test -d "$root/docs/src"
test -d "$root/rfcs"
test ! -d "$root/target"
test ! -d "$root/.git"
```

### 6.2 Release process update

Update the release process:

1. bump version;
2. update `CHANGELOG.md`;
3. update `ROADMAP.md` release table;
4. run tests;
5. run release-manifest verification against the working tree;
6. build archive;
7. unpack archive to a temp directory;
8. run release-manifest verification against the unpacked archive;
9. tag and publish crates.

---

## 7. Test plan

- Add a positive test fixture representing a valid release tree.
- Add negative test fixtures or script checks for:
  - missing `rfcs/`;
  - missing `CHANGELOG.md`;
  - included `target/`;
  - wrong version in `Cargo.toml`;
  - stale `docs/README.md` if the chosen policy is deletion.

If implemented as a shell script, CI can run it directly after packaging.

---

## 8. Compatibility

No public API changes.

Downstream users are only affected by improved artifact contents.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Maintainers view this as bureaucracy | Keep the first implementation small and script-based. |
| Archive layout policy changes later | Make the manifest explicit and update it with the release process. |
| Historical docs are accidentally removed | Move to `docs/legacy/` for one release if deletion feels risky. |

---

## 10. Acceptance criteria

- `rfcs/` exists in the source tree and release archive.
- Handoff docs no longer reference missing files.
- The RFC numbering collision in §5.2.1 is resolved: the handoff and all
  docs refer to ahead/behind as **RFC 004** and to no legacy
  `001-ahead-behind.md`.
- `docs/README.md` is removed or clearly marked as legacy outside the active docs path.
- A release manifest exists.
- A verification command catches a missing `rfcs/` directory before release.
