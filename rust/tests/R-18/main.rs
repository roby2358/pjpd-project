//! R-18: `pjpd_mark_task` with state `done` sets status `Done` and moves each
//! task to `tasks_done.txt`; done tasks list under status `done`, not `todo`.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, list_tasks, mark_tasks, server};

// R-18: a marked task is Done, moves files, and lists only under "done".
#[tokio::test]
async fn mark_task_done_moves_record_to_done_file() {
    let (dir, server) = server();
    let id = create_task(&server, "finish me", "done", 5).await;

    let body = assert_success(&mark_tasks(&server, &[&id], "done").await);
    assert_eq!(body["message"], "Marked 1 task(s) as done");
    assert_eq!(body["tasks"][0]["status"], "Done");
    assert_eq!(body["tasks"][0]["priority"], 5);

    let todo = std::fs::read_to_string(dir.path().join("pjpd/tasks.txt")).unwrap();
    let done = std::fs::read_to_string(dir.path().join("pjpd/tasks_done.txt")).unwrap();
    assert!(!todo.contains(&id));
    assert!(done.contains(&id));

    assert_eq!(list_tasks(&server, "todo").await["total_tasks"], 0);
    assert_eq!(list_tasks(&server, "done").await["total_tasks"], 1);
    assert_eq!(list_tasks(&server, "verified").await["total_tasks"], 0);
}

// R-18: the state value is matched case-insensitively.
#[tokio::test]
async fn mark_state_is_case_insensitive() {
    let (_dir, server) = server();
    let id = create_task(&server, "finish me", "done", 5).await;

    assert_success(&mark_tasks(&server, &[&id], "Done").await);
    assert_eq!(list_tasks(&server, "done").await["total_tasks"], 1);
}
