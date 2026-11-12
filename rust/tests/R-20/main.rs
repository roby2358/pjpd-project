//! R-20: `pjpd_put_idea` with `tag` creates an idea with the given score.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, response_json, server};
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
