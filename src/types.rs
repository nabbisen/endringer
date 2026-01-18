use std::{collections::HashMap, time::SystemTime};

use gix::ObjectId;

#[derive(Debug)]
pub struct BranchInfo {
    pub name: String,
    pub full_name: String,
    pub last_commit_id: ObjectId,
    /// UNIXタイムスタンプ (秒)
    pub last_commit_timestamp: i64,
    /// タイムゾーンのオフセット (秒)
    pub offset: i32,
}

#[derive(Debug, Clone)]
pub struct StatusDigest {
    pub repo_name: String,
    pub current_branch: String,

    pub last_commit_summary: String,
    pub last_commit_time: SystemTime,
}

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub short_id: String,
    pub author: String,
    pub summary: String,
    pub timestamp: i64,
}

#[deprecated(since = "0.4.0", note = "dag info fns possibly will be removed")]
#[derive(Clone, Debug)]
pub struct DagInfo {
    pub nodes: HashMap<gix::ObjectId, CommitInfo>,
    pub edges: Vec<(gix::ObjectId, gix::ObjectId)>,
}
