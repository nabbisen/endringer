use std::path::Path;
use std::time::{Duration, SystemTime};

use super::*;

fn open() -> Repository {
    repository(Path::new(".")).expect("failed to open repository")
}

#[test]
fn it_works_repository() {
    assert!(repository(Path::new(".")).is_ok());
    assert!(repository(Path::new("/no/such/repo")).is_err());
}

#[test]
fn it_works_backend_kind() {
    assert_eq!(open().backend_kind(), BackendKind::Git);
}

#[test]
fn it_works_status_digest() {
    let digest = open().status_digest().expect("status digest");
    assert!(!digest.repo_name.is_empty());
    assert_ne!(digest.repo_name, "unknown");
    assert!(!digest.current_branch.starts_with("refs/"));
    assert_eq!(digest.last_commit_id.short().len(), 7);
}

#[test]
fn it_works_local_branches() {
    let branches = open().local_branches().expect("local_branches");
    assert!(!branches.is_empty());
    for b in &branches {
        assert!(!b.name.is_empty());
        assert!(b.full_name.starts_with("refs/heads/"));
        assert_eq!(b.last_commit_id.short().len(), 7);
    }
}

#[test]
fn it_works_remote_branches() {
    let branches = open().remote_branches().expect("remote_branches");
    for b in &branches {
        assert!(b.full_name.starts_with("refs/remotes/"));
    }
}

#[test]
fn it_works_list_commits() {
    let commits = open().list_commits().expect("list_commits");
    assert!(!commits.is_empty());
    for c in &commits {
        assert_eq!(c.commit_id.short().len(), 7);
        assert!(!c.summary.is_empty());
        assert!(!c.author.is_empty());
    }
    for w in commits.windows(2) {
        assert!(
            w[0].timestamp >= w[1].timestamp,
            "commits must be newest-first"
        );
    }
}

#[test]
fn it_works_list_commits_sorted() {
    use crate::types::SortOrder;
    let repo = open();

    let newest_first = repo
        .list_commits_sorted(SortOrder::NewestFirst)
        .expect("sorted");
    for w in newest_first.windows(2) {
        assert!(w[0].timestamp >= w[1].timestamp, "NewestFirst violated");
    }

    let oldest_first = repo
        .list_commits_sorted(SortOrder::OldestFirst)
        .expect("sorted");
    for w in oldest_first.windows(2) {
        assert!(w[0].timestamp <= w[1].timestamp, "OldestFirst violated");
    }

    let mut ids_a: Vec<_> = newest_first
        .iter()
        .map(|c| c.commit_id.to_string())
        .collect();
    let mut ids_b: Vec<_> = oldest_first
        .iter()
        .map(|c| c.commit_id.to_string())
        .collect();
    ids_a.sort();
    ids_b.sort();
    assert_eq!(ids_a, ids_b);
}

#[test]
fn it_works_log_since() {
    let until = SystemTime::now();
    let since = until - Duration::from_secs(365 * 24 * 3600);

    let commits = open().log_since(since, until).expect("log_since");
    assert!(!commits.is_empty());
    for c in &commits {
        assert!(c.timestamp >= since);
        assert!(c.timestamp <= until);
    }

    let ancient = SystemTime::UNIX_EPOCH + Duration::from_secs(86400);
    let empty = open()
        .log_since(SystemTime::UNIX_EPOCH, ancient)
        .expect("log_since ancient");
    assert!(empty.is_empty());
}

#[test]
fn it_works_commit_id_from_hex() {
    use crate::types::CommitId;

    let digest = open().status_digest().expect("status digest");
    let hex = digest.last_commit_id.to_string();
    assert_eq!(hex.len(), 40);

    let parsed = CommitId::from_hex(&hex).expect("round-trip");
    assert_eq!(parsed, digest.last_commit_id);
    assert_eq!(parsed.short(), digest.last_commit_id.short());

    assert!(CommitId::from_hex("not-a-hash").is_err());
    assert!(CommitId::from_hex("abc123").is_err());
    assert!(CommitId::from_hex(&"z".repeat(40)).is_err());
    assert!(CommitId::from_hex(&"0".repeat(39)).is_err());
}

#[test]
fn it_works_find_commit() {
    let repo = open();
    let commits = repo.list_commits().expect("list commits");
    let expected = &commits[0];
    let found = repo.find_commit(&expected.commit_id).expect("find_commit");

    assert_eq!(found.commit_id, expected.commit_id);
    assert_eq!(found.author, expected.author);
    assert_eq!(found.summary, expected.summary);
    assert!(!found.committer.is_empty());
}

#[test]
fn it_works_commit_id_to_short_id() {
    let digest = open().status_digest().expect("status digest");
    let short = crate::commit_id_to_short_id(digest.last_commit_id);
    assert_eq!(short.len(), 7);
}

#[test]
fn it_works_commit_info_committer_fields() {
    let commits = open().list_commits().expect("list commits");
    for c in &commits {
        assert!(!c.author.is_empty());
        assert!(!c.committer.is_empty());
        let y2020 = SystemTime::UNIX_EPOCH + Duration::from_secs(1_577_836_800);
        assert!(c.committer_timestamp >= y2020);
    }
}

#[test]
fn it_works_list_tags() {
    let tags = open().list_tags().expect("list_tags");
    for t in &tags {
        assert!(t.full_name.starts_with("refs/tags/"));
        assert_eq!(t.commit_id.short().len(), 7);
        assert!(!t.commit_summary.is_empty());
    }
}

#[test]
fn it_works_create_and_delete_tag() {
    let repo = open();
    let tag_name = "endringer-test-tag-temp";
    let _ = repo.delete_tag(tag_name);

    repo.create_tag(tag_name).expect("create tag");
    let tags = repo.list_tags().expect("list tags");
    assert!(tags.iter().any(|t| t.name == tag_name));

    repo.delete_tag(tag_name).expect("delete tag");
    let tags = repo.list_tags().expect("list tags after delete");
    assert!(!tags.iter().any(|t| t.name == tag_name));
}

#[test]
fn it_works_create_and_delete_annotated_tag() {
    let repo = open();
    let tag_name = "endringer-annotated-test-tag-temp";
    let _ = repo.delete_tag(tag_name);

    repo.create_annotated_tag(tag_name, "Test annotated tag")
        .expect("create annotated tag");
    let tags = repo.list_tags().expect("list tags");
    let found = tags.iter().find(|t| t.name == tag_name);
    assert!(found.is_some());

    let t = found.unwrap();
    assert!(t.full_name.starts_with("refs/tags/"));
    assert_eq!(t.commit_id.short().len(), 7);

    repo.delete_tag(tag_name).expect("delete annotated tag");
}

#[test]
fn it_works_diff() {
    let repo = open();
    let commits = repo.list_commits().expect("list commits");
    if commits.len() < 2 {
        return;
    }

    let d = repo
        .diff(&commits[1].commit_id, &commits[0].commit_id)
        .expect("diff");
    let total = d.added.len() + d.modified.len() + d.deleted.len();
    assert!(total > 0);

    let empty = repo
        .diff(&commits[0].commit_id, &commits[0].commit_id)
        .expect("self diff");
    assert!(empty.added.is_empty() && empty.modified.is_empty() && empty.deleted.is_empty());
}

#[test]
fn it_works_remote_url() {
    let url = open().remote_url("origin");
    assert!(url.is_none());
}
