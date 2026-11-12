//! R-14: `pjpd_put_task` with `id` replaces the description, replaces priority
//! only when given, keeps the status, and fails for an unknown ID naming
//! `pjpd_list_tasks` without changing anything.

#[path = "../common/mod.rs"]
mod common;

use common::{
    assert_failure, assert_success, create_task, list_tasks, mark_tasks, response_json, server,
    snapshot,
};
use pjpd::server::PutTaskParams;
use rmcp::handler::server::wrapper::Parameters;

// R-14: description and a given priority are replaced.
#[tokio::test]
async fn put_task_updates_existing_task() {
    let (_dir, server) = server();
    let id = create_task(&server, "original", "fix", 3).await;

    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "updated text".to_string(),
            tag: None,
            id: Some(id.clone()),
            priority: Some(9),
            verbose: true,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["description"], "updated text");
    assert_eq!(body["priority"], 9);
    assert_eq!(body["message"], format!("Task '{id}' updated successfully"));
}

// R-14: an omitted priority keeps the existing one.
#[tokio::test]
async fn put_task_update_without_priority_preserves_existing() {
    let (_dir, server) = server();
    let id = create_task(&server, "keep my priority", "keep", 33).await;

    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "new words, same priority".to_string(),
            tag: None,
            id: Some(id.clone()),
            priority: None,
            verbose: true,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["description"], "new words, same priority");
    assert_eq!(body["priority"], 33);
}

// R-14: an unknown ID fails with a not-found message.
#[tokio::test]
async fn put_task_unknown_id_fails_gracefully() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "x".to_string(),
            tag: None,
            id: Some("ghost-ab29".to_string()),
            priority: Some(50),
            verbose: false,
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(
        error.contains("Task 'ghost-ab29' not found"),
        "got: {error}"
    );
}

// R-14: updating a task keeps its status, so a done task stays done and in
// tasks_done.txt.
#[tokio::test]
async fn put_task_update_keeps_status() {
    let (dir, server) = server();
    let id = create_task(&server, "finished work", "fin", 10).await;
    assert_success(&mark_tasks(&server, &[&id], "done").await);

    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "finished work, reworded".to_string(),
            tag: None,
            id: Some(id.clone()),
            priority: Some(20),
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["status"], "Done");
    assert_eq!(
        list_tasks(&server, "done").await["tasks"][0]["id"],
        id.as_str()
    );
    assert_eq!(list_tasks(&server, "todo").await["total_tasks"], 0);
    let done_file = std::fs::read_to_string(dir.path().join("pjpd/tasks_done.txt")).unwrap();
    assert!(done_file.contains("Status: Done") && done_file.contains("reworded"));
}

// R-14: an unknown ID fails naming pjpd_list_tasks and changes no file.
#[tokio::test]
async fn put_task_unknown_id_names_list_tool_and_changes_nothing() {
    let (dir, server) = server();
    create_task(&server, "existing", "exist", 5).await;
    let before = snapshot(&dir);

    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "x".to_string(),
            tag: None,
            id: Some("ghost-ab29".to_string()),
            priority: Some(50),
            verbose: false,
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(error.contains("pjpd_list_tasks"), "got: {error}");
    assert_eq!(snapshot(&dir), before);
}
