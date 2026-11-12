//! R-10: `Done` tasks are saved to `tasks_done.txt`, `Verified` tasks to
//! `tasks_verified.txt` and all others to `tasks.txt`, each sorted by priority
//! desc; task load reads all three files.

use pjpd::tasks::{TaskPatch, TaskStore};
use tempfile::TempDir;

fn status_patch(status: &str) -> TaskPatch {
    TaskPatch {
        description: None,
        priority: None,
        status: Some(status.to_string()),
    }
}

fn done_patch() -> TaskPatch {
    status_patch("Done")
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

// R-10: Verified tasks go to tasks_verified.txt; other statuses such as
// Blocked stay in tasks.txt; no archive file is created until it has tasks.
#[test]
fn verified_tasks_split_into_verified_file() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let blocked = store.add("blocked", 5, "blocked").unwrap();
    store.update(&blocked.id, &status_patch("Blocked")).unwrap();
    assert!(!store.done_tasks_file.exists());
    assert!(!store.verified_tasks_file.exists());

    let task = store.add("check it", 5, "check").unwrap();
    store.update(&task.id, &status_patch("Verified")).unwrap();

    let todo = std::fs::read_to_string(&store.tasks_file).unwrap();
    let verified = std::fs::read_to_string(&store.verified_tasks_file).unwrap();
    assert!(todo.contains(&blocked.id) && !todo.contains(&task.id));
    assert!(verified.contains(&task.id) && verified.contains("Status: Verified"));
    assert!(!store.done_tasks_file.exists());
}

// R-10: the on-disk layout: each stage in its own file under pjpd/.
#[test]
fn on_disk_layout_has_one_file_per_stage() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let done = store.add("d", 5, "d").unwrap();
    let verified = store.add("v", 5, "v").unwrap();
    store.add("t", 5, "t").unwrap();
    store.update(&done.id, &status_patch("Done")).unwrap();
    store
        .update(&verified.id, &status_patch("Verified"))
        .unwrap();

    let pjpd = dir.path().join("pjpd");
    assert_eq!(store.tasks_file, pjpd.join("tasks.txt"));
    assert_eq!(store.done_tasks_file, pjpd.join("tasks_done.txt"));
    assert_eq!(store.verified_tasks_file, pjpd.join("tasks_verified.txt"));
    let mut names: Vec<String> = std::fs::read_dir(&pjpd)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec!["bak", "tasks.txt", "tasks_done.txt", "tasks_verified.txt"]
    );
}

// R-10: load returns tasks from tasks.txt, tasks_done.txt and tasks_verified.txt.
#[test]
fn task_load_reads_all_three_files() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let keep = store.add("keep", 5, "keep").unwrap();
    let finish = store.add("finish", 5, "finish").unwrap();
    let check = store.add("check", 5, "check").unwrap();
    store.update(&finish.id, &done_patch()).unwrap();
    store.update(&check.id, &status_patch("Verified")).unwrap();

    let all = store.load();
    let ids: Vec<&str> = all.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&keep.id.as_str()));
    assert!(ids.contains(&finish.id.as_str()));
    assert!(ids.contains(&check.id.as_str()));
}

// R-10: statuses match case-insensitively (D-6): hand-edited `done` and
// `VERIFIED` statuses are archived on the next save, keeping their spelling.
#[test]
fn hand_edited_status_case_is_archived() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let done = store.add("d", 5, "d").unwrap();
    let verified = store.add("v", 5, "v").unwrap();
    let text = std::fs::read_to_string(&store.tasks_file).unwrap();
    let edited = text
        .replace(
            &format!("Status: ToDo\nID: {}", done.id),
            &format!("Status: done\nID: {}", done.id),
        )
        .replace(
            &format!("Status: ToDo\nID: {}", verified.id),
            &format!("Status: VERIFIED\nID: {}", verified.id),
        );
    assert_ne!(text, edited);
    std::fs::write(&store.tasks_file, edited).unwrap();

    let other = store.add("t", 5, "t").unwrap();

    let todo = std::fs::read_to_string(&store.tasks_file).unwrap();
    let done_text = std::fs::read_to_string(&store.done_tasks_file).unwrap();
    let verified_text = std::fs::read_to_string(&store.verified_tasks_file).unwrap();
    assert!(todo.contains(&other.id));
    assert!(!todo.contains(&done.id) && !todo.contains(&verified.id));
    assert!(done_text.contains(&done.id) && done_text.contains("Status: done"));
    assert!(verified_text.contains(&verified.id) && verified_text.contains("Status: VERIFIED"));
}
