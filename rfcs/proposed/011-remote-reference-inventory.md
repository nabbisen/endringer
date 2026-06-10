# RFC 011 — Remote and reference inventory

**Status.** Proposed  
**Priority.** P3  
**Target band.** v0.27.x+  
**Breaking change.** No  
**Primary area.** Repository metadata

---

## 1. Summary

Add read-only APIs for listing remotes, references, and HEAD state.

This generalizes the current `remote_url(name)` method and helps consumers build repository navigation UIs without parsing Git config or refs themselves.

---

## 2. Motivation

Consumers often need more than one remote URL:

- show all remotes;
- show fetch and push URLs;
- list local branches, remote-tracking branches, and tags uniformly;
- inspect HEAD state without calling status digest.

These are all local reads.

---

## 3. Goals

- Add `remotes()`.
- Add `references()` or scoped reference listing.
- Add `head()` if not covered by RFC 009.
- Avoid network operations.

---

## 4. Non-goals

- No fetch/push/remote update.
- No credential helpers.
- No ref mutation.
- No symbolic-ref editing.

---

## 5. External design

### 5.1 Remote model

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteInfo {
    pub name: String,
    pub fetch_urls: Vec<String>,
    pub push_urls: Vec<String>,
}
```

If Git config has no separate push URL, `push_urls` may be empty rather than duplicating fetch URLs. Document this clearly.

### 5.2 Reference model

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefInfo {
    pub name: String,
    pub kind: RefKind,
    pub target: RefTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefKind {
    LocalBranch,
    RemoteBranch,
    Tag,
    Head,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefTarget {
    Direct(ObjectId),
    Symbolic(String),
    Unborn,
}
```

`ObjectId` is provided by **RFC 031** (foundation pass, v0.21.x), so `RefTarget::Direct(ObjectId)` uses it directly. (Ref targets are not always commits — annotated tags target tag objects, and some refs target trees/blobs — so `ObjectId` rather than `CommitId` is the correct type here.)

### 5.3 API

```rust
pub fn remotes(&self) -> Result<Vec<RemoteInfo>>;

pub fn references(&self) -> Result<Vec<RefInfo>>;

pub fn references_by_kind(&self, kind: RefKind) -> Result<Vec<RefInfo>>;
```

Sorting:

- remotes sorted by name;
- references sorted by full name.

---

## 6. Internal design

### 6.1 Git remotes

Read repository config for remote sections.

Config keys:

- `remote.<name>.url`
- `remote.<name>.pushurl`

Handle multiple URL values if gix exposes them.

### 6.2 Git refs

Use gix reference iteration. For each ref:

- classify prefix;
- preserve full ref name;
- distinguish direct vs symbolic refs;
- avoid peeling tags unless a later field asks for peeled target.

This API should expose references as references, not conflate them with branch/tag info APIs.

### 6.3 jj behavior

Delegate where jj uses git refs. Document any missing refs in native jj mode after RFC 007.

---

## 7. Test plan

- repository with multiple remotes;
- remote with fetch and push URLs;
- local branches;
- remote-tracking branches;
- lightweight tag;
- annotated tag target as ref object target;
- symbolic HEAD;
- unborn HEAD if supported;
- sorting.

---

## 8. Compatibility

Adds public types and methods.

No existing behavior changes.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Ref model overlaps with branch/tag APIs | Keep this low-level and reference-oriented. |
| Object IDs for tags/blobs require new type | Depend on RFC 010 or introduce `ObjectId` here. |
| Push URL semantics are subtle | Document exact behavior. |

---

## 10. Acceptance criteria

- `RemoteInfo`, `RefInfo`, `RefKind`, and `RefTarget` are public.
- `remotes()` and reference listing APIs exist in sync and async forms.
- Tests cover remote URLs, symbolic refs, branches, and tags.
