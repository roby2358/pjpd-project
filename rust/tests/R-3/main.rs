//! R-3: properties are read only from the leading run of recognized lines, in
//! any order, keys case-insensitive and trimmed, later duplicate wins; the
//! first unrecognized line starts the description.

use pjpd::ideas::Idea;
use pjpd::record::parse_record;
use pjpd::tasks::Task;

// R-3: keys match case-insensitively and keys and values are trimmed.
#[test]
fn property_lines_are_case_insensitive_and_trimmed() {
    let parsed = parse_record(
        "PRIORITY :  5 \nStatus: ToDo\nbody",
        &["status"],
        &["priority"],
    );
    assert_eq!(parsed.numeric_property("priority"), Some(5));
    assert_eq!(parsed.property("status"), Some("ToDo"));
    assert_eq!(parsed.description, "body");
}

// R-3: a numeric key with a non-digit value ends the header.
#[test]
fn non_digit_numeric_property_ends_header() {
    let parsed = parse_record("Priority: high\nbody", &[], &["priority"]);
    assert_eq!(parsed.numeric_property("priority"), None);
    assert_eq!(parsed.description, "Priority: high\nbody");
}

// R-3: an unknown key ends the header.
#[test]
fn unrecognized_property_lines_stay_in_description() {
    let parsed = parse_record("note: remember\nbody", &["status"], &[]);
    assert_eq!(parsed.description, "note: remember\nbody");
}

// R-3: within the header a later duplicate wins.
#[test]
fn later_duplicate_within_header_wins() {
    let parsed = parse_record("Status: ToDo\nStatus: Done", &["status"], &[]);
    assert_eq!(parsed.property("status"), Some("Done"));
}

// R-3: property-shaped lines after the description starts are description.
#[test]
fn property_shaped_line_after_description_start_is_not_consumed() {
    let parsed = parse_record(
        "Status: ToDo\nreproduce with input\nStatus: Done\nPriority: 99",
        &["status"],
        &["priority"],
    );
    assert_eq!(parsed.property("status"), Some("ToDo"));
    assert_eq!(parsed.numeric_property("priority"), None);
    assert_eq!(
        parsed.description,
        "reproduce with input\nStatus: Done\nPriority: 99"
    );
}

// R-3: once "Priority: high" ends the header, a valid Status line after it is
// description too.
#[test]
fn header_ends_at_first_non_property_line() {
    let parsed = parse_record(
        "Priority: high\nStatus: Done\nbody",
        &["status"],
        &["priority"],
    );
    assert_eq!(parsed.property("status"), None);
    assert_eq!(parsed.description, "Priority: high\nStatus: Done\nbody");
}

// R-3: a full task header is read, and a line without a colon starts the
// multi-line description.
#[test]
fn parses_full_task_record() {
    let task = Task::from_text(
        "Priority:   7\nStatus: ToDo\nID: mytag-ab12\nCreated: 2026-01-01T00:00:00Z\nUpdated: 2026-01-02T00:00:00Z\nDo the thing\nacross two lines",
    )
    .unwrap();
    assert_eq!(task.id, "mytag-ab12");
    assert_eq!(task.priority, 7);
    assert_eq!(task.status, "ToDo");
    assert_eq!(task.description, "Do the thing\nacross two lines");
    assert_eq!(task.created.as_deref(), Some("2026-01-01T00:00:00Z"));
    assert_eq!(task.updated.as_deref(), Some("2026-01-02T00:00:00Z"));
}

// R-3: a non-digit Priority line in a task ends the header after ID.
#[test]
fn non_digit_priority_line_becomes_task_description() {
    let task = Task::from_text("ID: t-ab12\nPriority: high\nreal text").unwrap();
    assert_eq!(task.id, "t-ab12");
    assert_eq!(task.description, "Priority: high\nreal text");
}

// R-3: property lines inside a task description don't override the header.
#[test]
fn property_lines_inside_description_do_not_override_header() {
    let task = Task::from_text(
        "Priority:   80\nStatus: ToDo\nID: real-ab12\nSteps to close out:\nStatus: Done\nID: fake-zz99\nPriority: 1",
    )
    .unwrap();
    assert_eq!(task.id, "real-ab12");
    assert_eq!(task.status, "ToDo");
    assert_eq!(task.priority, 80);
    assert_eq!(
        task.description,
        "Steps to close out:\nStatus: Done\nID: fake-zz99\nPriority: 1"
    );
}

// R-3: an idea header (Score, ID) is read and the rest is description.
#[test]
fn parses_idea_record() {
    let idea = Idea::from_text("Score:  30\nID: spark-ab12\ntry the thing").unwrap();
    assert_eq!(idea.score, 30);
    assert_eq!(idea.id, "spark-ab12");
    assert_eq!(idea.description, "try the thing");
}
