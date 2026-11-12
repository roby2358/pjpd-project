//! Checks CRITERIA.md against the rule-test tree (D-19): every `automated`
//! row owns a `rust/tests/<ID>/` directory with at least one test, every such
//! directory belongs to an `automated` row, and every Instrument cell is one
//! of the allowed values.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// True for a criteria ID such as `R-1`, `U-3` or `X-2`.
fn is_criteria_id(text: &str) -> bool {
    text.split_once('-').is_some_and(|(kind, n)| {
        matches!(kind, "R" | "U" | "X") && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())
    })
}

/// Criteria ID -> Instrument cell (the last cell) for every table row whose
/// first cell is a criteria ID.
fn criteria_rows(markdown: &str) -> BTreeMap<String, String> {
    markdown
        .lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line
                .trim()
                .strip_prefix('|')?
                .strip_suffix('|')?
                .split('|')
                .map(str::trim)
                .collect();
            let id = cells.first().filter(|id| is_criteria_id(id))?;
            Some((id.to_string(), cells.last()?.to_string()))
        })
        .collect()
}

fn is_allowed_instrument(instrument: &str) -> bool {
    matches!(instrument, "automated" | "not yet covered" | "retired")
        || instrument
            .strip_prefix("manual — ")
            .is_some_and(|check| !check.trim().is_empty())
}

/// Names of the criteria-ID directories directly under rust/tests/.
fn rule_dirs(tests_dir: &Path) -> Vec<String> {
    fs::read_dir(tests_dir)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| is_criteria_id(name))
        .collect()
}

/// True when some .rs file in the directory (recursively) declares a test.
fn contains_test(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries.map(|entry| entry.unwrap().path()).any(|path| {
            if path.is_dir() {
                contains_test(&path)
            } else {
                path.extension().is_some_and(|ext| ext == "rs")
                    && fs::read_to_string(&path)
                        .is_ok_and(|src| src.contains("#[test]") || src.contains("#[tokio::test]"))
            }
        })
    })
}

fn load_rows() -> BTreeMap<String, String> {
    let criteria = fs::read_to_string(manifest_dir().join("../CRITERIA.md")).unwrap();
    let rows = criteria_rows(&criteria);
    assert!(!rows.is_empty(), "no criteria rows found in CRITERIA.md");
    rows
}

// Every Instrument cell is automated, manual — <check>, not yet covered or retired.
#[test]
fn every_instrument_is_allowed() {
    let bad: Vec<String> = load_rows()
        .into_iter()
        .filter(|(_, instrument)| !is_allowed_instrument(instrument))
        .map(|(id, instrument)| format!("{id}: {instrument}"))
        .collect();
    assert!(bad.is_empty(), "invalid Instrument cells: {bad:?}");
}

// Every automated row has rust/tests/<ID>/ containing at least one test.
#[test]
fn every_automated_row_has_tests() {
    let tests_dir = manifest_dir().join("tests");
    let missing: Vec<String> = load_rows()
        .into_iter()
        .filter(|(_, instrument)| instrument == "automated")
        .map(|(id, _)| id)
        .filter(|id| !contains_test(&tests_dir.join(id)))
        .collect();
    assert!(
        missing.is_empty(),
        "automated rows without tests in rust/tests/<ID>/: {missing:?}"
    );
}

// Every rust/tests/<ID>/ directory belongs to a row whose Instrument is automated.
#[test]
fn every_rule_dir_has_an_automated_row() {
    let rows = load_rows();
    let orphans: Vec<String> = rule_dirs(&manifest_dir().join("tests"))
        .into_iter()
        .filter(|name| rows.get(name).map(String::as_str) != Some("automated"))
        .collect();
    assert!(
        orphans.is_empty(),
        "test directories with no automated CRITERIA row: {orphans:?}"
    );
}

// The parser reads IDs and Instrument cells and ignores non-criteria rows.
#[test]
fn parses_criteria_rows() {
    let rows = criteria_rows(
        "| ID | Given | Instrument |\n|---|---|---|\n| R-1 | a | automated |\n| U-2 | b | manual — read it |\n",
    );
    assert_eq!(rows.len(), 2);
    assert_eq!(rows["R-1"], "automated");
    assert_eq!(rows["U-2"], "manual — read it");
}
