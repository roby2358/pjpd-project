//! R-2: rendered records have their header lines in the on-disk order,
//! Priority/Score padded to width 4, timestamps only when set, a blank line
//! before the description, and records joined with `----`.

use pjpd::tasks::Task;
use pjpd::textrec::join_records;

// R-2: records are joined with the four-hyphen RECORD_SEPARATOR.
#[test]
fn join_uses_four_hyphens() {
    let joined = join_records(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(joined, "a\n----\nb");
}

// R-2: Priority right-aligned to width 4, no Created/Updated when unset, one
// blank line before the description.
#[test]
fn to_text_pads_priority_and_separates_description_with_blank_line() {
    let mut task = Task::from_text("Priority: 5\nID: t-ab12\nx").unwrap();
    task.created = None;
    task.updated = None;
    assert_eq!(
        task.to_text(),
        "Priority:    5\nStatus: ToDo\nID: t-ab12\n\nx"
    );
}
