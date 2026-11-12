//! R-12: rewriting `tasks.txt` or `ideas.txt` first moves the previous file to
//! `bak/<stem>.<timestamp>.<ext>`; rewriting `tasks_done.txt` or
//! `tasks_verified.txt` leaves no backup.

#[path = "../common/mod.rs"]
mod common;

use common::{create_task, server};
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
