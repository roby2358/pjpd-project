//! R-8: a record with no Priority or Status line loads with priority
//! DEFAULT_LOADED_PRIORITY and status `ToDo`.

use pjpd::tasks::Task;

// R-8: no Priority and no Status line give priority 1 and status ToDo.
#[test]
fn missing_priority_and_status_get_defaults() {
    let task = Task::from_text("ID: t-ab12\nx").unwrap();
    assert_eq!(task.priority, 1);
    assert_eq!(task.status, "ToDo");
}

// R-8: a non-digit Priority line isn't a Priority, so the default applies.
#[test]
fn non_digit_priority_counts_as_missing() {
    let task = Task::from_text("ID: t-ab12\nPriority: high\nreal text").unwrap();
    assert_eq!(task.priority, 1);
}
