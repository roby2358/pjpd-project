# pjpd — Architecture Log

Dated entries, newest first, one per structural or design change. This is the only doc
that records what things used to be. Entries before 2026-11-12 are reconstructed from
`git log` and the retired SPEC.md.

### 2026-11-12 — Write tools return lean results; `verbose` opts into full records
The five write tools (`pjpd_put_task`, `pjpd_mark_task`, `pjpd_reprioritize_task`,
`pjpd_put_idea`, `pjpd_mark_idea_done`) used to echo every touched record in full,
description and timestamps included. They now take `verbose` (default false) and by
default report each task by id, status and priority and each idea by id and score
(D-14, X-1). `pjpd_mark_idea_done` returned `idea_ids`; it now returns `ideas` in the same
summary shape. No tool was added (D-18). New row R-27; R-26 updated; X-1 moved from not
yet covered to manual. R-9, R-14, R-19 and R-26 tests adjusted to the lean default.

### 2026-11-12 — Task statuses compared case-insensitively
A task's stage used to come from an exact status match (`Done`, `Verified`), while the
list filter was case-insensitive, so a hand-edited `done` stayed in `tasks.txt` and listed
as todo. Now one case-insensitive comparison (`Stage::parse`) decides both the file a task
is saved in and the filter, and the `ToDo` sort rank also ignores case (D-6). Status text
is kept as written (D-8). R-10 and R-16 updated, each with a new test.

### 2026-11-12 — `pjpd_mark_task` replaces `pjpd_mark_task_done`; verified archive added
`pjpd_mark_task_done(task_ids)` became `pjpd_mark_task(task_ids, state)` with state `done`,
`verified` or `idea` (D-21); the tool count is unchanged. `verified` sets status
`Verified` and moves the task to the new `pjpd/tasks_verified.txt`, written without
backups like `tasks_done.txt` (D-6), which keeps its name. `idea` demotes: each task's
description becomes a new idea with a fresh ID from its tag and score DEMOTED_IDEA_SCORE,
and the task is removed. `pjpd_list_tasks` lost `show_done` (todo only, or everything) and
gained `status` (`todo` default, `done`, `verified`), which selects by file: a status other
than `Done`/`Verified`, such as `Blocked`, used to be hidden unless `show_done` and now
lists under `todo`. The status rank no longer orders Done after ToDo, since one listing
holds one stage. New rows R-25 (verified) and R-26 (demote); R-10, R-12, R-15, R-16, R-17
and R-18 updated. The SKILL.md interface now works as written.

### 2026-11-12 — Rule tests moved into `rust/tests/R-n/`
Finishes the ontology conversion (D-19); no behavior change. `rust/tests/tool_tests.rs`,
`rust/tests/store_tests.rs` and every in-module `#[cfg(test)]` block (textrec, record, ids,
tasks, ideas, validation; all asserted rules) are replaced by one test target per
`automated` CRITERIA row, `rust/tests/R-1/` to `rust/tests/R-24/`, with shared setup in
`rust/tests/common/mod.rs`. Tests that checked two rules were split: put_task creation
(R-13, R-9 timestamps, R-24 `project_file`), the done-file split (R-10, R-12 no backup),
idea sorting (R-11, R-22 score kept), put_idea plus list (R-20, R-21), and task parsing
(R-3, R-6 tag, R-8 default priority). R-8 gained a test for a record with no Priority or
Status line. `rust/tests/criteria_coverage.rs` now checks CRITERIA.md against the tree.
Tests went from 59 to 71. The CRITERIA partial-coverage list was re-checked against the
moved tests.

### 2026-11-12 — SPEC.md converted to the ontology doc set
ONTOLOGY.md, CRITERIA.md, RATIONALE.md and this log replace SPEC.md, which is deleted. The
new docs describe the code. SPEC.md, written for the Python implementation, disagreed with
the code in these places:

1. **Done archive.** SPEC said completed tasks must remain in `tasks.txt`; they move to
   `tasks_done.txt`, written without backups (D-6).
2. **Commands.** SPEC listed `get_statistics` (removed 2026-05-02) and an `intro` prompt;
   it omitted `reprioritize_task`. Tool and prompt names carry a `pjpd_` prefix.
3. **Separator lines in descriptions.** SPEC forbade them; the code indents them on write (D-10).
4. **Header parsing.** SPEC gave a recommended property order only. The code reads
   properties from the leading run in any order, case-insensitively, recognizes `Tag:` for
   ID salvage, and writes a blank line before the description (D-9).
5. **Malformed records.** SPEC said skip them with a warning; the code salvages them and
   drops a record only when its `Tag:` is invalid (D-8).
6. **Ranges.** SPEC said priority and score are 0–100 with values outside allowed; the
   code rejects values outside 0–9999 (D-5).
7. **Sorting.** SPEC said tasks are saved ToDo-first by priority; with the done split, each
   file is saved by priority alone. `pjpd_list_tasks` adds a description tiebreak and ranks
   statuses other than ToDo/Done last.
8. **Configuration.** SPEC required `projectmcp.toml` with `max_results`; the Rust port
   never built it (D-1).
9. **Python-only requirements** (fastmcp, PEP 8, `text_records.py`, configurable log
   level) and unmeasured timing targets (500 ms / 200 ms) were dropped (D-11).
10. **Stale reference docs.** README and README_API say `pjpd_mark_idea_done` zeroes the
    score (it keeps it, D-7) and still advertise `get_statistics` and the config file;
    `resources/intro.txt` does too. README described the Python layout and commands. All
    corrected; CLAUDE.md now points at the doc set and cites decision IDs.

Questions settled during the conversion: Q-1 (config file) by D-1; Q-2 (statistics tool)
by D-18; Q-3 (value range) by D-5; Q-4 (test layout) by D-19; Q-5 (timing targets) by
D-11; Q-6 (unbuilt designs) by D-20. Q-7 (bulk operations save once per ID) became a task.
New decisions D-1 through D-20 record the existing design with alternatives; criteria
R-1–R-24, U-1–U-3 and X-1–X-2 are new.

### 2026-08-13 — Separator-shaped description lines indented on write
A description line of three or more hyphens used to split its record on reload, spawning
a fragment with a fresh old-form ID on every load. Such lines are now indented two spaces
when a description enters a store (D-10). Write-side only.

### 2026-08-13 — Property injection closed on both sides of the parser
Properties used to be read from any `Key: value` line in a record, so description text
could override the header. The parser now reads only the leading run of property lines,
and the writer puts a blank line between header and description (D-9).

### 2026-08-13 — Priority and score recorded as distinct concepts
CLAUDE.md now states that task priority and idea score are deliberately different (D-4).

### 2026-08-13 — Ported to Rust; Python implementation retired
The crate in `rust/` (official `rmcp` SDK) replaced the Python server with the same tool
names, schemas, responses and file formats, verified byte-identical on real project data
(D-2). Two behaviors changed: updating a task without a priority keeps the existing one
instead of resetting it to 50 (D-3), and `mark_idea_done` is documented as keeping the score
(D-7). The `projectmcp.toml` config was not carried over (D-1). Python source archived in
`pjpd-python-src.zip`.

### 2026-07-07 — Tags describe the record, never the project
Tool parameter descriptions now tell the model that a tag names the task or idea itself,
since every record already lives inside one project (D-13).

### 2026-05-02 — Done tasks split into `tasks_done.txt`
Marking a task done used to leave it in `tasks.txt`; it now moves to `tasks_done.txt`, an
archive written without a backup (D-6).

### 2026-05-02 — `get_statistics` removed
The tool was unused; listing covers the need (D-18).

### 2026-05-01 — `reprioritize_task` added; `mark_done` renamed `mark_task_done`
`reprioritize_task` lets a model change priorities without resending descriptions. It
shares one all-or-nothing bulk path with `mark_task_done` (D-16), which was named
`mark_done` before.

### 2026-04-02 — Python implementation and SPEC.md uploaded
Single-project server rooted at the working directory (D-1), with plain-text task and idea
files (D-2) and a warning for files left from the earlier multi-project layout (D-17).
