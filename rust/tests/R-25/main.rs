//! R-25: `pjpd_mark_task` with state `verified` sets status `Verified` and
//! moves each task to `tasks_verified.txt`, from done or straight from todo;
//! verified tasks list only under status `verified`.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, list_tasks, mark_tasks, server};

// R-25: one task verified straight from todo, one from done.
#[tokio::test]
async fn mark_task_verified_from_todo_and_done() {
    let (dir, server) = server();
    let straight = create_task(&server, "straight", "straight", 5).await;
    let reviewed = create_task(&server, "reviewed", "reviewed", 6).await;
    assert_success(&mark_tasks(&server, &[&reviewed], "done").await);

    let body = assert_success(&mark_tasks(&server, &[&straight, &reviewed], "verified").await);
    assert_eq!(body["message"], "Marked 2 task(s) as verified");
    assert_eq!(body["tasks"][0]["status"], "Verified");
    assert_eq!(body["tasks"][1]["status"], "Verified");

    let read = |name: &str| std::fs::read_to_string(dir.path().join("pjpd").join(name)).unwrap();
    let verified = read("tasks_verified.txt");
    assert!(verified.contains(&straight) && verified.contains(&reviewed));
    assert!(verified.contains("Status: Verified"));
    assert!(!read("tasks.txt").contains(&straight));
    assert!(!read("tasks_done.txt").contains(&reviewed));

    assert_eq!(list_tasks(&server, "todo").await["total_tasks"], 0);
    assert_eq!(list_tasks(&server, "done").await["total_tasks"], 0);
    let listed = list_tasks(&server, "verified").await;
    assert_eq!(listed["total_tasks"], 2);
    // Sorted by priority descending.
    assert_eq!(listed["tasks"][0]["id"], reviewed.as_str());
}
