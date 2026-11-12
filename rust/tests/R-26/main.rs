//! R-26: `pjpd_mark_task` with state `idea` turns each task's description
//! into a new idea (new ID with the task's tag, score DEMOTED_IDEA_SCORE, no
//! reference to the task) and removes the task from every task file.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, list_tasks, mark_tasks, server};
use pjpd::server::DEMOTED_IDEA_SCORE;

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
        .find(|i| i["id"].as_str().unwrap().starts_with("rethink-"))
        .expect("demoted idea takes the task's tag");
    let rethink_id = rethink["id"].as_str().unwrap();
    assert_ne!(rethink_id, todo);
    assert_eq!(rethink["score"], DEMOTED_IDEA_SCORE);

    // The ideas file holds the new ideas, descriptions in full, and no
    // reference to the tasks.
    let ideas_file = std::fs::read_to_string(dir.path().join("pjpd/ideas.txt")).unwrap();
    assert!(ideas_file.contains(rethink_id));
    assert!(ideas_file.contains("rethink this\nsecond line"));
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

// R-26: a verified task is demoted too, leaving tasks_verified.txt.
#[tokio::test]
async fn demote_verified_task() {
    let (dir, server) = server();
    let id = create_task(&server, "checked thing", "checked", 30).await;
    assert_success(&mark_tasks(&server, &[&id], "verified").await);

    let body = assert_success(&mark_tasks(&server, &[&id], "idea").await);
    let idea = &body["ideas"][0];
    assert!(idea["id"].as_str().unwrap().starts_with("checked-"));
    assert_ne!(idea["id"], id.as_str());
    assert_eq!(idea["score"], DEMOTED_IDEA_SCORE);

    let verified = std::fs::read_to_string(dir.path().join("pjpd/tasks_verified.txt")).unwrap();
    assert!(!verified.contains(&id));
    assert_eq!(list_tasks(&server, "verified").await["total_tasks"], 0);
    let ideas_file = std::fs::read_to_string(dir.path().join("pjpd/ideas.txt")).unwrap();
    assert!(ideas_file.contains("checked thing"));
}
