# Cookbook: Tag management

**When to use this pattern.** You are building a release tool, a tag browser,
or a script that creates and deletes tags.

## API calls

```rust
repo.list_tags_sorted(SortOrder::ByName)      // all tags
repo.create_tag(name)                          // lightweight tag at HEAD
repo.create_annotated_tag(name, message)       // annotated tag at HEAD
repo.delete_tag(name)                          // remove a tag
```

## Minimal example

```rust,no_run
use endringer::{repository, SortOrder};
use std::path::Path;

fn tag_browser(path: &Path) -> anyhow::Result<()> {
    let repo = repository(path)?;

    for tag in repo.list_tags_sorted(SortOrder::ByName)? {
        if let Some(ann) = &tag.annotation {
            println!("{} (annotated by {} <{}>)",
                tag.name,
                ann.tagger_name.as_deref().unwrap_or("unknown"),
                ann.tagger_email.as_deref().unwrap_or(""),
            );
        } else {
            println!("{} (lightweight → {})", tag.name, tag.commit_id.short());
        }
    }
    Ok(())
}
```

## Creating tags

```rust,no_run
// Lightweight — points directly to HEAD commit.
repo.create_tag("v1.0.0")?;

// Annotated — stores tagger identity and message in a tag object.
// Returns UnsupportedBackendFeature on jj repositories.
match repo.create_annotated_tag("v1.0.0", "First stable release") {
    Ok(()) => {}
    Err(endringer::Error::UnsupportedBackendFeature { .. }) => {
        // jj: fall back to lightweight
        repo.create_tag("v1.0.0")?;
    }
    Err(e) => return Err(e.into()),
}
```

## Cost notes

- `list_tags_sorted()` iterates all tag refs; linear in the number of tags.
- `create_tag` / `create_annotated_tag` / `delete_tag` write directly to the
  git object store.

## Boundary note

Pushing tags to a remote (`git push --tags`) is a network write and is out of
scope. Shell out to `git push` after creation; the next `list_tags()` call will
reflect the confirmed local state.
