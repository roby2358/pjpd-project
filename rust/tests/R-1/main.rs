//! R-1: records split on separator lines of 3+ hyphens with optional trailing
//! whitespace only; records are trimmed and empty ones dropped; a missing
//! file is no records.

use pjpd::tasks::TaskStore;
use pjpd::textrec::{read_records, split_records};
use tempfile::TempDir;

// R-1: three, four and seven hyphens all separate records.
#[test]
fn splits_on_three_or_more_hyphens() {
    let content = "a\n---\nb\n----\nc\n-------\nd";
    assert_eq!(split_records(content), vec!["a", "b", "c", "d"]);
}

// R-1: trailing whitespace keeps a separator; leading whitespace makes it content.
#[test]
fn separator_allows_trailing_whitespace_only() {
    assert_eq!(split_records("a\n----   \nb"), vec!["a", "b"]);
    assert_eq!(split_records("a\n  ----\nb"), vec!["a\n  ----\nb"]);
}

// R-1: fewer than three hyphens is content.
#[test]
fn two_hyphens_is_not_a_separator() {
    assert_eq!(split_records("a\n--\nb"), vec!["a\n--\nb"]);
}

// R-1: blank records between separators are dropped.
#[test]
fn drops_empty_records() {
    assert_eq!(split_records("----\n\n----\nx\n----"), vec!["x"]);
}

// R-1: a missing file reads as no records, and a store over a project with
// no pjpd/ directory loads no tasks.
#[test]
fn missing_file_is_no_records() {
    let dir = TempDir::new().unwrap();
    assert!(read_records(&dir.path().join("pjpd/tasks.txt")).is_empty());
    assert!(TaskStore::new(dir.path()).load().is_empty());
}
