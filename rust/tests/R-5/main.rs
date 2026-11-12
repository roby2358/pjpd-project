//! R-5: separator-shaped description lines are stored indented two spaces
//! when a description enters a store; other lines are unchanged and indenting
//! twice changes nothing.

use pjpd::ideas::IdeaStore;
use pjpd::tasks::{TaskPatch, TaskStore};
use pjpd::textrec::indent_separator_lines;
use tempfile::TempDir;

// R-5: only separator-shaped lines are indented, and the result is a fixed point.
#[test]
fn indent_separator_lines_defuses_only_separator_shaped_lines() {
    assert_eq!(
        indent_separator_lines("intro\n---\n----   \noutro"),
        "intro\n  ---\n  ----   \noutro"
    );
    // Not separators: text after the hyphens, existing indent, too short.
    assert_eq!(
        indent_separator_lines("--- text\n  ---\n--"),
        "--- text\n  ---\n--"
    );
    assert_eq!(
        indent_separator_lines(&indent_separator_lines("---")),
        "  ---"
    );
}

// R-5: task add stores the indented form and it reloads as one record.
#[test]
fn separator_lines_in_task_descriptions_are_indented_on_add() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let task = store.add("intro\n---\noutro", 5, "rule").unwrap();
    assert_eq!(task.description, "intro\n  ---\noutro");

    let loaded = store.load();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].description, "intro\n  ---\noutro");
}

// R-5: task patch stores the indented form.
#[test]
fn separator_lines_in_task_descriptions_are_indented_on_update() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());

    let task = store.add("plain", 5, "rule").unwrap();
    let patch = TaskPatch {
        description: Some("before\n----\nafter".to_string()),
        priority: None,
        status: None,
    };
    let updated = store.update(&task.id, &patch).unwrap().unwrap();
    assert_eq!(updated.description, "before\n  ----\nafter");
    assert_eq!(store.load().len(), 1);
}

// R-5: idea add and idea update store the indented form.
#[test]
fn separator_lines_in_idea_descriptions_are_indented() {
    let dir = TempDir::new().unwrap();
    let store = IdeaStore::new(dir.path());

    let idea = store.add("top\n---\nbottom", 5, "rule").unwrap();
    assert_eq!(idea.description, "top\n  ---\nbottom");

    let updated = store
        .update(&idea.id, "one\n-----\ntwo", 7)
        .unwrap()
        .unwrap();
    assert_eq!(updated.description, "one\n  -----\ntwo");
    assert_eq!(store.load().len(), 1);
}
