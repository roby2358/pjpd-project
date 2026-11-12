//! R-20: `pjpd_put_idea` with `tag` creates an idea with the given score;
//! with `id` it replaces description and score; an unknown ID fails naming
//! `pjpd_list_ideas` and changes nothing.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_failure, assert_success, create_idea, response_json, server, snapshot};
use pjpd::server::PutIdeaParams;
use rmcp::handler::server::wrapper::Parameters;

// R-20: a created idea gets a tag-based ID and a creation message naming it.
#[tokio::test]
async fn put_idea_creates_idea() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_idea(Parameters(PutIdeaParams {
            score: 60,
            description: "brilliant plan".to_string(),
            tag: Some("plan".to_string()),
            id: None,
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    let id = body["id"].as_str().unwrap().to_string();
    assert!(id.starts_with("plan-"));
    assert_eq!(
        body["message"],
        format!("Idea created successfully with ID '{id}'")
    );
}

// R-20: `id` replaces the description and score, on disk too.
#[tokio::test]
async fn put_idea_updates_by_id() {
    let (dir, server) = server();
    let id = create_idea(&server, "first draft", "draft", 10).await;

    let result = server
        .pjpd_put_idea(Parameters(PutIdeaParams {
            score: 75,
            description: "second draft".to_string(),
            tag: None,
            id: Some(id.clone()),
            verbose: true,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["id"], id.as_str());
    assert_eq!(body["description"], "second draft");
    assert_eq!(body["score"], 75);
    assert_eq!(body["message"], format!("Idea '{id}' updated successfully"));

    let on_disk = std::fs::read_to_string(dir.path().join("pjpd/ideas.txt")).unwrap();
    assert!(on_disk.contains("Score:   75") && on_disk.contains("second draft"));
    assert!(!on_disk.contains("first draft"));
}

// R-20: an unknown ID fails naming pjpd_list_ideas and changes no file.
#[tokio::test]
async fn put_idea_unknown_id_fails_and_changes_nothing() {
    let (dir, server) = server();
    create_idea(&server, "existing", "exist", 10).await;
    let before = snapshot(&dir);

    let result = server
        .pjpd_put_idea(Parameters(PutIdeaParams {
            score: 5,
            description: "x".to_string(),
            tag: None,
            id: Some("ghost-ab29".to_string()),
            verbose: false,
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(
        error.contains("Idea 'ghost-ab29' not found"),
        "got: {error}"
    );
    assert!(error.contains("pjpd_list_ideas"), "got: {error}");
    assert_eq!(snapshot(&dir), before);
}
