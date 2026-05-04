//! Tag creation, deletion, and listing tests.
#[path = "support/fixture.rs"]
mod fixture;
use fixture::Fixture;

use endringer::repository::repository;
use endringer::SortOrder;

#[test]
fn list_tags_initial() {
    let f = Fixture::new();
    let tags = repository(f.path()).unwrap().list_tags().unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "v0.1.0");
    assert!(tags[0].full_name.starts_with("refs/tags/"));
    assert_eq!(tags[0].commit_summary, "initial commit");
}

#[test]
fn list_tags_sorted_by_name() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    repo.create_tag("v0.2.0").unwrap();
    let tags = repo.list_tags_sorted(SortOrder::ByName).unwrap();
    let names: Vec<_> = tags.iter().map(|t| t.name.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    repo.delete_tag("v0.2.0").unwrap();
}

#[test]
fn create_and_delete_lightweight_tag() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let name = "test-tag-lightweight";
    let _ = repo.delete_tag(name); // pre-clean

    repo.create_tag(name).unwrap();
    assert!(repo.list_tags().unwrap().iter().any(|t| t.name == name));

    repo.delete_tag(name).unwrap();
    assert!(!repo.list_tags().unwrap().iter().any(|t| t.name == name));
}

#[test]
fn create_and_delete_annotated_tag() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();
    let name = "test-tag-annotated";
    let _ = repo.delete_tag(name);

    match repo.create_annotated_tag(name, "Release note") {
        Err(e)
            if e.to_string().contains("committer")
                || e.to_string().contains("identity")
                || e.to_string().contains("user.name") =>
        {
            return; // no git identity in this environment — skip
        }
        Err(e) => panic!("unexpected error: {e}"),
        Ok(()) => {}
    }

    assert!(repo.list_tags().unwrap().iter().any(|t| t.name == name));
    repo.delete_tag(name).unwrap();
}
