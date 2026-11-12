//! R-2: rendered records have their header lines in the on-disk order,
//! Priority/Score padded to width 4, timestamps only when set, a blank line
//! before the description, and records joined with `----`.

use pjpd::ideas::Idea;
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

// R-2: a task with both timestamps renders Created then Updated after ID.
#[test]
fn task_to_text_renders_timestamps_after_id() {
    let task = Task::from_text(
        "Priority: 42\nStatus: Done\nID: t-ab12\n\
         Created: 2020-01-01T00:00:00Z\nUpdated: 2020-01-02T00:00:00Z\n\nx",
    )
    .unwrap();
    assert_eq!(
        task.to_text(),
        "Priority:   42\nStatus: Done\nID: t-ab12\n\
         Created: 2020-01-01T00:00:00Z\nUpdated: 2020-01-02T00:00:00Z\n\nx"
    );
}

// R-2: an idea renders Score right-aligned to width 4, then ID, then the
// timestamps that are set, then a blank line and the description.
#[test]
fn idea_to_text_pads_score_and_orders_header() {
    let mut idea = Idea::from_text(
        "ID: i-ab12\nScore: 7\nUpdated: 2020-01-02T00:00:00Z\nCreated: 2020-01-01T00:00:00Z\nidea text",
    )
    .unwrap();
    assert_eq!(
        idea.to_text(),
        "Score:    7\nID: i-ab12\n\
         Created: 2020-01-01T00:00:00Z\nUpdated: 2020-01-02T00:00:00Z\n\nidea text"
    );

    idea.created = None;
    idea.updated = None;
    idea.score = 1234;
    assert_eq!(idea.to_text(), "Score: 1234\nID: i-ab12\n\nidea text");
}

// R-2: only the timestamps that are set are rendered.
#[test]
fn only_set_timestamps_are_rendered() {
    let task =
        Task::from_text("Priority: 1\nID: t-ab12\nUpdated: 2020-01-02T00:00:00Z\n\nx").unwrap();
    assert_eq!(
        task.to_text(),
        "Priority:    1\nStatus: ToDo\nID: t-ab12\nUpdated: 2020-01-02T00:00:00Z\n\nx"
    );
}
