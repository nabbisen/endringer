use std::{collections::HashMap, time::SystemTime};

#[derive(Debug)]
pub struct StatusDigest {
    pub repo_name: String,
    pub current_branch: String,

    pub last_commit_summary: String,
    pub last_commit_time: SystemTime,
}

#[derive(Clone, Debug)]
pub struct DagInfo {
    pub nodes: HashMap<gix::ObjectId, CommitInfo>,
    pub edges: Vec<(gix::ObjectId, gix::ObjectId)>,
}

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub short_id: String,
    pub author: String,
    pub summary: String,
    pub timestamp: i64,
}
