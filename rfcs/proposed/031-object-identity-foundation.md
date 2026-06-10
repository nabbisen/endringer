# RFC 031 — Object identity foundation (`ObjectId` and `CommitId` relationship)

**Status.** Proposed
**Priority.** P1
**Target band.** v0.21.x (foundation; must land before RFC 006)
**Breaking change.** No (additive type); enables a later non-breaking internal `CommitId` refactor
**Primary area.** Core identity types

---

## 1. Summary

Introduce a public opaque `ObjectId` type in `endringer-core` as a shared
foundation, and define its relationship to the existing `CommitId`. This RFC
exists to resolve a **dependency inversion** in the current RFC set: four
later RFCs (006, 008, 010, 011) all reference an `ObjectId` type, but it is
only *defined* in RFC 010 — which is scheduled (v0.26.x) **after** the two
RFCs that need it first (006 at v0.23.x, 008 at v0.25.x).

This RFC pulls the type definition forward into a small foundational unit so
the dependent RFCs stop carrying "if `ObjectId` does not exist yet…"
escape hatches.

---

## 2. Motivation

The current cross-RFC situation:

| RFC | Uses `ObjectId` as | Band |
|---|---|---|
| 006 typed errors | `Error::NotATree { id: ObjectId }` | v0.23.x |
| 008 conflict state | `ConflictStage { object_id: ObjectId }` | v0.25.x |
| 010 point-in-time reads | **defines** `ObjectId`; `TreeEntry.object_id` | v0.26.x |
| 011 remote/ref inventory | `RefTarget::Direct(ObjectId)` | v0.27.x+ |
| 023 SHA-256 validation | `CommitIdRepr` inline enum | v0.28.x |

RFC 006 and RFC 008 each work around the missing type with conditional
wording ("If `ObjectId` does not yet exist, `NotATree` may temporarily use
`String` or be deferred", "If `ObjectId` does not exist yet, start with
`unmerged_paths()` only"). That is fragile: it lets the typed-error variant
set and the conflict model drift depending on landing order, which is exactly
the kind of churn RFC 006 is trying to end.

`endringer` already proves the pattern with `CommitId` (opaque, byte-backed,
`from_hex`/`from_bytes`/`as_bytes`/`short`/`Display`, hides
`gix::ObjectId`). A generic `ObjectId` is the same shape with a wider domain
(blobs, trees, commits-as-submodule-gitlinks, tag objects).

---

## 3. Goals

- Define a public, opaque `ObjectId` once, early, in `endringer-core::types`.
- Define the `CommitId`/`ObjectId` relationship precisely.
- Remove the "if `ObjectId` does not exist yet" hedges from RFCs 006, 008,
  010, 011.
- Keep `gix::ObjectId` fully hidden, consistent with the design mindset.
- Do **not** force the `CommitId` inline-storage change (RFC 023); only make
  it cleanly possible later.

---

## 4. Non-goals

- No change to `CommitId`'s public surface.
- No requirement that `CommitId` become a newtype over `ObjectId` now.
- No new repository operations (this is a type-only RFC).
- No exposure of object *kind* on `ObjectId` itself (kind belongs to the
  context that produced the id, e.g. `TreeEntry.kind`, not to the id).

---

## 5. External design

### 5.1 The type

```rust
/// Opaque identifier for any Git/jj object (blob, tree, commit, or tag),
/// stored as raw bytes. Mirrors `CommitId` but is not restricted to commits.
///
/// `gix::ObjectId` is never exposed; callers construct via `from_hex` /
/// `from_bytes`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(Vec<u8>);

impl ObjectId {
    pub fn from_bytes(bytes: Vec<u8>) -> Self;
    /// Accepts 40-char (SHA-1) or 64-char (SHA-256) lowercase hex.
    pub fn from_hex(hex: &str) -> Result<Self, ObjectIdFromHexError>;
    pub fn as_bytes(&self) -> &[u8];
    /// First 7 hex characters (conventional short form).
    pub fn short(&self) -> String;
}
// Display = full hex, identical rules to CommitId.
```

The hex/byte rules are **identical** to `CommitId` (40 or 64 hex chars;
byte-level lexicographic ordering; SHA-1 and SHA-256 never compare equal).
The implementation should share the existing `hex_nibble` / `nibble_char`
helpers rather than duplicate them.

### 5.2 Relationship to `CommitId`

Two viable models; this RFC picks **Model A** for the first landing and
permits **Model B** as a later non-breaking internal refactor.

- **Model A (now): two parallel opaque newtypes.** `CommitId` and
  `ObjectId` are independent `struct(Vec<u8>)` types that happen to share
  byte semantics. `CommitId` keeps meaning "an id known to denote a commit";
  `ObjectId` means "an id that may denote any object kind." Conversions are
  explicit and lossless:

  ```rust
  impl From<CommitId> for ObjectId { /* a commit id is a valid object id */ }
  impl CommitId {
      /// A commit id is always a valid object id.
      pub fn to_object_id(&self) -> ObjectId;
  }
  impl ObjectId {
      /// Reinterpret as a commit id. The caller asserts (or has verified)
      /// that this object is a commit; endringer does not check object kind
      /// here. Prefer `Repository::find_commit` when verification matters.
      pub fn assume_commit(self) -> CommitId;
  }
  ```

  `From<ObjectId> for CommitId` is intentionally **not** provided, because
  not every object is a commit; the named `assume_commit` keeps the assertion
  visible at call sites.

- **Model B (later, optional, non-breaking): `CommitId` wraps `ObjectId`.**
  `struct CommitId(ObjectId)`. Public API is unchanged. This is the natural
  home for the RFC 023 inline-storage change, which would then live once in
  `ObjectId` and be inherited by `CommitId`.

Choosing Model A first avoids coupling this foundation to the inline-storage
decision while still giving every dependent RFC a real type to name.

### 5.3 Error type

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectIdFromHexError(String); // same shape as CommitIdFromHexError
```

After RFC 006, `ObjectId::from_hex` failures map to
`Error::InvalidObjectId { value }` (a sibling of `InvalidCommitId`). Before
RFC 006 the standalone error type is used directly.

---

## 6. Internal design

### 6.1 Placement and re-exports

- Define in `endringer-core/src/types.rs` immediately after `CommitId`
  (or, given `types.rs` is already ~345 ELOC and over the 300-line "consider
  splitting" guideline, split identity types into
  `endringer-core/src/types/identity.rs` as part of this RFC — see §6.3).
- Re-export from `endringer` alongside `CommitId`.

### 6.2 Conversion helpers in `endringer-git`

Add a `gix_object_id_to_object_id` helper mirroring the existing
`gix_id_to_commit_id` in `util.rs`. Backend code that enumerates tree
entries, ref targets, or conflict stages uses it.

### 6.3 Opportunistic `types.rs` split

`endringer-core/src/types.rs` is currently ~345 ELOC, past the project's
300-ELOC "consider splitting" threshold (development-instructions §3.5 and
the v0.19.0 audit both flag this). Introducing `ObjectId` is a natural moment
to split identity out:

```text
endringer-core/src/types.rs           // re-exports submodules (2018 module style)
endringer-core/src/types/identity.rs  // CommitId, ObjectId, *FromHexError, hex helpers
endringer-core/src/types/...          // (future splits deferred)
```

This is a non-breaking move (paths stay `endringer_core::types::CommitId` via
re-export, and the façade re-export is unchanged). Keep it scoped: only the
identity types move in this RFC; status/commit/tag type splits stay deferred
until their type sets stabilise, per the v0.19.0 audit's own advice.

---

## 7. Test plan

- `ObjectId::from_hex` accepts 40/64 hex, rejects other lengths and non-hex
  (mirror the existing `CommitId` tests).
- `short()`, `Display`, `Ord`, `Hash` behave identically to `CommitId`.
- Round-trip `CommitId -> ObjectId -> assume_commit -> CommitId` is identity.
- `From<CommitId> for ObjectId` preserves bytes.
- After the `types.rs` split, all existing paths still resolve (compile test).

---

## 8. Compatibility

Additive. No existing public item changes. Enables (does not require) the
RFC 023 inline-storage change and the RFC 006 `InvalidObjectId` /
`NotATree { id: ObjectId }` variants.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Two near-identical id types confuse users | Document the domain difference (commit vs any object) and provide named, lossless conversions; do not offer a silent `From<ObjectId> for CommitId`. |
| `assume_commit` is misused | Name encodes the assertion; docs steer kind-sensitive callers to `find_commit`. |
| Splitting `types.rs` churns imports | Keep re-exports so external paths are unchanged; bundle the split with this single RFC. |
| Couples to inline-storage debate | Model A explicitly avoids that; RFC 023 can adopt Model B later. |

---

## 10. Acceptance criteria

- `ObjectId` and `ObjectIdFromHexError` are public in `endringer-core` and
  re-exported from `endringer`.
- The `CommitId`/`ObjectId` relationship (Model A) and conversions are
  implemented and documented.
- RFCs 006, 008, 010, and 011 are updated to reference RFC 031's `ObjectId`
  and drop their "if `ObjectId` does not exist yet" hedges.
- No `gix` type is exposed.
- Identity types optionally moved to `types/identity.rs` with unchanged
  public paths.
