//! R-11: ideas are saved and loaded active first, then done, each group by
//! score desc; an idea is done when its description starts with `(Done)`.

use pjpd::ideas::{Idea, IdeaStore};
use tempfile::TempDir;

// R-11: done-ness is read from the `(Done)` description prefix.
#[test]
fn done_detection_uses_description_prefix() {
    let done = Idea::from_text("Score: 5\nID: x-ab12\n(Done) finished").unwrap();
    assert!(done.is_done());
    let active = Idea::from_text("Score: 5\nID: x-ab12\ntry the thing").unwrap();
    assert!(!active.is_done());
}

// R-11: a done idea sorts after an active one even with a higher score.
#[test]
fn ideas_sort_active_before_done() {
    let dir = TempDir::new().unwrap();
    let store = IdeaStore::new(dir.path());

    let done = store.add("finished thing", 90, "olddone").unwrap();
    store.add("small", 10, "small").unwrap();
    store.mark_done(&done.id).unwrap();

    let ideas = store.load();
    assert_eq!(ideas[0].description, "small");
    assert!(ideas[1].is_done());
}
