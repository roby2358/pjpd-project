//! Text files broken into records with `----` separators.
//! Reads SEPARATOR_MIN_HYPHENS+ hyphens for backward compatibility, writes 4 hyphens (asciidoc standard).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const RECORD_SEPARATOR: &str = "----";

/// SEPARATOR_MIN_HYPHENS: the fewest hyphens a line needs to read as a
/// separator (D-2).
pub const SEPARATOR_MIN_HYPHENS: usize = 3;

/// BACKUP_SUFFIX_WIDTH: digits in the `_NN` suffix of a backup name taken in
/// a second that already has one (D-12).
pub const BACKUP_SUFFIX_WIDTH: usize = 2;

/// A separator is a line of SEPARATOR_MIN_HYPHENS+ hyphens, optionally followed by trailing whitespace.
fn is_separator_line(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.len() >= SEPARATOR_MIN_HYPHENS && trimmed.bytes().all(|b| b == b'-')
}

/// Read a file and return its records (separated by SEPARATOR_MIN_HYPHENS+ hyphens on a line) as plain strings.
/// A missing or unreadable file yields an empty list.
pub fn read_records(file_path: &Path) -> Vec<String> {
    match fs::read_to_string(file_path) {
        Ok(content) => split_records(&content),
        Err(_) => Vec::new(),
    }
}

/// Split content on separator lines, trimming each record and dropping empty ones.
pub fn split_records(content: &str) -> Vec<String> {
    let mut records = Vec::new();
    let mut current = String::new();
    for line in content.lines() {
        if is_separator_line(line) {
            push_trimmed(&mut records, &current);
            current.clear();
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    push_trimmed(&mut records, &current);
    records
}

fn push_trimmed(records: &mut Vec<String>, text: &str) {
    let trimmed = text.trim();
    if !trimmed.is_empty() {
        records.push(trimmed.to_string());
    }
}

/// Indent any line that would read back as a record separator, so a hyphen
/// rule inside a description can't split its record on reload. Write-side
/// only, applied when a description enters a store: the indented form is the
/// stored form — nothing is unescaped on read.
pub fn indent_separator_lines(text: &str) -> String {
    text.lines()
        .map(|line| {
            if is_separator_line(line) {
                format!("  {line}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Join already-formatted record texts with the canonical record separator.
pub fn join_records<I>(record_texts: I) -> String
where
    I: IntoIterator<Item = String>,
{
    record_texts
        .into_iter()
        .collect::<Vec<_>>()
        .join(&format!("\n{RECORD_SEPARATOR}\n"))
}

/// Atomically write content, archiving the previous file into bak/ if one exists.
pub fn write_atomic(file_path: &Path, content: &str) -> io::Result<()> {
    archive(file_path)?;
    atomic_replace(file_path, content)
}

/// Atomically write content, replacing the previous file without keeping a copy.
pub fn write_atomic_no_backup(file_path: &Path, content: &str) -> io::Result<()> {
    atomic_replace(file_path, content)
}

fn timestamp() -> String {
    chrono::Local::now().format("%Y%m%d%H%M%S").to_string()
}

/// `tasks.txt` -> `tasks.20260813185300.txt`
fn timestamped_name(file_path: &Path) -> String {
    suffixed_name(file_path, &timestamp())
}

/// `tasks.txt` + `20260813185300_01` -> `tasks.20260813185300_01.txt`
fn suffixed_name(file_path: &Path, label: &str) -> String {
    let stem = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let suffix = file_path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    format!("{stem}.{label}{suffix}")
}

/// The first free backup path in `bak_dir`: `<stem>.<timestamp>.<ext>`, or
/// `<stem>.<timestamp>_NN.<ext>` (NN = 01, 02, ...) when a write in the same
/// second already took that name, so no backup is overwritten (D-12). `_`
/// sorts after `.`, so names sort in write order.
fn backup_path(bak_dir: &Path, file_path: &Path) -> PathBuf {
    let stamp = timestamp();
    std::iter::once(stamp.clone())
        .chain((1..).map(|n| format!("{stamp}_{n:0BACKUP_SUFFIX_WIDTH$}")))
        .map(|label| bak_dir.join(suffixed_name(file_path, &label)))
        .find(|path| !path.exists())
        .expect("an unbounded candidate sequence always has a free name")
}

fn parent_dir(file_path: &Path) -> PathBuf {
    file_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Move an existing file into a sibling bak/ directory with a timestamped name
/// that no earlier backup holds.
fn archive(file_path: &Path) -> io::Result<()> {
    if !file_path.exists() {
        return Ok(());
    }
    let bak_dir = parent_dir(file_path).join("bak");
    fs::create_dir_all(&bak_dir)?;
    fs::rename(file_path, backup_path(&bak_dir, file_path))
}

/// Write content to a temp sibling and atomically swap it into place.
fn atomic_replace(file_path: &Path, content: &str) -> io::Result<()> {
    fs::create_dir_all(parent_dir(file_path))?;
    let tmp_path = file_path.with_file_name(timestamped_name(file_path));
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, file_path)
}
