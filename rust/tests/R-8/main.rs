//! R-8: a record with no Priority/Score or Status line loads with
//! priority/score DEFAULT_LOADED_PRIORITY and status `ToDo`.

use pjpd::ideas::Idea;
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

// R-8: an idea with no Score line, or a non-digit one, gets score 1.
#[test]
fn missing_score_gets_default() {
    let idea = Idea::from_text("ID: i-ab12\nan idea").unwrap();
    assert_eq!(idea.score, 1);
    let idea = Idea::from_text("ID: i-ab12\nScore: lots\nan idea").unwrap();
    assert_eq!(idea.score, 1);
}
