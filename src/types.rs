use std::{collections::HashMap, time::SystemTime};

use gix::{self, ObjectId};

#[derive(Clone, Debug)]
pub struct Repository {
    pub inner: gix::Repository,
}

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

#[deprecated(since = "0.4.0", note = "dag info fns possibly will be removed")]
#[derive(Clone, Debug)]
pub struct DagInfo {
    pub nodes: HashMap<gix::ObjectId, CommitInfo>,
    pub edges: Vec<(gix::ObjectId, gix::ObjectId)>,
}
