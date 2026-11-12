//! R-4: render then parse yields an equal record; no description line becomes
//! a property.

use pjpd::ideas::Idea;
use pjpd::tasks::Task;

// R-4: a task with every field set round-trips unchanged.
#[test]
fn task_roundtrips_through_text() {
    let original = Task::from_text(
        "Priority:  42\nStatus: Done\nID: tag-wxyz\nCreated: 2026-01-01T00:00:00Z\nUpdated: 2026-01-01T00:00:00Z\nline one\nline two",
    )
    .unwrap();
    let reparsed = Task::from_text(&original.to_text()).unwrap();
    assert_eq!(original, reparsed);
}

// R-4: a task description of property-shaped lines survives write and reload.
#[test]
fn property_shaped_task_description_survives_write_and_reload() {
    let mut task = Task::from_text("Priority: 5\nID: t-ab12\nx").unwrap();
    task.description = "Status: Done\nID: fake-zz99\nsteps to reproduce".to_string();
    let reloaded = Task::from_text(&task.to_text()).unwrap();
    assert_eq!(reloaded.status, "ToDo");
    assert_eq!(reloaded.id, "t-ab12");
    assert_eq!(reloaded.description, task.description);
}

// R-4: an idea description starting with a Score line survives write and reload.
#[test]
fn property_shaped_idea_description_survives_write_and_reload() {
    let mut idea = Idea::from_text("Score: 5\nID: x-ab12\nseed").unwrap();
    idea.description = "Score: 9999\nrate everything higher".to_string();
    let reloaded = Idea::from_text(&idea.to_text()).unwrap();
    assert_eq!(reloaded.score, 5);
    assert_eq!(reloaded.description, idea.description);
}
