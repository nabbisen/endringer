# Cookbook: Jujutsu repositories

**When to use this pattern.** You need to read a jj repository with the same
API you use for git. endringer presents jj through the underlying git object
store ("git-store view").

## Supported reads

All read methods work on jj repositories. The only behavioural difference is
`create_annotated_tag`, which returns `UnsupportedBackendFeature` because jj
only supports lightweight tags.

```rust,no_run
use endringer::{jj_repository, Error};
use std::path::Path;

fn open_jj(path: &Path) -> anyhow::Result<()> {
    let repo = jj_repository(path)?;

    let digest = repo.status_digest()?;
    println!("repo: {}, branch: {}", digest.repo_name, digest.current_branch);

    // Lightweight tag: ok.
    repo.create_tag("v0.1.0")?;

    // Annotated tag: returns UnsupportedBackendFeature.
    if let Err(Error::UnsupportedBackendFeature { .. }) =
        repo.create_annotated_tag("v0.2.0", "msg")
    {
        println!("jj: annotated tags not supported, used lightweight");
    }
    Ok(())
}
```

## What is not surfaced

- jj **change IDs** (distinct from commit IDs)
- jj **operation log**
- jj **working-copy commit** (a jj-internal ref)
- **First-class conflict objects** stored inside jj commits

These are intentionally absent. endringer's jj path is a "git-store view" and
does not expose jj-native concepts.

## Store layout detection

| Layout | Detection | Git store |
|---|---|---|
| Co-located | `.git/` and `.jj/` both present | project root |
| Native jj | only `.jj/` present | `.jj/repo/store/git/` |
