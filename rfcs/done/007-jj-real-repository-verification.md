# RFC 007 — jj real-repository verification

**Status.** Implemented (v0.24.0)  
**Priority.** P1  
**Target band.** v0.24.x  
**Breaking change.** No  
**Primary area.** Jujutsu backend verification

---

## 1. Summary

Add tests that verify `JjBackend` against repositories created by a real `jj` binary.

The current jj backend intentionally reads jj's underlying git store with `gix` and presents a "git view of jj." That design is acceptable, but it must be verified and bounded. This RFC adds real jj fixtures and documentation of supported and unsupported jj behavior.

---

## 2. Motivation

The jj backend is attractive because consumers can use one `Repository` API for both Git and Jujutsu. However, jj has concepts Git does not:

- change IDs distinct from commit IDs;
- operation log;
- working-copy commit;
- first-class conflicts in commits;
- evolving storage formats.

A git-store read may be sufficient for many repository facts, but correctness should be tested against jj-authored repositories rather than assumed.

---

## 3. Goals

- Add a real jj test environment.
- Verify existing read APIs on jj-authored repositories.
- Test both colocated `.git` + `.jj` and native `.jj/repo/store/git` layouts where possible.
- Keep runtime dependency promise: no `jj` binary required at library runtime.
- Document the exact jj support boundary.

---

## 4. Non-goals

- No jj-native public API yet.
- No change IDs, operation log, or working-copy commit surface in this RFC.
- No jj write operations.
- No support promise for every future jj storage format.

---

## 5. External design

### 5.1 Documentation update

Add or update `docs/src/reference/backends.md`:

```markdown
## Jujutsu support level

`endringer` currently exposes a git-store view of jj repositories. It reads
commit objects, refs, trees, and tags through the underlying git store. It does
not expose jj change IDs, the operation log, the working-copy commit, or
jj-native conflict objects.

The `jj` binary is used only in tests and is not required at runtime.
```

### 5.2 CI behavior

Add one of these options:

#### Option A — mandatory jj job

A separate CI job installs `jj` and runs jj tests.

Pros: strongest confidence.  
Cons: more CI dependencies.

#### Option B — optional jj job

Run jj tests only when `jj` is installed or when a feature/env var is set:

```sh
ENDRINGER_RUN_JJ_CLI_TESTS=1 cargo test -p endringer --test jj_real
```

Pros: less disruptive.  
Cons: weaker default confidence.

Preferred: Option A on Linux CI, Option B for local development fallback.

---

## 6. Internal design

### 6.1 Test fixture helper

Add:

```text
crates/endringer/tests/support/jj_fixture.rs
```

Example helper:

```rust
pub struct JjFixture {
    pub temp: tempfile::TempDir,
    pub path: PathBuf,
}

impl JjFixture {
    pub fn native() -> Self;
    pub fn colocated() -> Self;
    pub fn jj(&self, args: &[&str]);
    pub fn write(&self, path: &str, contents: &str);
}
```

Environment isolation should mirror Git fixture isolation where applicable:

- no interactive prompts;
- deterministic author/committer identity if jj honors environment variables;
- temporary config.

### 6.2 Test cases

Initial tests:

1. open native jj repository;
2. open colocated jj repository;
3. `status_digest` reports project root repo name, not `.jj/repo/store/git`;
4. commit history returns jj-created commits;
5. `file_at_commit` reads a file from a jj-created commit;
6. lightweight tags are visible if jj/git-store creates them;
7. annotated tag creation returns unsupported;
8. unsupported or unverified methods are explicitly documented.

### 6.3 Feature gating

Mark tests requiring `jj`:

```rust
fn require_jj() {
    let ok = std::process::Command::new("jj")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        eprintln!("skipping jj CLI test: jj not installed");
        return;
    }
}
```

Better for CI: fail when env var requires them.

```rust
if std::env::var_os("ENDRINGER_REQUIRE_JJ_CLI_TESTS").is_some() && !ok {
    panic!("jj CLI tests required but jj is not installed");
}
```

---

## 7. Test plan

The RFC itself is the test plan. Success means jj-created repositories are part of the normal confidence story.

Additionally:

- run tests against pinned jj version in CI;
- record jj version in CI logs;
- periodically test newer jj versions manually or in a scheduled job.

---

## 8. Compatibility

No public API changes.

Docs may become more conservative about jj support. That is a correction, not a regression.

---

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| jj CLI changes break tests | Pin CI version or allow a tested version range. |
| Installing jj slows CI | Put jj tests in a separate job. |
| Tests reveal current backend assumptions are wrong | Treat that as the desired outcome; fix docs or code. |
| Native jj storage evolves | Document supported jj versions. |

---

## 10. Acceptance criteria

- CI has at least one jj real-repository test path.
- Native and colocated layouts are tested or explicitly documented as pending.
- `docs/src/reference/backends.md` defines the jj promise precisely.
- No runtime dependency on `jj` is introduced.
