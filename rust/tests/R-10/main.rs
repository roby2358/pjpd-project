//! R-10: `Done` tasks are saved to `tasks_done.txt` and all others to
//! `tasks.txt`, each sorted by priority desc; task load reads both files.

use pjpd::tasks::{TaskPatch, TaskStore};
use tempfile::TempDir;

fn done_patch() -> TaskPatch {
    TaskPatch {
        description: None,
        priority: None,
        status: Some("Done".to_string()),
    }
}

// R-10: a task marked Done leaves tasks.txt and lands in tasks_done.txt.
#[test]
fn done_tasks_split_into_done_file() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let task = store.add("do it", 5, "split").unwrap();
    store.update(&task.id, &done_patch()).unwrap();

    let todo = std::fs::read_to_string(&store.tasks_file).unwrap();
    let done = std::fs::read_to_string(&store.done_tasks_file).unwrap();
    assert!(!todo.contains(&task.id));
    assert!(done.contains(&task.id));
    assert!(done.contains("Status: Done"));
}

// R-10: tasks.txt is written in priority-descending order.
#[test]
fn tasks_persist_sorted_by_priority_descending() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    store.add("low", 1, "low").unwrap();
    store.add("high", 99, "high").unwrap();
    store.add("mid", 50, "mid").unwrap();

    let content = std::fs::read_to_string(&store.tasks_file).unwrap();
    let high_pos = content.find("high").unwrap();
    let mid_pos = content.find("mid").unwrap();
    let low_pos = content.find("low").unwrap();
    assert!(high_pos < mid_pos && mid_pos < low_pos);
}

// R-10: load returns tasks from both tasks.txt and tasks_done.txt.
#[test]
fn task_load_reads_both_todo_and_done_files() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let keep = store.add("keep", 5, "keep").unwrap();
    let finish = store.add("finish", 5, "finish").unwrap();
    store.update(&finish.id, &done_patch()).unwrap();

    let all = store.load();
    let ids: Vec<&str> = all.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&keep.id.as_str()));
    assert!(ids.contains(&finish.id.as_str()));
}
