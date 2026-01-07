use anyhow::{Context, Result};
use gix::Repository;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use crate::types::{CommitInfo, DagInfo, StatusDigest};

pub fn repo(repo_path: &Path) -> Result<Repository> {
    gix::open(repo_path).context("failed to open git repository")
}

pub fn status_digest(repo_path: &Path) -> Result<StatusDigest> {
    let repo = repo(repo_path)?;

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

pub fn dag(repo_path: &Path) -> Result<DagInfo> {
    let repo = repo(repo_path)?;

    // Resolve a reference (HEAD, branch name, tag, etc.) to an OID.
    let oid = repo
        .head()?
        .peel_to_object()
        .expect("HEAD must point to a commit")
        .id;

    // Store each commit once (avoid revisiting).
    let mut visited: HashSet<gix::ObjectId> = HashSet::new();

    // Nodes: OID → useful display info (author, message, timestamp)
    let mut nodes: HashMap<gix::ObjectId, CommitInfo> = HashMap::new();

    // Edges: child → parent relationships
    let mut edges: Vec<(gix::ObjectId, gix::ObjectId)> = Vec::new();

    dag_walk(&repo, oid, &mut visited, &mut nodes, &mut edges)?;

    Ok(DagInfo { nodes, edges })
}

fn dag_walk(
    repo: &Repository,
    oid: gix::ObjectId,
    visited: &mut HashSet<gix::ObjectId>,
    nodes: &mut HashMap<gix::ObjectId, CommitInfo>,
    edges: &mut Vec<(gix::ObjectId, gix::ObjectId)>,
) -> anyhow::Result<()> {
    if !visited.insert(oid) {
        // Already processed
        return Ok(());
    }

    // Decode the commit object
    let commit = repo.find_object(oid)?.into_commit();

    // Collect UI‑friendly data
    let info = CommitInfo {
        short_id: oid.to_string()[..7].to_owned(),
        author: commit
            .author()
            .expect("failed to get author")
            .name
            .to_string(),
        summary: commit
            .message()
            .expect("failed to get message")
            .summary()
            .to_string(),
        timestamp: commit.time().expect("failed to get time").seconds,
    };
    nodes.insert(oid, info);

    // Record parent edges and recurse
    for parent_oid in commit.parent_ids() {
        let parent_oid = parent_oid.object().expect("failed to get object").id;
        edges.push((oid, parent_oid));
        dag_walk(repo, parent_oid, visited, nodes, edges)?;
    }
    Ok(())
}
