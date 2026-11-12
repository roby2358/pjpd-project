//! R-15: tool calls with bad arguments (bad tag, bad ID, both or neither of
//! tag/id, values out of range) fail with a message.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_failure, create_task, list_tasks, mark_tasks, response_json, server};
use pjpd::server::{ListTasksParams, PutTaskParams};
use pjpd::validation::{
    validate_exactly_one_of_tag_or_id, validate_id, validate_range, validate_tag,
};
use rmcp::handler::server::wrapper::Parameters;

// R-15: tags must be 1..12 chars of [A-Za-z0-9-].
#[test]
fn tag_rules() {
    assert!(validate_tag("fix-bug").is_ok());
    assert!(validate_tag("").is_err());
    assert!(validate_tag("thirteenchars").is_err());
    assert!(validate_tag("has space").is_err());
}

// R-15: IDs must match the ID pattern.
#[test]
fn id_rules() {
    assert!(validate_id("tag-ab29", "Task ID").is_ok());
    assert!(validate_id("nodash", "Task ID").is_err());
    assert!(validate_id("tag-ABCD", "Task ID").is_err());
}

// R-15: exactly one of tag or id.
#[test]
fn exactly_one_of_tag_or_id() {
    let some = |s: &str| Some(s.to_string());
    assert!(validate_exactly_one_of_tag_or_id(&some("t"), &None).is_ok());
    assert!(validate_exactly_one_of_tag_or_id(&None, &some("t-ab12")).is_ok());
    assert!(validate_exactly_one_of_tag_or_id(&some("t"), &some("t-ab12")).is_err());
    assert!(validate_exactly_one_of_tag_or_id(&None, &None).is_err());
}

// R-15: values outside the allowed range fail.
#[test]
fn ranges() {
    assert!(validate_range(0, 0, 9999, "priority").is_ok());
    assert!(validate_range(10000, 0, 9999, "priority").is_err());
    assert!(validate_range(0, 1, 1000, "count").is_err());
}

// R-15: pjpd_put_task with both tag and id fails.
#[tokio::test]
async fn put_task_rejects_tag_and_id_together() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "x".to_string(),
            tag: Some("t".to_string()),
            id: Some("t-ab29".to_string()),
            priority: Some(50),
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(error.contains("not both"), "got: {error}");
}

// R-15: pjpd_mark_task with an unknown state fails and changes nothing.
#[tokio::test]
async fn mark_task_rejects_unknown_state() {
    let (_dir, server) = server();
    let id = create_task(&server, "x", "x", 5).await;

    let error = assert_failure(&mark_tasks(&server, &[&id], "finished").await);
    assert!(
        error.contains("'done', 'verified' or 'idea'"),
        "got: {error}"
    );
    assert_eq!(list_tasks(&server, "todo").await["total_tasks"], 1);
}

// R-15: pjpd_list_tasks with an unknown status filter fails.
#[tokio::test]
async fn list_tasks_rejects_unknown_status() {
    let (_dir, server) = server();
    let result = server
        .pjpd_list_tasks(Parameters(ListTasksParams {
            count: 20,
            status: "all".to_string(),
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(
        error.contains("'todo', 'done' or 'verified'"),
        "got: {error}"
    );
}
