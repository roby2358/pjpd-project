//! R-18: `pjpd_mark_task_done` sets status `Done` and moves each task to
//! `tasks_done.txt`; done tasks are hidden by default and shown with `show_done`.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, response_json, server};
use pjpd::server::{ListTasksParams, MarkTaskDoneParams};
use rmcp::handler::server::wrapper::Parameters;

// R-18: a marked task is Done, moves files, and is visible only with show_done.
#[tokio::test]
async fn mark_task_done_moves_record_to_done_file() {
    let (dir, server) = server();
    let id = create_task(&server, "finish me", "done", 5).await;

    let result = server
        .pjpd_mark_task_done(Parameters(MarkTaskDoneParams {
            task_ids: vec![id.clone()],
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["message"], "Marked 1 task(s) as done");
    assert_eq!(body["tasks"][0]["status"], "Done");
    assert_eq!(body["tasks"][0]["priority"], 5);

    let todo = std::fs::read_to_string(dir.path().join("pjpd/tasks.txt")).unwrap();
    let done = std::fs::read_to_string(dir.path().join("pjpd/tasks_done.txt")).unwrap();
    assert!(!todo.contains(&id));
    assert!(done.contains(&id));

    let hidden = assert_success(&response_json(
        &server
            .pjpd_list_tasks(Parameters(ListTasksParams {
                count: 20,
                show_done: false,
            }))
            .await,
    ));
    assert_eq!(hidden["total_tasks"], 0);
    let shown = assert_success(&response_json(
        &server
            .pjpd_list_tasks(Parameters(ListTasksParams {
                count: 20,
                show_done: true,
            }))
            .await,
    ));
    assert_eq!(shown["total_tasks"], 1);
}
