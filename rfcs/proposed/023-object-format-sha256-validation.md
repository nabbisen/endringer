# RFC 023 — Object format and SHA-256 repository validation

**Status.** Proposed  
**Priority.** P2  
**Target band.** v0.28.x  
**Breaking change.** No for validation; may add API  
**Primary area.** Repository format / correctness

---

## 1. Summary

Validate and document object-format behavior, especially SHA-1 versus SHA-256 repositories, and ensure `CommitId` and repository discovery handle supported formats consistently.

---

## 2. Motivation

`CommitId` already accepts 20-byte SHA-1 and 32-byte SHA-256 IDs, but the handoff notes that jj SHA-256 repositories are not covered. Git SHA-256 repository support may also depend on gix behavior and fixture coverage.

Before broad stabilization, the project should know which object formats are supported, tested, or explicitly unsupported.

---

## 3. Goals

- Add object-format discovery to repository capability data if RFC 009 lands.
- Test `CommitId::from_hex` and display behavior for both SHA-1 and SHA-256.
- Add Git SHA-256 fixture tests where the installed git/gix support allows it.
- Bound jj SHA-256 support explicitly.
- Ensure errors for unsupported formats are typed after RFC 006.

---

## 4. Non-goals

- Do not implement a custom object database.
- Do not support experimental formats beyond what gix can read.
- Do not claim SHA-256 support merely because `CommitId` can store 32 bytes.

---

## 5. External design

### 5.1 Public type

`ObjectFormat` is **owned by RFC 009** (repository info & capabilities) and
defined there as:

```rust
#[non_exhaustive]
pub enum ObjectFormat { Sha1, Sha256, Unknown(String) }
```

This RFC does **not** redefine it. (An earlier draft of this RFC declared a
conflicting `Unknown(String)` while RFC 009 declared a unit `Unknown`; the
canonical form above resolves that — `Unknown` carries the raw format name
for diagnostics, and the type is `#[non_exhaustive]`.)

Expose through repository info:

```rust
Repository::repository_info() -> Result<RepositoryInfo>
```

If RFC 009 has not landed when this validation work begins, introduce
`ObjectFormat` here with the exact definition above so the two RFCs cannot
diverge; otherwise depend on RFC 009's definition. A standalone convenience
is also acceptable:

```rust
Repository::object_format() -> Result<ObjectFormat>
```

### 5.2 Supported behavior table

Add documentation table:

| Backend | SHA-1 | SHA-256 | Notes |
|---|---|---|---|
| Git | supported/tested | supported/tested or unsupported/documented | depends on fixture result |
| jj git view | supported/tested after RFC 007 | unsupported until verified | no overclaim |

### 5.3 CommitId storage

If inline storage is accepted:

```rust
pub enum CommitIdRepr {
    Sha1([u8; 20]),
    Sha256([u8; 32]),
}
```

Keep public `CommitId` opaque.

---

## 6. Internal design

### 6.1 Fixture creation

A SHA-256 Git fixture may require:

```sh
git init --object-format=sha256
```

Tests should skip gracefully if the local git used for fixture creation does not support SHA-256.

### 6.2 gix integration

Read object format from gix repository metadata if available. If not available, inspect config only if reliable.

### 6.3 jj backend

Do not infer jj SHA-256 support from Git support. Add a separate verification case under RFC 007 or mark unsupported.

---

## 7. Tests and verification

- `CommitId::from_hex` accepts 40 and 64 hex chars, rejects others.
- Display and short forms work for both lengths.
- Git SHA-256 fixture test exists or is explicitly skipped with reason.
- jj SHA-256 behavior is documented as unsupported or verified.
- Unsupported object format maps to typed error after RFC 006.

---

## 8. Rollout plan

1. Add documentation and `ObjectFormat` type if not already present.
2. Add `CommitId` unit tests for SHA-256 edge cases.
3. Add optional Git SHA-256 integration test.
4. Add jj SHA-256 decision to RFC 007 verification matrix.

---

## 9. Risks and mitigations

**Risk: test environment cannot create SHA-256 repos.** Use skip-with-reason, not failure.

**Risk: public claim outruns actual backend support.** Separate storage capability from repository support.

**Risk: gix behavior changes.** Include object-format checks in gix upgrade procedure.

---

## 10. Definition of done

- Object-format behavior is documented.
- Git SHA-256 support is either tested or explicitly unsupported.
- jj SHA-256 stance is explicit.
- `CommitId` behavior is fully tested for both supported sizes.
