//! R-17: a task ID list with any unknown ID fails naming the missing IDs and
//! "No tasks were modified"; no task changes.

#[path = "../common/mod.rs"]
mod common;

use common::{
    assert_failure, assert_success, backups_of, clear_backups, create_task, list_tasks, mark_tasks,
    response_json, server, snapshot,
};
use pjpd::server::ReprioritizeTaskParams;
use rmcp::handler::server::wrapper::Parameters;

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

// R-17: pjpd_reprioritize_task with one real and one unknown ID fails the
// same way and changes no file.
#[tokio::test]
async fn reprioritize_with_unknown_id_changes_nothing() {
    let (dir, server) = server();
    let real = create_task(&server, "real", "real", 5).await;
    let before = snapshot(&dir);

    let error = assert_failure(&response_json(
        &server
            .pjpd_reprioritize_task(Parameters(ReprioritizeTaskParams {
                priority: 80,
                task_ids: vec![real, "ghost-ab29".to_string()],
                verbose: false,
            }))
            .await,
    ));
    assert!(
        error.contains("Tasks not found: ['ghost-ab29']"),
        "got: {error}"
    );
    assert!(error.contains("No tasks were modified"));
    assert_eq!(snapshot(&dir), before);
}

// R-17: a batch is one load and one save, so it leaves exactly one tasks.txt
// backup, holding the file as it was before the batch (D-16). Backup names
// are per second, so per-ID saves would leave either several backups or one
// holding a half-applied batch.
#[tokio::test]
async fn batch_saves_tasks_file_once() {
    let (dir, server) = server();
    let ids = [
        create_task(&server, "one", "one", 5).await,
        create_task(&server, "two", "two", 6).await,
        create_task(&server, "three", "three", 7).await,
    ];
    let tasks_file = dir.path().join("pjpd/tasks.txt");

    let before = std::fs::read_to_string(&tasks_file).unwrap();
    clear_backups(&dir);
    assert_success(&response_json(
        &server
            .pjpd_reprioritize_task(Parameters(ReprioritizeTaskParams {
                priority: 50,
                task_ids: ids.to_vec(),
                verbose: false,
            }))
            .await,
    ));
    assert_eq!(backups_of(&dir, "tasks"), vec![before]);

    let before = std::fs::read_to_string(&tasks_file).unwrap();
    clear_backups(&dir);
    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    assert_success(&mark_tasks(&server, &id_refs, "done").await);
    assert_eq!(backups_of(&dir, "tasks"), vec![before]);
}

// R-17: marking several tasks verified is one save too: one tasks.txt
// backup holding the file as it was before the batch.
#[tokio::test]
async fn verified_batch_saves_tasks_file_once() {
    let (dir, server) = server();
    let ids = [
        create_task(&server, "one", "one", 5).await,
        create_task(&server, "two", "two", 6).await,
        create_task(&server, "three", "three", 7).await,
    ];
    let before = std::fs::read_to_string(dir.path().join("pjpd/tasks.txt")).unwrap();
    clear_backups(&dir);

    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    assert_success(&mark_tasks(&server, &id_refs, "verified").await);
    assert_eq!(backups_of(&dir, "tasks"), vec![before]);
    assert_eq!(list_tasks(&server, "verified").await["total_tasks"], 3);
}
