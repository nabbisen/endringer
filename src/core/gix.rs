use anyhow::{Context, Result};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct StatusDigest {
    pub repo_name: String,
    pub current_branch: String,

    pub last_commit_summary: String,
    pub last_commit_time: SystemTime,
}

pub fn status_digest(repo_path: &Path) -> Result<StatusDigest> {
    // open repository
    let repo = gix::open(repo_path).context("failed to open git repository")?;

    // repo name
    let repo_name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // default branch (HEAD symbolic ref)
    let mut head = repo.head()?;
    let current_branch = if head.is_detached() {
        "(detached)".to_owned()
    } else {
        head.referent_name()
            .expect("failed to get referent name")
            .to_string()
    };

    // last commit
    let commit = head.peel_to_commit()?;

    let last_commit_summary = commit
        .message()
        .expect("failed to get message")
        .summary()
        .to_string();

    let commit_time = {
        let secs = commit.time().expect("failed to get time").seconds;
        UNIX_EPOCH + Duration::from_secs(secs as u64)
    };

    Ok(StatusDigest {
        repo_name,
        current_branch,

        last_commit_summary,
        last_commit_time: commit_time,
    })
}
