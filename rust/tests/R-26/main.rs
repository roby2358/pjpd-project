//! R-26: `pjpd_mark_task` with state `idea` turns each task's description
//! into a new idea (new ID with the task's tag, score DEMOTED_IDEA_SCORE, no
//! reference to the task) and removes the task from every task file.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, list_tasks, mark_tasks, server};

// R-26: a todo task and a done task are both demoted.
#[tokio::test]
async fn demote_turns_tasks_into_ideas() {
    let (dir, server) = server();
    let todo = create_task(&server, "rethink this\nsecond line", "rethink", 80).await;
    let done = create_task(&server, "finished thing", "finished", 10).await;
    let keep = create_task(&server, "keep me", "keep", 5).await;
    assert_success(&mark_tasks(&server, &[&done], "done").await);

    let body = assert_success(&mark_tasks(&server, &[&todo, &done], "idea").await);
    let ideas = body["ideas"].as_array().unwrap();
    assert_eq!(ideas.len(), 2);
    assert!(
        body["project_file"]
            .as_str()
            .unwrap()
            .ends_with("ideas.txt")
    );

    let rethink = ideas
        .iter()
        .find(|i| i["description"] == "rethink this\nsecond line")
        .expect("demoted description kept in full");
    let rethink_id = rethink["id"].as_str().unwrap();
    assert!(rethink_id.starts_with("rethink-"));
    assert_ne!(rethink_id, todo);
    assert_eq!(rethink["score"], 50);

    // The ideas file holds the new ideas and no reference to the tasks.
    let ideas_file = std::fs::read_to_string(dir.path().join("pjpd/ideas.txt")).unwrap();
    assert!(ideas_file.contains(rethink_id));
    assert!(!ideas_file.contains(&todo) && !ideas_file.contains(&done));

    // The tasks are gone from every task file; the other task is untouched.
    let task_files = ["tasks.txt", "tasks_done.txt"]
        .map(|name| std::fs::read_to_string(dir.path().join("pjpd").join(name)).unwrap());
    assert!(
        task_files
            .iter()
            .all(|f| !f.contains(&todo) && !f.contains(&done))
    );
    let listed = list_tasks(&server, "todo").await;
    assert_eq!(listed["total_tasks"], 1);
    assert_eq!(listed["tasks"][0]["id"], keep.as_str());
    assert_eq!(list_tasks(&server, "done").await["total_tasks"], 0);
}
