//! R-6: a record missing its ID keeps an explicit one, else generates one from
//! `Tag:`, else an old-form `XX-XXXX-XX` ID; an invalid `Tag:` drops the
//! record; the tag is re-derived from the ID. A saved legacy ID is rewritten
//! to a valid `legacy-XXXX` ID.

use std::fs;

use tempfile::TempDir;

use pjpd::ideas::IdeaStore;
use pjpd::ids::{LEGACY_TAG, generate_legacy, is_legacy_id, is_valid_id};
use pjpd::record::{resolve_id, tag_from_id};
use pjpd::tasks::{Task, TaskStore};

// R-6: an explicit non-empty ID wins over a tag.
#[test]
fn resolve_id_keeps_explicit_id() {
    assert_eq!(
        resolve_id(Some("keep-ab29"), Some("other"), "task"),
        Some("keep-ab29".to_string())
    );
}

// R-6: generate from the tag, else old-form; an invalid tag can't be salvaged.
#[test]
fn resolve_id_generates_from_tag_or_legacy() {
    assert!(
        resolve_id(None, Some("fix"), "task")
            .unwrap()
            .starts_with("fix-")
    );
    assert_eq!(resolve_id(Some(""), None, "task").unwrap().len(), 10);
    assert_eq!(resolve_id(None, Some("far too long a tag"), "task"), None);
}

// R-6: old-form IDs are XX-XXXX-XX.
#[test]
fn legacy_id_has_expected_shape() {
    let id = generate_legacy();
    let parts: Vec<&str> = id.split('-').collect();
    assert_eq!(
        parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
        vec![2, 4, 2]
    );
}

// R-6: the tag is the first ID segment when at most 12 chars, else "legacy".
#[test]
fn tag_comes_from_first_id_segment() {
    assert_eq!(tag_from_id("mytag-ab12"), "mytag");
    assert_eq!(tag_from_id("multi-part-ab12"), "multi");
    assert_eq!(tag_from_id("nodash"), "legacy");
    assert_eq!(tag_from_id("averyverylongfirstsegment-ab12"), "legacy");
}

// R-6: a loaded task with an explicit ID gets its tag re-derived from it.
#[test]
fn loaded_task_tag_is_derived_from_its_id() {
    let task = Task::from_text("Priority: 7\nID: mytag-ab12\nx").unwrap();
    assert_eq!(task.tag, "mytag");
}

// R-6: a task with no ID but a Tag line gets an ID generated from the tag.
#[test]
fn missing_id_is_generated_from_tag() {
    let task = Task::from_text("Tag: fix\nsomething").unwrap();
    assert!(task.id.starts_with("fix-"));
    assert_eq!(task.tag, "fix");
}

// R-6: with neither ID nor Tag, an old-form ID is generated and the tag is
// taken from its first segment.
#[test]
fn missing_id_and_tag_falls_back_to_legacy_id() {
    let task = Task::from_text("just a description").unwrap();
    assert_eq!(task.id.len(), 10);
    assert_eq!(task.tag, task.id.split('-').next().unwrap());
}

// R-6: the legacy-form check matches only 2-4-2 segments of the ID alphabet.
#[test]
fn legacy_form_is_recognized_exactly() {
    assert!(is_legacy_id("ab-cdef-gh"));
    assert!(is_legacy_id(&generate_legacy()));
    assert!(!is_legacy_id("keep-ab29"));
    assert!(!is_legacy_id("ab-cdef-g1"));
    assert!(!is_legacy_id("abc-cdef-gh"));
    assert!(!is_legacy_id("AB-CDEF-GH"));
}

fn write_pjpd_file(dir: &TempDir, name: &str, text: &str) {
    fs::create_dir_all(dir.path().join("pjpd")).unwrap();
    fs::write(dir.path().join("pjpd").join(name), text).unwrap();
}

// R-6: a task saved with a legacy ID is rewritten to a valid `legacy-XXXX`
// ID, in every task file, keeping the rest of the record; other IDs are kept.
#[test]
fn task_save_rewrites_legacy_ids() {
    let dir = TempDir::new().unwrap();
    write_pjpd_file(
        &dir,
        "tasks.txt",
        "Priority:    5\nStatus: ToDo\nID: ab-cdef-gh\n\nold todo\n----\n\
         Priority:    4\nStatus: ToDo\nID: custom\n\nhand id",
    );
    write_pjpd_file(
        &dir,
        "tasks_done.txt",
        "Priority:    3\nStatus: Done\nID: zz-2345-yy\n\nold done",
    );
    let store = TaskStore::new(dir.path());
    store.add("trigger a save", 9, "new").unwrap();

    let tasks = store.load();
    let find = |desc: &str| tasks.iter().find(|t| t.description == desc).unwrap();
    for desc in ["old todo", "old done"] {
        let task = find(desc);
        assert!(task.id.starts_with("legacy-"), "{}", task.id);
        assert!(is_valid_id(&task.id));
        assert_eq!(task.tag, LEGACY_TAG);
        assert_eq!(store.get(&task.id).unwrap().description, desc);
    }
    assert_eq!(find("old todo").priority, 5);
    assert_eq!(find("old done").status, "Done");
    assert_eq!(find("hand id").id, "custom");
}

// R-6: an idea saved with a legacy ID is rewritten to `legacy-XXXX`.
#[test]
fn idea_save_rewrites_legacy_ids() {
    let dir = TempDir::new().unwrap();
    write_pjpd_file(&dir, "ideas.txt", "Score:   20\nID: ab-cdef-gh\n\nold idea");
    let store = IdeaStore::new(dir.path());
    store.add("trigger a save", 10, "new").unwrap();

    let ideas = store.load();
    let idea = ideas.iter().find(|i| i.description == "old idea").unwrap();
    assert!(idea.id.starts_with("legacy-"), "{}", idea.id);
    assert!(is_valid_id(&idea.id));
    assert_eq!(idea.score, 20);
    assert!(
        fs::read_to_string(dir.path().join("pjpd/ideas.txt"))
            .unwrap()
            .contains(&format!("ID: {}", idea.id))
    );
}
