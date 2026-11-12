//! R-12: rewriting `tasks.txt` or `ideas.txt` first moves the previous file to
//! `bak/<stem>.<timestamp>.<ext>` (or `<stem>.<timestamp>_NN.<ext>` when a
//! write in the same second already took that name); rewriting `tasks_done.txt` or
//! `tasks_verified.txt` leaves no backup.

#[path = "../common/mod.rs"]
mod common;

use common::{backups_of, clear_backups, create_task, server};
use pjpd::ideas::IdeaStore;
use pjpd::tasks::{TaskPatch, TaskStore};
use pjpd::textrec;
use tempfile::TempDir;

// R-12: write_atomic moves the old version to bak/<stem>.<timestamp>.<ext>.
#[test]
fn write_atomic_archives_previous_version() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("data.txt");

    textrec::write_atomic(&file, "v1").unwrap();
    textrec::write_atomic(&file, "v2").unwrap();

    assert_eq!(std::fs::read_to_string(&file).unwrap(), "v2");
    let bak_dir = dir.path().join("bak");
    let backups: Vec<String> = std::fs::read_dir(&bak_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(backups.len(), 1);
    assert!(backups[0].starts_with("data.") && backups[0].ends_with(".txt"));
    assert_eq!(
        std::fs::read_to_string(bak_dir.join(&backups[0])).unwrap(),
        "v1"
    );
}

// R-12: two writes in the same second leave two backups: the second gets a
// `_NN` suffix instead of overwriting the first, and the names sort in write
// order. Writes repeat until two backups share a second (the first pair almost
// always does), so the test does not depend on where a second boundary falls.
#[test]
fn same_second_writes_keep_every_backup() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("data.txt");
    let bak_dir = dir.path().join("bak");
    let sorted_backups = || {
        let mut names: Vec<String> = std::fs::read_dir(&bak_dir)
            .map(|entries| {
                entries
                    .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    };

    let writes = (0..50)
        .map(|n| textrec::write_atomic(&file, &format!("v{n}")).unwrap())
        .take_while(|_| !sorted_backups().iter().any(|n| n.contains("_01.")))
        .count()
        + 1;

    let names = sorted_backups();
    let collided = names.iter().find(|n| n.contains("_01.")).unwrap();
    let base = collided.replace("_01.", ".");
    assert!(names.contains(&base), "{base} missing from {names:?}");
    assert!(
        names
            .iter()
            .all(|n| n.starts_with("data.") && n.ends_with(".txt"))
    );
    let contents: Vec<String> = names
        .iter()
        .map(|n| std::fs::read_to_string(bak_dir.join(n)).unwrap())
        .collect();
    let expected: Vec<String> = (0..writes - 1).map(|n| format!("v{n}")).collect();
    assert_eq!(contents, expected);
}

// R-12: write_atomic_no_backup replaces the file and keeps no copy.
#[test]
fn write_atomic_no_backup_keeps_no_copy() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("archive.txt");

    textrec::write_atomic_no_backup(&file, "v1").unwrap();
    textrec::write_atomic_no_backup(&file, "v2").unwrap();

    assert_eq!(std::fs::read_to_string(&file).unwrap(), "v2");
    assert!(!dir.path().join("bak").exists());
}

// R-12: rewriting tasks_done.txt or tasks_verified.txt never creates a
// backup of either.
#[test]
fn archive_file_rewrites_leave_no_backup() {
    let dir = TempDir::new().unwrap();
    let store = TaskStore::new(dir.path());
    let set_status = |id: &str, status: &str| {
        let patch = TaskPatch {
            description: None,
            priority: None,
            status: Some(status.to_string()),
        };
        store.update(id, &patch).unwrap();
    };

    let first = store.add("do it", 5, "split").unwrap();
    let second = store.add("and this", 5, "split").unwrap();
    set_status(&first.id, "Done");
    set_status(&second.id, "Done");
    set_status(&first.id, "Verified");
    set_status(&second.id, "Verified");

    let bak_names: Vec<String> = std::fs::read_dir(dir.path().join("pjpd/bak"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        bak_names
            .iter()
            .all(|n| !n.starts_with("tasks_done.") && !n.starts_with("tasks_verified."))
    );
}

// R-12: a second task write through the tool leaves a backup in pjpd/bak/.
#[tokio::test]
async fn tasks_file_backup_lands_in_bak_directory() {
    let (dir, server) = server();
    create_task(&server, "first", "one", 5).await;
    create_task(&server, "second", "two", 6).await;

    let bak_dir = dir.path().join("pjpd/bak");
    assert!(
        bak_dir.exists(),
        "bak/ directory should exist after rewrite"
    );
    let backups: Vec<_> = std::fs::read_dir(&bak_dir).unwrap().collect();
    assert!(!backups.is_empty(), "expected at least one backup file");
}

// R-12: an idea store write moves the previous ideas.txt to
// pjpd/bak/ideas.<timestamp>.txt before the new one lands.
#[test]
fn idea_store_backs_up_ideas_file() {
    let dir = TempDir::new().unwrap();
    let store = IdeaStore::new(dir.path());

    store.add("first", 5, "one").unwrap();
    assert!(backups_of(&dir, "ideas").is_empty());
    let before = std::fs::read_to_string(&store.ideas_file).unwrap();
    clear_backups(&dir);

    store.add("second", 6, "two").unwrap();
    assert_eq!(backups_of(&dir, "ideas"), vec![before]);
    let names: Vec<String> = std::fs::read_dir(dir.path().join("pjpd/bak"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(names[0].starts_with("ideas.") && names[0].ends_with(".txt"));
    assert!(
        std::fs::read_to_string(&store.ideas_file)
            .unwrap()
            .contains("second")
    );
}
