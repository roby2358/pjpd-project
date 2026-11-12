//! R-19: `pjpd_reprioritize_task` changes only priority (and `updated`).

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, response_json, server};
use pjpd::server::ReprioritizeTaskParams;
use rmcp::handler::server::wrapper::Parameters;

// R-19: the new priority is set and the description is untouched.
#[tokio::test]
async fn reprioritize_updates_priority_only() {
    let (_dir, server) = server();
    let id = create_task(&server, "shuffle me", "prio", 5).await;

    let result = server
        .pjpd_reprioritize_task(Parameters(ReprioritizeTaskParams {
            priority: 77,
            task_ids: vec![id.clone()],
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["message"], "Reprioritized 1 task(s) to priority 77");
    assert_eq!(body["tasks"][0]["priority"], 77);
    assert_eq!(body["tasks"][0]["description"], "shuffle me");
}
