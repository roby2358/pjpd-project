//! R-6: a record missing its ID keeps an explicit one, else generates one from
//! `Tag:`, else an old-form `XX-XXXX-XX` ID; an invalid `Tag:` drops the
//! record; the tag is re-derived from the ID.

use pjpd::ids::generate_legacy;
use pjpd::record::{resolve_id, tag_from_id};
use pjpd::tasks::Task;

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
