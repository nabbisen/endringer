# RFC 004 — Ahead/behind graph computation

**Status.** Proposed  
**Priority.** P1  
**Target band.** v0.21.x  
**Breaking change.** Adds API; trait impact should be eased by RFC 003  
**Primary area.** Commit graph / branch relationship

---

## 1. Summary

Add read-side ahead/behind computation for two commit tips and branch-upstream convenience methods.

This gives consumers a dependable local graph primitive instead of forcing every VCS widget or branch table to reimplement divergence logic.

---

## 2. Motivation

Branch relationship is one of the most common VCS UI needs:

- "main is 3 behind origin/main";
- "feature is 5 ahead and 2 behind";
- "branch has no upstream";
- "branch upstream is gone".

`endringer` already exposes `merge_base` and `is_ancestor`, but consumers still need to walk commits and count divergence correctly. This computation is local, read-only, and backend-specific enough that it belongs in the library.

---

## 3. Goals

- Add a backend-agnostic commit-tip primitive.
- Add a branch-upstream convenience method where the backend can resolve upstream metadata.
- Define edge cases precisely: identical tips, fast-forward, unrelated histories, missing upstream.
- Preserve the read-only boundary.

---

## 4. Non-goals

- No fetch/pull/push.
- No network contact to discover whether remote branches changed.
- No automatic branch table eager computation unless RFC 005 adds a batch method.
- No jj-native change graph semantics.

---

## 5. External design

### 5.1 New type

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AheadBehind {
    /// Commits reachable from `local` but not from `upstream`.
    pub ahead: usize,
    /// Commits reachable from `upstream` but not from `local`.
    pub behind: usize,
    /// Best common ancestor, if one exists.
    pub merge_base: Option<CommitId>,
}
```

### 5.2 New API

Add to `Repository`:

```rust
pub fn ahead_behind(
    &self,
    local: &CommitId,
    upstream: &CommitId,
) -> Result<AheadBehind>;

pub fn branch_ahead_behind(
    &self,
    branch: &str,
) -> Result<Option<AheadBehind>>;
```

Semantics:

- `ahead_behind` operates on commit IDs and always returns counts if both commits exist.
- `branch_ahead_behind` resolves the configured upstream of a local branch.
- `branch_ahead_behind` returns `Ok(None)` when the branch has no configured upstream.
- If the branch has a configured upstream that no longer exists, return an error now, or defer to RFC 005 to expose `upstream_gone` explicitly. Preferred: return an error with a future typed `NotFound { kind: Ref }` once RFC 006 lands.

### 5.3 Edge cases

| Case | Result |
|---|---|
| `local == upstream` | `ahead = 0`, `behind = 0`, `merge_base = Some(local)` |
| local descends from upstream | `ahead > 0`, `behind = 0` |
| upstream descends from local | `ahead = 0`, `behind > 0` |
| both diverged | both counts may be > 0 |
| unrelated histories | `merge_base = None`; counts are commits reachable from each side and not the other |
| missing commit ID | error |
| branch without upstream | `branch_ahead_behind` returns `Ok(None)` |

---

## 6. Internal design

### 6.1 Trait method

Add to `VcsBackend`:

```rust
fn ahead_behind(&self, local: &CommitId, upstream: &CommitId) -> Result<AheadBehind>;

fn branch_ahead_behind(&self, branch: &str) -> Result<Option<AheadBehind>> {
    anyhow::bail!("backend does not support branch_ahead_behind")
}
```

If RFC 003 has landed, `branch_ahead_behind` may have an unsupported default.

### 6.2 Git implementation

Use graph traversal equivalent to:

```sh
git rev-list --left-right --count local...upstream
```

Implementation choices:

1. Use gix revision traversal APIs if stable enough.
2. Implement a small reachability-count helper using commit parents exposed by gix.

Preferred initial implementation:

- validate both object IDs resolve to commits;
- compute counts with a **single symmetric-difference traversal** rather than
  two independent full-history walks (the naive `count_reachable_excluding`
  ×2 sketch below is correct but O(2 × |reachable history|), which is
  unacceptable on large repositories).

#### Algorithm (matches `git rev-list --left-right --count local...upstream`)

`A...B` is the *symmetric difference*: commits reachable from exactly one of
`A`, `B`. `ahead` = count reachable from `local` only; `behind` = count
reachable from `upstream` only. Commits reachable from **both** (the shared
history at and below the merge base) are excluded from both counts.

Traverse both sides at once and stop descending as soon as a commit is known
to be reachable from both tips:

```text
mark(local)    with LEFT
mark(upstream) with RIGHT
frontier = max-heap of (commit_time, oid) seeded with {local, upstream}

while frontier not empty:
    pop the newest commit c with its flag set F (LEFT | RIGHT | BOTH)
    if F == BOTH:
        # c and everything below it is common history; do not count, and
        # propagate BOTH to parents so we stop counting the shared tail
        for p in parents(c): add_flag(p, BOTH)
        continue
    if F == LEFT:  ahead  += 1
    if F == RIGHT: behind += 1
    for p in parents(c):
        add_flag(p, F)         # union of flags already on p with F
        push p if newly seen or its flag set changed
```

Notes:

- Use a commit-time-ordered frontier (a priority queue keyed by committer or
  generation number) so a commit is only finalised once all paths to it have
  contributed their flags — the same discipline `git` uses. If a generation
  number / commit-graph is available via gix, prefer it over timestamps
  (timestamps can be non-monotonic across rebases and skew the stop point);
  fall back to committer time otherwise and document the caveat.
- The walk terminates once every frontier entry is `BOTH` (i.e. the merge
  base layer is reached on all paths). Cost is **O(commits strictly between
  the merge base and the two tips)**, not the full history.
- `merge_base` for the `AheadBehind.merge_base` field can be taken from the
  existing `graph::merge_base` helper, or recovered as the newest commit that
  first becomes `BOTH` during the same walk (saving a second traversal).

#### Edge-case shortcuts

| Relationship | Shortcut | Result |
|---|---|---|
| `local == upstream` | none needed | `0 / 0`, `merge_base = Some(local)` |
| `merge_base == upstream` (local is ahead only / fast-forward) | walk only `local` down to `upstream` | `behind = 0` |
| `merge_base == local` (upstream is ahead only) | walk only `upstream` down to `local` | `ahead = 0` |
| `merge_base == None` (unrelated) | two bounded full counts | per §5.3: each count is that side's whole reachable set |

The fast-forward shortcuts reuse the existing `is_ancestor` relationship
(`is_ancestor(upstream, local)` ⇒ behind = 0) but should be expressed in
terms of the merge base computed above to avoid an extra traversal.

#### Naive reference (correctness oracle only — not the shipped path)

The two-walk sketch is retained only as a parity/correctness oracle in tests
against small fixtures; production uses the single-traversal algorithm above.

```rust
fn ahead_behind(repo: &Repository, local: &CommitId, upstream: &CommitId) -> Result<AheadBehind> {
    let local_id = to_gix_object_id(local)?;
    let upstream_id = to_gix_object_id(upstream)?;
    ensure_commit(repo, local_id)?;
    ensure_commit(repo, upstream_id)?;

    // Single symmetric-difference walk (see algorithm above); merge_base is
    // recovered from the first BOTH commit or via graph::merge_base.
    symmetric_difference_counts(repo, local_id, upstream_id)
}
```

### 6.3 Branch upstream resolution

Git branch upstream comes from config:

- `branch.<name>.remote`
- `branch.<name>.merge`

For `remote = .`, upstream may be a local branch. For a normal remote, resolve to:

```text
refs/remotes/<remote>/<merge-short-name>
```

Example:

```text
branch.main.remote = origin
branch.main.merge = refs/heads/main
=> refs/remotes/origin/main
```

### 6.4 jj implementation

For now, delegate to the underlying Git backend where commit IDs and refs are git-shaped.

If upstream metadata is not faithfully available in native jj repos, `branch_ahead_behind` may return unsupported until RFC 007 verifies behavior.

---

## 7. Test plan

### 7.1 Git graph tests

Create fixture histories:

1. identical tips;
2. local one commit ahead;
3. local two commits behind;
4. both diverged after a common base;
5. unrelated histories;
6. merge commit included in history;
7. missing commit ID.

Compare to `git rev-list --left-right --count A...B` in parity tests after RFC 015.

### 7.2 Branch upstream tests

- branch with upstream;
- branch without upstream;
- upstream deleted locally;
- local upstream via `remote = .`;
- remote branch name not equal local branch name.

### 7.3 Async tests

Add wrappers in `endringer-async` and ensure sync/async results match.

---

## 8. Compatibility

Adds public types and methods.

If added as required trait methods before RFC 003, this breaks custom backend implementors. Therefore, implement RFC 003 first or give the new methods safe defaults.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Full history traversal is expensive | Document cost and add batch branch method in RFC 005. |
| Upstream resolution has Git edge cases | Cover config cases with fixtures. |
| jj semantics differ | Delegate only where verified; otherwise unsupported until RFC 007. |

---

## 10. Acceptance criteria

- `AheadBehind` is public and re-exported.
- Sync and async APIs expose ahead/behind.
- Git implementation matches `git rev-list --left-right --count` on fixtures.
- Branch upstream convenience works for normal Git remote-tracking branches.
- Missing/no-upstream states are documented.
