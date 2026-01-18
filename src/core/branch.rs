use anyhow::Result;
use gix::Repository;

use crate::{
    core::util::seconds_to_systemtime,
    types::{BranchInfo, CommitInfo},
};

mod util;

pub fn local_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    util::branches(&repository, "refs/heads/")
}

pub fn remote_branches(repository: &Repository) -> Result<Vec<BranchInfo>> {
    util::branches(&repository, "refs/remotes/")
}

pub fn list_commits(repository: &Repository) -> Result<Vec<CommitInfo>> {
    // 2. HEAD を取得
    let head = repository.head()?;
    let head_id = head
        .id()
        .ok_or_else(|| anyhow::anyhow!("HEAD is not pointing to a commit"))?;

    let mut history = Vec::new();

    // 3. 履歴を走査
    let ancestors = head_id.ancestors().all()?;

    for info in ancestors {
        let info = info?;
        let commit = info.object()?;

        // メッセージと著者情報の取得
        let message = commit.message()?;
        let author = commit.author()?;

        history.push(CommitInfo {
            commit_id: info.id, // フルID
            summary: message.summary().to_string(),
            author: author.name.to_string(),
            timestamp: seconds_to_systemtime(
                commit.time().expect("failed to get commit time").seconds as u64,
            ),
        });
    }

    Ok(history)
}
