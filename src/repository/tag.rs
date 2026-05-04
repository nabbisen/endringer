use anyhow::{Context, Result};
use gix::Repository;

use crate::{
    types::{CommitId, TagInfo},
    util::seconds_to_systemtime,
};

const TAGS_PREFIX: &str = "refs/tags/";

/// Lists all lightweight and annotated tags in the repository.
///
/// Annotated tag objects are automatically peeled to their target commit.
pub(crate) fn list_tags(repository: &Repository) -> Result<Vec<TagInfo>> {
    let references = repository
        .references()
        .context("failed to get references")?;
    let platform = references
        .prefixed(TAGS_PREFIX)
        .context("failed to filter tag references")?;

    let mut tags = Vec::new();

    for res in platform {
        let reference = res.map_err(|e| anyhow::anyhow!("reference error: {}", e))?;

        // Peel through tag objects to reach the underlying commit.
        let commit = reference
            .clone()
            .peel_to_commit()
            .map_err(|e| anyhow::anyhow!("failed to peel tag to commit: {}", e))?;

        let commit_id = CommitId(commit.id);
        let commit_summary = commit
            .message()
            .context("failed to read tagged commit message")?
            .summary()
            .to_string();
        let commit_timestamp = seconds_to_systemtime(
            commit
                .time()
                .context("failed to read tagged commit timestamp")?
                .seconds,
        );

        tags.push(TagInfo {
            name: reference.name().shorten().to_string(),
            full_name: reference.name().as_bstr().to_string(),
            commit_id,
            commit_summary,
            commit_timestamp,
        });
    }

    Ok(tags)
}

/// Creates a new **lightweight** tag pointing to the current HEAD commit.
///
/// Returns an error if a tag with `name` already exists.
pub(crate) fn create_tag(repository: &Repository, name: &str) -> Result<()> {
    let head_id = repository
        .head()?
        .id()
        .ok_or_else(|| anyhow::anyhow!("HEAD is not pointing to a commit"))?;

    let full_ref = format!("{}{}", TAGS_PREFIX, name);

    repository
        .reference(
            full_ref.as_str(),
            head_id.detach(),
            gix::refs::transaction::PreviousValue::MustNotExist,
            format!("tag: created lightweight tag {}", name),
        )
        .with_context(|| format!("failed to create tag '{}'", name))?;

    Ok(())
}

/// Creates a new **annotated** tag pointing to the current HEAD commit.
///
/// The tag object records `message` and derives the tagger identity from the
/// repository's `user.name` / `user.email` git configuration.  The tagger
/// timestamp is taken from the system clock at call time.
///
/// Returns an error if a tag with `name` already exists, or if the repository
/// has no configured identity.
pub(crate) fn create_annotated_tag(
    repository: &Repository,
    name: &str,
    message: &str,
) -> Result<()> {
    let head_id = repository
        .head()?
        .id()
        .ok_or_else(|| anyhow::anyhow!("HEAD is not pointing to a commit"))?
        .detach();

    // Resolve tagger identity from git config (user.name / user.email).
    let tagger = repository
        .committer()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no committer identity found — set user.name and user.email in git config"
            )
        })?
        .context("failed to resolve committer identity")?;

    repository
        .tag(
            name,
            &head_id,
            gix::object::Kind::Commit,
            Some(tagger),
            message,
            gix::refs::transaction::PreviousValue::MustNotExist,
        )
        .with_context(|| format!("failed to create annotated tag '{}'", name))?;

    Ok(())
}

/// Deletes the tag with the given `name`.
///
/// Returns an error if no tag with that name exists.
pub(crate) fn delete_tag(repository: &Repository, name: &str) -> Result<()> {
    let full_ref = format!("{}{}", TAGS_PREFIX, name);

    let reference = repository
        .find_reference(full_ref.as_str())
        .with_context(|| format!("tag '{}' not found", name))?;

    reference
        .delete()
        .with_context(|| format!("failed to delete tag '{}'", name))?;

    Ok(())
}

/// Returns all tags sorted by `order`.
pub(crate) fn list_tags_sorted(
    repository: &Repository,
    order: crate::types::SortOrder,
) -> Result<Vec<TagInfo>> {
    let mut tags = list_tags(repository)?;
    apply_tag_sort(&mut tags, order);
    Ok(tags)
}

fn apply_tag_sort(tags: &mut Vec<TagInfo>, order: crate::types::SortOrder) {
    use crate::types::SortOrder::*;
    match order {
        NewestFirst => tags.sort_by(|a, b| b.commit_timestamp.cmp(&a.commit_timestamp)),
        OldestFirst => tags.sort_by(|a, b| a.commit_timestamp.cmp(&b.commit_timestamp)),
        ByName => tags.sort_by(|a, b| a.name.cmp(&b.name)),
    }
}
