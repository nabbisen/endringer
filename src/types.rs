use std::time::SystemTime;

use gix::{self, ObjectId};

#[derive(Debug)]
pub struct BranchInfo {
    pub name: String,
    pub full_name: String,
    pub last_commit_id: ObjectId,
    pub last_commit_summary: String,
    pub last_commit_timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct StatusDigest {
    pub repo_name: String,
    pub current_branch: String,
    pub last_commit_id: ObjectId,
    pub last_commit_summary: String,
    pub last_commit_timestamp: SystemTime,
}

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub commit_id: ObjectId,
    pub author: String,
    pub summary: String,
    pub timestamp: SystemTime,
}
