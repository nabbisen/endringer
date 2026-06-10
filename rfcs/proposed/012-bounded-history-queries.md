# RFC 012 — Bounded history queries

**Status.** Proposed  
**Priority.** P3  
**Target band.** v0.27.x+  
**Breaking change.** No  
**Primary area.** Commit history scalability

---

## 1. Summary

Add bounded commit-history query APIs so consumers are not forced to load the entire reachable history from HEAD.

---

## 2. Motivation

`list_commits()` and `list_commits_sorted()` are simple and useful, but a large repository may have hundreds of thousands of commits. UIs usually need a first page, not the full history.

Consumers can implement their own pagination only by bypassing `endringer` or repeatedly fetching full lists. A bounded query API keeps the read abstraction useful for larger repositories.

---

## 3. Goals

- Add a query object for bounded history reads.
- Keep results owned.
- Avoid streaming/iterator lifetimes in the first version.
- Preserve existing `list_commits*` APIs.

---

## 4. Non-goals

- No revset language.
- No jj-native revsets.
- No streaming iterator API.
- No persistent cursor state.
- No built-in cache.

---

## 5. External design

### 5.1 Query model

```rust
#[derive(Clone, Debug)]
pub struct CommitQuery {
    pub start: CommitQueryStart,
    pub max_count: Option<usize>,
    pub skip: usize,
    pub since: Option<SystemTime>,
    pub until: Option<SystemTime>,
    pub order: SortOrder,
}

#[derive(Clone, Debug)]
pub enum CommitQueryStart {
    Head,
    Commit(CommitId),
    Ref(String),
}
```

### 5.2 Result model

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitQueryResult {
    pub commits: Vec<CommitInfo>,
    pub truncated: bool,
}
```

`truncated = true` means more commits may exist beyond `max_count`.

### 5.3 API

```rust
pub fn query_commits(&self, query: CommitQuery) -> Result<CommitQueryResult>;
```

Default helper:

```rust
impl CommitQuery {
    pub fn head_page(max_count: usize) -> Self;
}
```

---

## 6. Internal design

### 6.1 Traversal behavior

Initial behavior:

- start at HEAD/ref/commit;
- walk first-parent? Full graph? This RFC chooses **full reachable graph**, matching `list_commits()` unless current behavior is first-parent. Confirm before implementation;
- apply timestamp filter while walking;
- apply `skip` and `max_count`;
- sort by requested order if supported.

If `SortOrder::ByName` is poorly defined for commits, document or restrict it for `CommitQuery`.

### 6.2 Truncation detection

To set `truncated`, fetch one more commit than `max_count` when `max_count` is set.

Pseudo-code:

```rust
let limit = query.max_count.map(|n| n + 1);
let mut commits = walk(query, limit)?;
let truncated = query.max_count.is_some_and(|n| commits.len() > n);
if truncated { commits.truncate(query.max_count.unwrap()); }
```

### 6.3 No persistent cursor

Do not store cursor state in the repository. If cursor support is needed later, use opaque owned tokens generated from query parameters, not backend-held state.

---

## 7. Test plan

- first page from HEAD;
- `max_count = 0` behavior explicitly rejected or returns empty; preferred: reject with invalid argument after RFC 006;
- skip behavior;
- timestamp filters;
- start from specific commit;
- start from ref;
- truncation true/false;
- sorting contract.

---

## 8. Compatibility

Adds API only.

Existing history methods remain.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Pagination by `skip` is inefficient for deep pages | Accept for first version; document that it is offset-style. |
| Sort semantics differ from traversal order | Define exact order or restrict options. |
| Query object grows too large | Keep it minimal; avoid revset language. |

---

## 10. Acceptance criteria

- `CommitQuery`, `CommitQueryStart`, and `CommitQueryResult` are public.
- `query_commits()` exists in sync and async APIs.
- Tests prove bounded behavior avoids returning full history.
- Existing `list_commits()` behavior is unchanged.
