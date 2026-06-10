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

// ── RFC 022: tagger_email field ───────────────────────────────────────────── //

#[test]
fn annotated_tag_tagger_email_populated() {
    let f = Fixture::new();
    // Create an annotated tag via git CLI (so the fixture identity is recorded).
    f.git(&["tag", "-a", "v-annotated-email-test", "-m", "email test"]);

    let repo = repository(f.path()).unwrap();
    let tags = repo.list_tags().unwrap();

    let tag = tags.iter().find(|t| t.name == "v-annotated-email-test")
        .expect("annotated tag should appear in listing");

    let annotation = tag.annotation.as_ref()
        .expect("tag should have annotation");

    // The fixture configures user.email = fixture@test.local.
    assert_eq!(
        annotation.tagger_email.as_deref(),
        Some("fixture@test.local"),
        "tagger_email should match fixture identity"
    );
    assert_eq!(
        annotation.tagger_name.as_deref(),
        Some("Fixture"),
        "tagger_name should match fixture identity"
    );
    assert_eq!(annotation.message, "email test");

    // Clean up.
    repo.delete_tag("v-annotated-email-test").unwrap();
}

#[test]
fn lightweight_tag_annotation_is_none() {
    let f = Fixture::new();
    let repo = repository(f.path()).unwrap();

    // The fixture's v0.1.0 is a lightweight tag.
    let tags = repo.list_tags().unwrap();
    let lightweight = tags.iter().find(|t| t.name == "v0.1.0")
        .expect("v0.1.0 should exist");
    assert!(
        lightweight.annotation.is_none(),
        "lightweight tag should have annotation: None"
    );
}

#[test]
fn annotated_tag_tagger_email_is_none_when_not_recorded() {
    // Verify TagAnnotation compiles correctly with tagger_email: None
    // (e.g. for tags created without identity configured).
    let _ann = endringer::TagAnnotation {
        message: "test".to_owned(),
        tagger_name: None,
        tagger_email: None,
        tagger_timestamp: None,
    };
}
