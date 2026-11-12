//! R-9: stamping sets `created` once, only if missing, and refreshes `updated`;
//! both are ISO-8601 UTC.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, response_json, server};
use pjpd::ideas::IdeaStore;
use pjpd::server::PutTaskParams;
use pjpd::tasks::{TaskPatch, TaskStore};
use rmcp::handler::server::wrapper::Parameters;
use tempfile::TempDir;

// R-9: a patch keeps `created` and refreshes `updated`.
#[test]
fn update_preserves_created_and_refreshes_updated() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let task = store.add("stamped", 5, "time").unwrap();
    let created = task.created.clone().unwrap();
    assert_eq!(task.updated.as_deref(), Some(created.as_str()));

    let describe = TaskPatch {
        description: Some("new text".to_string()),
        priority: None,
        status: None,
    };
    let updated = store.update(&task.id, &describe).unwrap().unwrap();
    assert_eq!(updated.created.as_deref(), Some(created.as_str()));
    assert!(updated.updated.is_some());
}

// R-9: a task created through the tool carries a UTC (`Z`) `created` stamp.
#[tokio::test]
async fn put_task_create_stamps_created_in_utc() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "Write the docs".to_string(),
            tag: Some("docs".to_string()),
            id: None,
            priority: Some(7),
            verbose: true,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert!(body["created"].as_str().unwrap().ends_with('Z'));
}

const OLD: &str = "2020-01-01T00:00:00Z";

/// True when the value is an ISO-8601 UTC timestamp in the stored format.
fn is_iso_utc(value: &str) -> bool {
    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ").is_ok()
}

fn write_pjpd_file(dir: &TempDir, name: &str, text: &str) {
    std::fs::create_dir_all(dir.path().join("pjpd")).unwrap();
    std::fs::write(dir.path().join("pjpd").join(name), text).unwrap();
}

// R-9: patching a hand-written task with no timestamps sets both, to the
// same ISO-8601 UTC instant, and saves them.
#[test]
fn patch_stamps_record_without_timestamps() {
    let dir = TempDir::new().unwrap();
    write_pjpd_file(
        &dir,
        "tasks.txt",
        "Priority:    5\nStatus: ToDo\nID: bare-ab29\n\nno stamps",
    );
    let store = TaskStore::new(dir.path());

    let reprioritize = TaskPatch {
        description: None,
        priority: Some(9),
        status: None,
    };
    let task = store.update("bare-ab29", &reprioritize).unwrap().unwrap();
    let created = task.created.unwrap();
    assert!(is_iso_utc(&created), "{created}");
    assert_eq!(task.updated.as_deref(), Some(created.as_str()));
    assert_eq!(store.get("bare-ab29").unwrap().created, Some(created));
}

// R-9: updating an idea keeps `created` and refreshes `updated`.
#[test]
fn idea_update_stamps() {
    let dir = TempDir::new().unwrap();
    write_pjpd_file(
        &dir,
        "ideas.txt",
        &format!("Score:   10\nID: idea-ab29\nCreated: {OLD}\nUpdated: {OLD}\n\nold text"),
    );
    let store = IdeaStore::new(dir.path());

    let idea = store.update("idea-ab29", "new text", 20).unwrap().unwrap();
    assert_eq!(idea.created.as_deref(), Some(OLD));
    let updated = idea.updated.unwrap();
    assert_ne!(updated, OLD);
    assert!(is_iso_utc(&updated), "{updated}");
    assert_eq!(store.load()[0].updated, Some(updated));
}

// R-9: marking an idea done keeps `created` and refreshes `updated`; one
// with no timestamps gets both.
#[test]
fn idea_mark_done_stamps() {
    let dir = TempDir::new().unwrap();
    write_pjpd_file(
        &dir,
        "ideas.txt",
        &format!(
            "Score:   10\nID: old-ab29\nCreated: {OLD}\nUpdated: {OLD}\n\nstamped\n----\n\
             Score:    5\nID: bare-cd34\n\nbare"
        ),
    );
    let store = IdeaStore::new(dir.path());

    let marked = store
        .mark_done_all(&["old-ab29".to_string(), "bare-cd34".to_string()])
        .unwrap();
    let old = marked.iter().find(|i| i.id == "old-ab29").unwrap();
    assert_eq!(old.created.as_deref(), Some(OLD));
    let updated = old.updated.clone().unwrap();
    assert_ne!(updated, OLD);
    assert!(is_iso_utc(&updated), "{updated}");

    let bare = marked.iter().find(|i| i.id == "bare-cd34").unwrap();
    let created = bare.created.clone().unwrap();
    assert!(is_iso_utc(&created), "{created}");
    assert_eq!(bare.updated, Some(created));
}
