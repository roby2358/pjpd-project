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

// R-11: within each group ideas are by score desc, on disk and on load.
#[test]
fn ideas_sort_by_score_within_each_group() {
    let dir = TempDir::new().unwrap();
    let store = IdeaStore::new(dir.path());

    for (desc, score) in [("a10", 10), ("a90", 90), ("a50", 50)] {
        store.add(desc, score, desc).unwrap();
    }
    let done: Vec<String> = [("d20", 20), ("d80", 80), ("d5", 5)]
        .iter()
        .map(|(desc, score)| store.add(desc, *score, desc).unwrap().id)
        .collect();
    done.iter().for_each(|id| {
        store.mark_done(id).unwrap();
    });

    let expected = vec!["a90", "a50", "a10", "(Done) d80", "(Done) d20", "(Done) d5"];
    let loaded: Vec<String> = store.load().into_iter().map(|i| i.description).collect();
    assert_eq!(loaded, expected);

    let on_disk = std::fs::read_to_string(&store.ideas_file).unwrap();
    let positions: Vec<usize> = expected
        .iter()
        .map(|d| on_disk.find(&format!("\n\n{d}")).unwrap())
        .collect();
    assert!(positions.windows(2).all(|w| w[0] < w[1]), "{on_disk}");
}
