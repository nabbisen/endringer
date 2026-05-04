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
