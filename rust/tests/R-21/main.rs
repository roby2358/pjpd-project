//! R-21: `pjpd_list_ideas` returns ideas in save order, truncated to `count`
//! with `info` naming returned and total; `total_ideas` counts every idea.
//! Each idea is a summary (id, score, `summary`) unless `verbose` asks for the
//! full record.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, response_json, server};
use pjpd::server::{ListIdeasParams, PjpdServer, PutIdeaParams};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::Value;

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
                verbose: false,
            }))
            .await,
    ));

    let listed = assert_success(&response_json(
        &server
            .pjpd_list_ideas(Parameters(ListIdeasParams {
                count: 20,
                verbose: false,
            }))
            .await,
    ));
    assert_eq!(listed["total_ideas"], 1);
    assert_eq!(listed["ideas"][0]["score"], 60);
    assert_eq!(listed["message"], "Retrieved 1 ideas");
}

async fn put_idea(server: &PjpdServer, tag: &str, description: &str, score: i64) {
    assert_success(&response_json(
        &server
            .pjpd_put_idea(Parameters(PutIdeaParams {
                score,
                description: description.to_string(),
                tag: Some(tag.to_string()),
                id: None,
                verbose: false,
            }))
            .await,
    ));
}

async fn list_ideas(server: &PjpdServer, count: i64, verbose: bool) -> Value {
    assert_success(&response_json(
        &server
            .pjpd_list_ideas(Parameters(ListIdeasParams { count, verbose }))
            .await,
    ))
}

// R-21: truncated to count, info names 2 of 3, total_ideas counts all 3.
#[tokio::test]
async fn list_ideas_truncates_with_info_and_total() {
    let (_dir, server) = server();
    put_idea(&server, "low", "low", 1).await;
    put_idea(&server, "high", "high", 90).await;
    put_idea(&server, "mid", "mid", 40).await;

    let body = list_ideas(&server, 2, false).await;
    assert_eq!(body["total_ideas"], 3);
    let summaries: Vec<&str> = body["ideas"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["summary"].as_str().unwrap())
        .collect();
    assert_eq!(summaries, vec!["high", "mid"]);
    assert_eq!(body["info"], "Returned 2 of 3 total ideas");

    assert!(list_ideas(&server, 3, false).await.get("info").is_none());
}

// R-21: by default an idea is id, score and its first line; verbose gives the
// full record with timestamps.
#[tokio::test]
async fn summary_by_default_full_with_verbose() {
    let (_dir, server) = server();
    put_idea(&server, "multi", "first line\nsecond line", 30).await;

    let brief = list_ideas(&server, 20, false).await;
    let idea = brief["ideas"][0].as_object().unwrap();
    let keys: Vec<&String> = idea.keys().collect();
    assert_eq!(keys, vec!["id", "score", "summary"]);
    assert_eq!(idea["summary"], "first line...");

    let full = list_ideas(&server, 20, true).await;
    let idea = &full["ideas"][0];
    assert_eq!(idea["description"], "first line\nsecond line");
    assert!(idea["created"].is_string());
    assert!(idea["updated"].is_string());
}
