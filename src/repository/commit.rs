use anyhow::Result;
use gix::Repository;

use crate::{types::StatusDigest, util::seconds_to_systemtime};

pub fn status_digest(repository: &Repository) -> Result<StatusDigest> {
    // repo name
    let repo_name = repository
        .workdir()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // default branch (HEAD symbolic ref)
    let mut head = repository.head()?;
    let current_branch = if head.is_detached() {
        "(detached)".to_owned()
    } else {
        head.referent_name()
            .expect("failed to get referent name")
            .to_string()
    };

    // last commit
    let commit = head.peel_to_commit()?;

    let last_commit_id = commit.id;

    let last_commit_summary = commit
        .message()
        .expect("failed to get message")
        .summary()
        .to_string();

    let commit_time = {
        let secs = commit.time().expect("failed to get time").seconds;
        seconds_to_systemtime(secs as u64)
    };

    Ok(StatusDigest {
        repo_name,
        current_branch,
        last_commit_id,
        last_commit_summary,
        last_commit_timestamp: commit_time,
    })
}
