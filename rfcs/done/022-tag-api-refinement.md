# RFC 022 — Tag API refinement and annotation completeness

**Status.** Implemented (v0.28.0)  
**Priority.** P2  
**Target band.** v0.28.x  
**Breaking change.** May be breaking if existing tag types are changed  
**Primary area.** Tags / API polish

---

## 1. Summary

Refine tag metadata before broad API stabilization: add missing annotation fields, document peeling behavior precisely, and clarify tag creation semantics across Git and jj.

---

## 2. Motivation

The handoff already flags `TagAnnotation.tagger_email` as available but omitted. Tag APIs are one of the few areas where endringer writes, so they deserve especially clear contracts.

The project also needs to remove stale comments implying that jj annotated tags fall back to lightweight tags. The better contract is explicit unsupported behavior for annotated tags on jj.

---

## 3. Goals

- Add `tagger_email` to `TagAnnotation` if accepted before a stabilization boundary.
- Document that `TagInfo.commit_id` is peeled to a commit.
- Clarify behavior for tags pointing to non-commit objects.
- Clarify lightweight vs annotated tag creation.
- Standardize errors for unsupported jj annotated tags.

---

## 4. Non-goals

- Do not add signed tag creation.
- Do not verify GPG/SSH signatures.
- Do not push tags.
- Do not add tag renaming.

---

## 5. External design

### 5.1 Type change

```rust
pub struct TagAnnotation {
    pub message: String,
    pub tagger_name: Option<String>,
    pub tagger_email: Option<String>,
    pub tagger_timestamp: Option<SystemTime>,
}
```

Adding a public field is breaking for exhaustive struct literals. If consumers are expected not to construct library-produced structs, this is still semver-relevant in Rust and should be bundled with a planned breaking band.

### 5.2 Peeling semantics

Document `TagInfo.commit_id` as:

> The commit reached by peeling the tag target to a commit. Tags that cannot be peeled to a commit are skipped or returned as an error according to the method's documented behavior.

Choose one behavior explicitly:

- **Option A: skip non-commit tags** for list methods;
- **Option B: include richer tag target type**.

Recommended first choice: Option A for current compatibility, plus a future `TagTarget` type only if needed.

### 5.3 Creation semantics

```rust
Repository::create_tag(name: &str) -> Result<()>
Repository::create_annotated_tag(name: &str, message: &str) -> Result<()>
```

For jj backend:

```rust
Err(Error::UnsupportedBackendFeature {
    backend: Some(BackendKind::Jj),
    feature: "create_annotated_tag",
})
```

---

## 6. Internal design

### 6.1 Git backend

Read full signature data from gix tag objects and populate `tagger_email`.

**Concretely**, `endringer-git/src/tag.rs::read_annotation` already calls
`tag.tagger()` and reads `sig.name`; the tagger email is the adjacent
`sig.email` field on the same `gix` signature. The change is a single added
field read:

```rust
// existing: let name = sig.name.to_str_lossy().into_owned();
let email = sig.email.to_str_lossy().into_owned();
// ... TagAnnotation { message, tagger_name, tagger_email: Some(email), tagger_timestamp }
```

No extra object lookup or traversal is required, so the only cost of this
change is the breaking struct-field addition discussed in §5.1.

### 6.2 jj backend

Ensure implementation and docs agree: annotated tag creation returns an explicit unsupported error. It must not silently create a lightweight tag.

### 6.3 Documentation cleanup

Search and update all stale references to fallback behavior.

---

## 7. Tests and verification

- Annotated tag includes name, email, timestamp, and message.
- Lightweight tag has `annotation: None`.
- jj annotated tag creation returns unsupported error.
- Tag list sorting remains unchanged.
- Tests cover non-commit tag behavior if fixtures are feasible.

---

## 8. Rollout plan

1. Clean stale docs immediately.
2. Add `tagger_email` in a breaking-compatible release band.
3. Add typed unsupported error after or with RFC 006.
4. Update migration notes.

---

## 9. Risks and mitigations

**Risk: public field addition breaks consumers.** Bundle with other API-polish changes.

**Risk: non-commit tags force larger design.** Document current behavior and defer richer target types unless needed.

**Risk: jj behavior confusion.** Add direct tests and docs.

---

## 10. Definition of done

- `TagAnnotation` completeness decision is implemented.
- jj annotated-tag behavior is consistent in implementation, rustdoc, handoff, and tests.
- Peeling behavior is documented.
- Migration note exists if public fields change.
