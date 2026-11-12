//! R-21: `pjpd_list_ideas` returns ideas in save order, truncated to `count`.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, response_json, server};
use pjpd::server::{ListIdeasParams, PutIdeaParams};
use rmcp::handler::server::wrapper::Parameters;

// R-21: a stored idea is listed with its score.
#[tokio::test]
async fn list_ideas_returns_stored_idea() {
    let (_dir, server) = server();
    assert_success(&response_json(
        &server
            .pjpd_put_idea(Parameters(PutIdeaParams {
                score: 60,
                description: "brilliant plan".to_string(),
                tag: Some("plan".to_string()),
                id: None,
            }))
            .await,
    ));

    let listed = assert_success(&response_json(
        &server
            .pjpd_list_ideas(Parameters(ListIdeasParams { count: 20 }))
            .await,
    ));
    assert_eq!(listed["total_ideas"], 1);
    assert_eq!(listed["ideas"][0]["score"], 60);
    assert_eq!(listed["message"], "Retrieved 1 ideas");
}
