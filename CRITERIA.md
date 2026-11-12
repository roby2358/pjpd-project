# pjpd — Criteria

How we know pjpd is correct. Each row has a stable ID that is never reused. The
Instrument column is the row's status: `automated`, `manual — <check>`,
`not yet covered`, or `retired`. Each `automated` R-row owns the directory
`rust/tests/R-n/` (D-19); the detail of each case lives in its tests. Constants are named
as in the ONTOLOGY Numbers table.

## R — Rules (automated)

| ID | Given | Expect | Instrument |
|---|---|---|---|
| R-1 | File text with separator lines of various shapes | Records split on lines of SEPARATOR_MIN_HYPHENS+ hyphens with optional trailing whitespace only; a line with leading whitespace or fewer hyphens is content; records trimmed, empty ones dropped; a missing file is no records | automated |
| R-2 | A task or idea rendered to text; several records joined | Header lines in the order the ONTOLOGY on-disk format gives, Priority/Score right-aligned to NUMBER_WIDTH, Created/Updated only when set, one blank line before a non-empty description; records joined with RECORD_SEPARATOR | automated |
| R-3 | Record text with a property header | Properties read only from the leading run of recognized lines, any order, keys case-insensitive and trimmed, later duplicate wins; a numeric key with a non-digit value or a value too large for i64 (kept verbatim; the largest i64 still parses), an unknown key, or a line without a colon ends the header and starts the description | automated |
| R-4 | Any record with an ID, including descriptions full of property-shaped lines | Render then parse yields an equal record; no description line becomes a property | automated |
| R-5 | A description containing separator-shaped lines enters a store (task add, task patch, idea add, idea update) | Each separator-shaped line is stored indented two spaces; other lines, already-indented lines, and short hyphen runs are unchanged; indenting twice changes nothing | automated |
| R-6 | A record missing its ID; a record with an old-form ID saved | An explicit non-empty ID is kept; else one is generated from `Tag:`; else an old-form `XX-XXXX-XX` ID is generated; an invalid `Tag:` drops the record; the tag is re-derived from the ID. Saving tasks (any file) or ideas rewrites each old-form ID, and only those, to a valid `legacy-XXXX` ID with tag `legacy`, the rest of the record unchanged | automated |
| R-7 | Tag and ID strings | Generated IDs are `<tag>-` plus ID_SUFFIX_LEN chars of ID_ALPHABET; the valid-ID check matches `^[a-zA-Z0-9\-]+-[a-z2-9]{4}$`; tags of 1..TAG_MAX_LEN chars of `[A-Za-z0-9-]` pass, others fail | automated |
| R-8 | A record with no Priority/Score or no Status line | Priority/Score is DEFAULT_LOADED_PRIORITY; task status is `ToDo` | automated |
| R-9 | A record is stamped (add, patch, idea update, idea mark done) | `created` set once, only if missing; `updated` refreshed to now; both ISO-8601 UTC | automated |
| R-10 | Tasks saved with mixed statuses and priorities | Status `Done` tasks in `tasks_done.txt`, status `Verified` tasks in `tasks_verified.txt` (status compared case-insensitively, spelling kept), all others (including unknown statuses) in `tasks.txt`, each sorted by priority desc; an archive file is written only if it has tasks or already exists; task load reads all three files | automated |
| R-11 | Ideas saved with mixed done state and scores | Active ideas before done ideas, each group by score desc; load returns the same order | automated |
| R-12 | `tasks.txt` or `ideas.txt` rewritten; `tasks_done.txt` or `tasks_verified.txt` rewritten | The previous main file moves to `bak/<stem>.<timestamp>.<ext>` before the new one lands; when a write in the same second already took that name, it moves to `bak/<stem>.<timestamp>_NN.<ext>` instead (first free NN from `01`), so two writes in one second leave two backups and names sort in write order; the archive files leave no backup | automated |
| R-13 | `pjpd_put_task` with `tag` | New task with a generated ID, given priority or DEFAULT_NEW_TASK_PRIORITY, status `ToDo`, both timestamps; `pjpd/` and `tasks.txt` created if missing | automated |
| R-14 | `pjpd_put_task` with `id` | Description replaced; priority replaced only when given; status unchanged; an unknown ID fails naming `pjpd_list_tasks` and changes nothing | automated |
| R-15 | Tool calls with bad arguments | Empty description, bad tag, bad ID, both or neither of tag/id, priority or score outside VALUE_MIN..VALUE_MAX, count outside COUNT_MIN..COUNT_MAX, an empty ID list, a `pjpd_mark_task` state other than `done`/`verified`/`idea`, or a `pjpd_list_tasks` status other than `todo`/`done`/`verified` each fail with a message and change nothing | automated |
| R-16 | `pjpd_list_tasks` | `status` (case-insensitive, default `todo`) lists exactly the tasks saved to that file: `todo` → `tasks.txt`, `done` → `tasks_done.txt`, `verified` → `tasks_verified.txt`; a hand-edited status is matched case-insensitively; sorted status `ToDo` (any case) before other statuses, then priority desc, then description lowercased; truncated to `count` with `info` naming returned and total; `total_tasks` is the number matching before truncation. By default each task is id, priority, status and `summary` (the description's first line, cut to SUMMARY_MAX_CHARS characters, `...` when text was left out), with no description or timestamps; with `verbose: true` the full record including `created` and `updated` | automated |
| R-17 | A task ID list (`pjpd_mark_task` in every state, `pjpd_reprioritize_task`) | Any unknown ID: fails naming the missing IDs and "No tasks were modified"; no task changes. All IDs known (states `done`, `verified`, and reprioritize): one save, leaving exactly one `tasks.txt` backup, holding the file as it was before the batch | automated |
| R-18 | `pjpd_mark_task` with state `done` and known IDs | Each task's status becomes `Done` and it moves to `tasks_done.txt`; listed under status `done` only; the state is matched case-insensitively | automated |
| R-19 | `pjpd_reprioritize_task` with known IDs | Only priority (and `updated`) change | automated |
| R-20 | `pjpd_put_idea` | `tag` creates an idea with the score; `id` replaces description and score; an unknown ID fails naming `pjpd_list_ideas` and changes nothing | automated |
| R-21 | `pjpd_list_ideas` | Ideas in save order (active first, score desc), truncated to `count` with `info` naming returned and total; `total_ideas` is the number before truncation. By default each idea is id, score and `summary` (as in R-16); with `verbose: true` the full record including `created` and `updated` | automated |
| R-22 | `pjpd_mark_idea_done` | First description line gets the `(Done) ` prefix once (never doubled), score unchanged; any unknown ID fails and no idea changes; several known IDs are one save, leaving exactly one `ideas.txt` backup, holding the file as it was before | automated |
| R-23 | `pjpd/<dir-name>.txt` exists | Task tool results include `warning`; idea tools never do; no warning when that file is `tasks.txt` itself | automated |
| R-24 | Any tool call | Response is `{success, result, error}`; every success result includes `project_file` with the full path of the file operated on; failures have `result: ""` | automated |
| R-25 | `pjpd_mark_task` with state `verified` and known IDs, from todo or done | Each task's status becomes `Verified` and it moves to `tasks_verified.txt`; listed under status `verified` only | automated |
| R-26 | `pjpd_mark_task` with state `idea` and known IDs, from any file | Each task's description becomes a new idea with a fresh ID from the task's tag and score DEMOTED_IDEA_SCORE, with no reference to the task; the tasks leave every task file; the result lists the new ideas (id and score) and names `ideas.txt` | automated |
| R-27 | A mutating tool (`pjpd_put_task`, `pjpd_mark_task` in every state, `pjpd_reprioritize_task`, `pjpd_put_idea`, `pjpd_mark_idea_done`) succeeds, with and without `verbose` | By default each task is reported by id, status and priority only and each idea by id and score only (no description or timestamps), besides `project_file`, `message` and any `warning`; with `verbose: true` the full records | automated |
| R-28 | The JSON schemas the tools advertise for their parameters | priority and score carry `minimum` VALUE_MIN and `maximum` VALUE_MAX; count `minimum` COUNT_MIN and `maximum` COUNT_MAX; tag `minLength` 1 and `maxLength` TAG_MAX_LEN; description `minLength` 1; ID lists `minItems` 1; the priority and score descriptions name VALUE_MAX as the upper bound and never call values outside 0–100 allowed | automated |

## U — Usability (manual)

| ID | Criterion | Instrument |
|---|---|---|
| U-1 | A person can read and hand-edit `tasks.txt` and `ideas.txt` in a plain editor, and the next tool call picks up the edit | manual — edit a record by hand, then list |
| U-2 | Every failure message names a concrete next step (which tool to call, what format an ID or tag takes) | manual — read each error string in `server.rs` and `validation.rs` |
| U-3 | Tool and parameter descriptions are enough for a model to use each tool correctly without the intro prompt | manual — a fresh agent session uses each tool once |

## X — Experience / outcomes

| ID | Criterion | Linked decision | Instrument |
|---|---|---|---|
| X-1 | An agent working the backlog one task at a time spends little of its context on pjpd responses. Fails if routine write calls echo full record text | D-14 | manual — work one task with an agent and read the pjpd write responses; R-27 pins the lean shape |
| X-2 | Real project data survives every release byte-identically on a load/save round trip with no edits. Fails if any record changes | D-2 | manual — round-trip a real `pjpd/` directory and diff |

## Status

**2026-11-12.** The criteria were written from the code when SPEC.md was converted. Every
`automated` row's tests live in `rust/tests/R-n/`, and `rust/tests/criteria_coverage.rs`
fails when an `automated` row has no tests there, a directory has no `automated` row, or
an Instrument cell is not an allowed value.

**Not yet covered:** none.
**Manual only:** U-1, U-2, U-3, X-1, X-2.
**Retired:** none.

**Partial coverage:** none.
