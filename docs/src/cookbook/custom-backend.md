# Cookbook: Custom backend

**When to use this pattern.** You need to test code that depends on
`endringer::Repository` without a real git repository, or you want to support
a VCS that endringer does not include.

## Implementing `VcsBackend`

```rust,no_run
use endringer::{VcsBackend, CommitInfo, StatusDigest, SortOrder, BackendKind, Repository};
use endringer::Result;

struct TestBackend {
    commits: Vec<CommitInfo>,
}

impl VcsBackend for TestBackend {
    fn status_digest(&self) -> Result<StatusDigest> {
        Ok(StatusDigest {
            repo_name: "test-repo".into(),
            current_branch: "main".into(),
            last_commit_id: self.commits[0].commit_id.clone(),
            last_commit_summary: self.commits[0].summary.clone(),
            last_commit_timestamp: self.commits[0].timestamp,
        })
    }

    fn list_commits(&self) -> Result<Vec<CommitInfo>> {
        Ok(self.commits.clone())
    }

    fn list_commits_sorted(&self, order: SortOrder) -> Result<Vec<CommitInfo>> {
        let mut v = self.commits.clone();
        // apply sort ...
        Ok(v)
    }

    // Methods with defaults (UnsupportedBackendFeature) need not be overridden
    // unless you want to support them.
    // Required methods with no default must be implemented.
    // See VcsBackend docs for the full list.
    # fn log_since(&self, ..) -> .. { todo!() }
    # // ... other required methods
}

fn make_repo(commits: Vec<CommitInfo>) -> Repository {
    Repository::with_backend(
        Box::new(TestBackend { commits }),
        BackendKind::Git,
    )
}
```

## Stability note

`VcsBackend` is **not yet stable**. Its method set may change before v1.0.
Methods with default implementations (`UnsupportedBackendFeature`) are safe
to omit from custom backends — adding new defaulted methods in a future
release will not break existing backends. Required methods (those without a
default) are the stable core; see the trait docs for the current list.

## Cost notes

Custom backends are injected at construction time. The `Repository` façade
is identical to the git/jj backends — all the same methods and async
wrappers are available.
