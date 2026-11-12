//! R-17: a task ID list with any unknown ID fails naming the missing IDs and
//! "No tasks were modified"; no task changes.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_failure, create_task, list_tasks, mark_tasks, server};

// R-17: pjpd_mark_task with one real and one unknown ID changes nothing, for
// every state.
#[tokio::test]
async fn bulk_operations_are_all_or_nothing() {
    let (dir, server) = server();
    let real = create_task(&server, "real", "real", 5).await;

    for state in ["done", "verified", "idea"] {
        let error = assert_failure(&mark_tasks(&server, &[&real, "ghost-ab29"], state).await);
        assert!(
            error.contains("Tasks not found: ['ghost-ab29']"),
            "got: {error}"
        );
        assert!(error.contains("No tasks were modified"));

        // The real task must still be ToDo, and no idea was made.
        let body = list_tasks(&server, "todo").await;
        assert_eq!(body["total_tasks"], 1, "state {state}");
        assert!(!dir.path().join("pjpd/ideas.txt").exists());
    }
}
