//! R-17: a task ID list with any unknown ID fails naming the missing IDs and
//! "No tasks were modified"; no task changes.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_failure, assert_success, create_task, response_json, server};
use pjpd::server::{ListTasksParams, MarkTaskDoneParams};
use rmcp::handler::server::wrapper::Parameters;

// R-17: mark_task_done with one real and one unknown ID changes nothing.
#[tokio::test]
async fn bulk_operations_are_all_or_nothing() {
    let (_dir, server) = server();
    let real = create_task(&server, "real", "real", 5).await;

    let result = server
        .pjpd_mark_task_done(Parameters(MarkTaskDoneParams {
            task_ids: vec![real.clone(), "ghost-ab29".to_string()],
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(
        error.contains("Tasks not found: ['ghost-ab29']"),
        "got: {error}"
    );
    assert!(error.contains("No tasks were modified"));

    // The real task must still be ToDo.
    let body = assert_success(&response_json(
        &server
            .pjpd_list_tasks(Parameters(ListTasksParams {
                count: 20,
                show_done: false,
            }))
            .await,
    ));
    assert_eq!(body["total_tasks"], 1);
}
