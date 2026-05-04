//! Parsing helpers for `jj` CLI output.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::types::{BranchInfo, CommitId, CommitInfo, TagInfo};

/// Minimal struct for a single log-line record.
pub(super) struct LogRecord {
    pub commit_id: CommitId,
    pub summary: String,
    pub timestamp: SystemTime,
}

/// Parses a single `\t`-delimited log line for status_digest:
/// `change_id \t commit_id \t summary \t unix_secs`
pub(super) fn log_line(line: &str) -> Result<LogRecord> {
    let cols: Vec<&str> = line.splitn(4, '\t').collect();
    if cols.len() < 4 {
        anyhow::bail!("unexpected jj log output: {:?}", line);
    }
    let commit_id = CommitId::from_hex(cols[1].trim())
        .with_context(|| format!("bad commit id: {:?}", cols[1]))?;
    let summary = cols[2].trim().to_owned();
    let secs: i64 = cols[3]
        .trim()
        .parse()
        .with_context(|| format!("bad timestamp: {:?}", cols[3]))?;
    Ok(LogRecord {
        commit_id,
        summary,
        timestamp: unix_secs(secs),
    })
}

/// Parses a branch line:
/// `name \t commit_id \t summary \t unix_secs`
pub(super) fn branch_line(line: &str, prefix: &str) -> Result<BranchInfo> {
    let cols: Vec<&str> = line.splitn(4, '\t').collect();
    if cols.len() < 4 {
        anyhow::bail!("unexpected jj branch output: {:?}", line);
    }
    let name = cols[0].trim().to_owned();
    let full_name = format!("{}{}", prefix, name);
    let last_commit_id = CommitId::from_hex(cols[1].trim())
        .with_context(|| format!("bad commit id: {:?}", cols[1]))?;
    let last_commit_summary = cols[2].trim().to_owned();
    let secs: i64 = cols[3]
        .trim()
        .parse()
        .with_context(|| format!("bad timestamp: {:?}", cols[3]))?;
    Ok(BranchInfo {
        name,
        full_name,
        last_commit_id,
        last_commit_summary,
        last_commit_timestamp: unix_secs(secs),
    })
}

/// Parses a remote branch line:
/// `name \t remote \t commit_id \t summary \t unix_secs`
pub(super) fn remote_branch_line(line: &str) -> Result<BranchInfo> {
    let cols: Vec<&str> = line.splitn(5, '\t').collect();
    if cols.len() < 5 {
        anyhow::bail!("unexpected jj remote branch output: {:?}", line);
    }
    let name = cols[0].trim().to_owned();
    let remote = cols[1].trim();
    let full_name = format!("refs/remotes/{}/{}", remote, name);
    let last_commit_id = CommitId::from_hex(cols[2].trim())
        .with_context(|| format!("bad commit id: {:?}", cols[2]))?;
    let last_commit_summary = cols[3].trim().to_owned();
    let secs: i64 = cols[4]
        .trim()
        .parse()
        .with_context(|| format!("bad timestamp: {:?}", cols[4]))?;
    Ok(BranchInfo {
        name,
        full_name,
        last_commit_id,
        last_commit_summary,
        last_commit_timestamp: unix_secs(secs),
    })
}

/// Parses a full commit line:
/// `commit_id \t author \t committer \t summary \t author_secs \t committer_secs`
pub(super) fn commit_line(line: &str) -> Result<CommitInfo> {
    let cols: Vec<&str> = line.splitn(6, '\t').collect();
    if cols.len() < 6 {
        anyhow::bail!("unexpected jj commit output: {:?}", line);
    }
    let commit_id = CommitId::from_hex(cols[0].trim())
        .with_context(|| format!("bad commit id: {:?}", cols[0]))?;
    let author = cols[1].trim().to_owned();
    let committer = cols[2].trim().to_owned();
    let summary = cols[3].trim().to_owned();
    let author_secs: i64 = cols[4]
        .trim()
        .parse()
        .with_context(|| format!("bad author timestamp: {:?}", cols[4]))?;
    let committer_secs: i64 = cols[5]
        .trim()
        .parse()
        .with_context(|| format!("bad committer timestamp: {:?}", cols[5]))?;
    Ok(CommitInfo {
        commit_id,
        author,
        committer,
        summary,
        timestamp: unix_secs(author_secs),
        committer_timestamp: unix_secs(committer_secs),
    })
}

/// Parses a tag line:
/// `name \t commit_id \t summary \t unix_secs`
pub(super) fn tag_line(line: &str) -> Result<TagInfo> {
    let cols: Vec<&str> = line.splitn(4, '\t').collect();
    if cols.len() < 4 {
        anyhow::bail!("unexpected jj tag output: {:?}", line);
    }
    let name = cols[0].trim().to_owned();
    let full_name = format!("refs/tags/{}", name);
    let commit_id = CommitId::from_hex(cols[1].trim())
        .with_context(|| format!("bad commit id: {:?}", cols[1]))?;
    let commit_summary = cols[2].trim().to_owned();
    let secs: i64 = cols[3]
        .trim()
        .parse()
        .with_context(|| format!("bad timestamp: {:?}", cols[3]))?;
    Ok(TagInfo {
        name,
        full_name,
        commit_id,
        commit_summary,
        commit_timestamp: unix_secs(secs),
    })
}

fn unix_secs(secs: i64) -> SystemTime {
    if secs >= 0 {
        UNIX_EPOCH + Duration::from_secs(secs as u64)
    } else {
        UNIX_EPOCH
    }
}
