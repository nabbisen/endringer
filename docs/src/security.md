# Security posture

This page describes endringer's security stance for reading local repositories.
endringer is not a network service or a sandbox. Its security posture is
defensive minimalism: avoid doing things that create risk rather than claiming
resistance to attacks.

## What endringer does not do

**No runtime external-command execution.** endringer never invokes `git`,
`jj`, or any other process at runtime. Repository reads go directly through
the `gix` library to the on-disk object store. The `git` and `jj` binaries
may appear only in test fixture setup code; they are never called by library
code paths that a consumer triggers.

**No hooks executed.** `git hooks` (`post-checkout`, `post-commit`, etc.) are
not executed at any point. endringer reads objects and refs directly, bypassing
the hook infrastructure entirely.

**No network access.** All operations are local. endringer has no fetch, pull,
push, or clone functionality and no network stack.

**No credentials handled.** There is no credential store, SSH agent
interaction, or token storage.

## Threat model for local repository reads

Consumers may point endringer at repositories from semi-trusted sources (CI
checkouts, downloaded archives, repositories cloned from unknown origins).
The relevant risks and endringer's stance:

| Risk | Endringer's stance |
|---|---|
| Malicious repository hooks | Not applicable — hooks are never executed |
| Path traversal in returned paths | Paths are repository-root-relative where applicable; absolute paths appear only for worktree/repository locations |
| Very large files returned by `file_at_commit` | Raw bytes; caller is responsible for size checks before loading |
| Very deep commit history causing memory exhaustion | Use `query_commits(max_count)` for bounded reads; `list_commits()` loads full reachable history |
| Corrupt or adversarial object data | Propagated as typed `Error::Backend` or `Error::CorruptRepository`; no panic |
| Symbolic link traversal | Symlinks in tree entries are reported as `TreeEntryKind::Symlink`; their targets are not followed |
| SHA-1 collision detection | `gix` uses `sha1_checked`; a detected collision becomes `Error::Backend` |

## Resource considerations

endringer imposes no internal global limits. Consumers are responsible for:

- **Bounding history reads** with `CommitQuery::head_page(n)` rather than
  `list_commits()` on large repositories.
- **Bounding concurrent reads** with a consumer-owned `tokio::sync::Semaphore`
  when using `AsyncRepository` for multi-repo scans.
- **Checking file sizes** before acting on bytes from `file_at_commit` or
  `tree_at_commit` in environments with tight memory limits.

## Reporting security issues

Security issues may be reported directly to the maintainer (nabbisen) by
email or through the repository's issue tracker. There is no formal bug-bounty
programme; please use responsible disclosure and allow reasonable time for a
fix before public disclosure.

## What gix provides

endringer's git backend is built on [gix](https://github.com/Byron/gitoxide).
`gix` performs SHA-1 collision detection (`sha1_checked` feature). Object-store
reads that detect a collision surface as `Error::Backend` in endringer's typed
error model.

Security improvements in `gix` (and the object-store libraries it uses) are
inherited by endringer automatically on each `gix` version bump.
