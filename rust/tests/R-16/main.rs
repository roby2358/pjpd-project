//! R-16: `pjpd_list_tasks` sorts by status rank, then priority desc, and
//! truncates to `count` with `info` naming returned and total.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, response_json, server};
use pjpd::server::ListTasksParams;
use rmcp::handler::server::wrapper::Parameters;

// R-16: highest priority first, truncated to count, info names 2 of 3.
#[tokio::test]
async fn list_tasks_sorts_and_truncates() {
    let (_dir, server) = server();
    create_task(&server, "low", "low", 1).await;
    create_task(&server, "high", "high", 90).await;
    create_task(&server, "mid", "mid", 40).await;

    let result = server
        .pjpd_list_tasks(Parameters(ListTasksParams {
            count: 2,
            show_done: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["total_tasks"], 2);
    let descriptions: Vec<&str> = body["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["description"].as_str().unwrap())
        .collect();
    assert_eq!(descriptions, vec!["high", "mid"]);
    assert_eq!(body["info"], "Returned 2 of 3 total tasks");
}
