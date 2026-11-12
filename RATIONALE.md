# pjpd — Rationale

Why each current choice was made, what else was considered, and what's still open.
Decisions are cited by ID from code, tests and the other docs; IDs never move. History of
how decisions changed is in ARCHITECTURE_LOG.md.

## Scope

### D-1 The working directory is the project; one project, no configuration — *Decided*
- **Alternatives:** a configured projects directory holding many projects; a `projectmcp.toml` config file with options such as a result cap.
- **Why:** an MCP server is already launched per project by the client, so the working
  directory already names the project. Data inside the project's own tree gets committed
  and branched with it (G-2). A config file would add a second source of truth for very
  little (G-4); the obvious option, a result cap, is a tool parameter instead (D-15).
- **Evidence:** the project has run without a config file since the Rust port.

### D-18 Keep the command count small; extend before adding — *Decided*
- **Alternatives:** one tool per operation (separate get, delete, statistics, filter tools).
- **Why:** every tool costs the calling model context on every session (G-3, G-4). New
  capability goes into an existing command's parameters where it fits. There is no
  statistics tool: listing covers what the model needs.

## On-disk format

### D-2 Plain-text records are the contract — *Decided*
- **Alternatives:** JSON or YAML files; SQLite; one file per record.
- **Why:** plain text with `RECORD_SEPARATOR` lines is readable, hand-editable and diffable
  (G-2), and needs no tooling to inspect. Because the format outlives any implementation,
  its details are pinned by tests: separator (written as RECORD_SEPARATOR, read as
  SEPARATOR_MIN_HYPHENS or more hyphens so older files still load), NUMBER_WIDTH padding,
  property order, blank line before the description, ID pattern and sort orders.
- **Evidence:** `Measured by:` the Rust port was checked byte-identical against the
  Python implementation on real project data before the Python side was retired.

### D-9 Properties come only from the leading header run; a blank line ends it — *Decided*
- **Alternatives:** read `Key: value` lines from anywhere in the record; escape property-shaped description lines.
- **Why:** reading properties from anywhere would let a description line such as
  `Status: Done` silently override the real header (G-1). Parsing only the leading run
  closes that on read; writing a blank line between header and description closes it on
  write, so even a description that starts with a property-shaped line can't be absorbed.
  The parser trims the blank line back out. Any order is accepted within the run so hand
  edits don't have to follow the written order.

### D-10 Separator-shaped description lines are indented on write — *Decided*
- **Alternatives:** reject descriptions containing them; escape and unescape symmetrically.
- **Why:** a hyphen rule inside a description would otherwise split the record on the next
  load (G-1). Rejecting is unfriendly to models that write markdown. Indenting by two
  spaces is the smallest change that defuses the line, and keeping the indented form as
  the stored form (no unescape on read) means hand-edited files need no special handling.

### D-13 Tag-based IDs, unique by entropy — *Decided*
- **Alternatives:** sequential numbers; UUIDs; a registry checked for collisions.
- **Why:** `<tag>-XXXX` IDs describe themselves in a list or commit message (G-3) and need
  no coordination or counter file (G-4). ID_SUFFIX_LEN characters of ID_ALPHABET give 32⁴
  IDs per tag, plenty for one project's backlog. The alphabet drops `l`, `o`, `0` and `1` so
  IDs survive being read aloud or retyped. Tags describe the record, never the project
  name, since every record already lives inside one project.

## Tasks and ideas

### D-3 Priority is a plain integer; higher means more urgent — *Decided*
- **Alternatives:** named levels (high/medium/low); lower-is-more-urgent ranks.
- **Why:** integers let a model insert a task between two others without renumbering, and
  sort trivially. New tasks get DEFAULT_NEW_TASK_PRIORITY when none is given, the middle of
  the conventional scale. Updating a task without a priority keeps the existing one, so a
  description edit can't silently reprioritize.

### D-4 Priority and score are distinct concepts — *Decided*
- **Alternatives:** one shared "priority" field on both tasks and ideas.
- **Why:** score measures how promising an idea is; priority measures how urgently a task
  needs doing, and only exists once there's commitment to act. Ideas just are;
  prioritization happens when something graduates into a task. The fields, their names and
  their vocabulary stay separate.

### D-5 Priority and score are limited to VALUE_MIN..VALUE_MAX — *Decided*
- **Alternatives:** 0–100 strictly; unbounded integers.
- **Why:** 0–100 is the conventional scale the tool descriptions teach, but exceptional
  cases sometimes need room above it, and NUMBER_WIDTH padding holds four digits. Values
  outside VALUE_MIN..VALUE_MAX are rejected rather than silently clamped.

### D-6 Done and Verified tasks move to their own archive files, which have no backups — *Decided*
- **Alternatives:** keep finished tasks in `tasks.txt`; delete them; one archive holding
  both Done and Verified tasks.
- **Why:** an agent reads `tasks.txt` constantly; finished work there is noise and cost
  (G-3). Keeping it in separate archives preserves the record (G-1). Done tasks (in
  `tasks_done.txt`) wait for the user's review; Verified ones (in `tasks_verified.txt`) are
  finished for good, so one file per stage lets the user see the review queue at a glance
  and lets `pjpd_list_tasks` filter by file. The file is chosen by a case-insensitive status
  match, and the list filter uses the same rule, so every task lists under exactly one status
  value; an unknown status such as `Blocked` stays in `tasks.txt` and lists as todo. The
  archives only grow by tasks that already had backups in `tasks.txt`'s history, so
  backing them up on every write would multiply `bak/` for no safety gain.
- **Why case-insensitive:** a hand edit to `done` or `verified` plainly means Done or
  Verified; an exact match would leave it in `tasks.txt`, listed as todo, while the tool
  filter values were already case-insensitive (D-21). One comparison, `Stage::parse`,
  serves both the saved status and the filter. The status text is kept as written rather
  than normalized to `Done`/`Verified`, since statuses are open strings that round-trip
  unchanged (D-8). The same rule puts a `todo` status in the `ToDo` sort rank.
- **Why `tasks_verified.txt` rather than renaming the done file:** `tasks_done.txt` keeps its
  name so existing projects and older servers still read and write the same Done archive.

### D-7 An idea closes with a `(Done)` description prefix and keeps its score — *Decided*
- **Alternatives:** a Status line on ideas; zeroing the score; deleting closed ideas.
- **Why:** ideas are not actionable, so they have no workflow state to track. The prefix
  is visible in the file and in any listing, and keeping the score shows what the idea was
  worth even though it's off the books. Closed ideas sort after active ones (R-11).

### D-21 Marking a task takes a state: done, verified or idea — *Decided*
- **Alternatives:** separate tools per transition (`mark_task_done`, `mark_task_verified`,
  `demote_task`); a delete tool plus a manual `put_idea`.
- **Why:** one `pjpd_mark_task(task_ids, state)` replaces `pjpd_mark_task_done`, so the
  tool count stays the same (D-18). `done` means an agent finished and committed the
  work; `verified` means the user reviewed or tried it, and is allowed from any stage,
  including straight from todo, because the user may approve without an agent's done step.
  `idea` demotes a task that turned out not to be ready: its description becomes a new
  idea and the task is removed.
- **Demoted ideas:** the idea gets a fresh ID from the task's tag, not the task's ID, and
  no property pointing back, because tasks and ideas are never linked (ONTOLOGY
  non-goals). Its score is DEMOTED_IDEA_SCORE, the middle of the scale, rather than the
  task's priority: priority and score measure different things (D-4), and the idea is
  re-scored when someone looks at it. Ideas are saved before tasks are removed, so a
  failure in between leaves the text in both places rather than in neither (G-1).
- **State and status values** are matched case-insensitively, like the old `todo` filter,
  since a calling model may capitalize them.

### D-8 The parser is forgiving — *Decided*
- **Alternatives:** reject malformed records with an error; skip them with a warning.
- **Why:** a hand edit must never cost data (G-1). A missing ID is rebuilt from a `Tag:`
  line or generated in the old `XX-XXXX-XX` form; a missing priority or score becomes
  DEFAULT_LOADED_PRIORITY; statuses are open strings, so an unknown value like `Blocked`
  survives a load/save round trip. A record is dropped only when its `Tag:` is invalid and
  no ID can be built.
- **Known gaps (tracked as tasks):** an old-form ID fails the valid-ID check, so tools
  can't address that record; an oversized number is silently replaced with
  DEFAULT_LOADED_PRIORITY.

## Server behavior

### D-11 The server is stateless; every call rereads the files — *Decided*
- **Alternatives:** load once at startup and cache; watch files for changes.
- **Why:** people and other tools edit the files between calls (G-2), and stale state could
  overwrite their edits (G-1). Backlogs are small, so rereading costs nothing noticeable.
  There are no timing targets: none was ever measured, and the files are tiny.

### D-12 Writes are atomic with a timestamped backup — *Decided*
- **Alternatives:** write in place; rely on git for history.
- **Why:** a crash mid-write must leave either the old or the new file, never a torn one
  (G-1), so content goes to a temporary sibling that is renamed over the target. The
  previous version is moved into `bak/` first, which also recovers from a bad write by a
  model. Backups are not committed (`.gitignore`) and are never pruned automatically.

### D-14 One response envelope, always naming the file — *Decided*
- **Alternatives:** MCP-native errors; tool-specific result shapes.
- **Why:** a uniform `{success, result, error}` lets a model check one field for every
  tool (G-3). `project_file` tells the model and the person exactly which file changed, so
  either can open it. Errors name the next step, such as which tool lists valid IDs,
  because the reader is a model that will act on them.
- **Known gap (tracked as a task):** write tools echo the full record, which costs
  context in agent loops (X-1).

### D-15 List tools take a `count`, DEFAULT_COUNT by default, capped at COUNT_MAX — *Decided*
- **Alternatives:** return everything; a configured cap (D-1).
- **Why:** bounds the response size an agent pays for (G-3) without a config file.
  `pjpd_list_tasks` says when it truncated.

### D-16 ID lists are all-or-nothing — *Decided*
- **Alternatives:** apply to the IDs that exist and report the rest.
- **Why:** a typo in one ID shouldn't leave a batch half applied (G-1); the model fixes the
  list and retries. Every ID is checked before any change.
- **Known gap (tracked as a task):** each ID is then saved separately, so a disk error
  partway through can still leave a partial batch, and a batch leaves one backup per ID.

### D-17 Warn about the old multi-project file — *Decided*
- **Alternatives:** migrate it automatically; ignore it.
- **Why:** projects from the old layout have their tasks in `pjpd/<dir-name>.txt`. Moving
  them automatically risks clobbering `tasks.txt` (G-1); a warning on task tool results
  lets the person or model migrate deliberately.

## Process

### D-19 Rule tests live in `rust/tests/R-n/` — *Decided*
- **Alternatives:** keep tests in-module and tag them with `// R-n` comments for a
  coverage scan.
- **Why:** the directory is the label: a test's rule is visible from its path, and a
  coverage test can check every `automated` CRITERIA row against the tree. Cargo builds
  each `rust/tests/R-n/main.rs` as its own test target. In-module unit tests remain for
  helpers that assert no rule. Shared setup lives in `rust/tests/common/mod.rs`, which each
  target includes by `#[path]`. Tests of a request-validation function belong to the rule
  for bad tool arguments (R-15), not the ID-format rule (R-7), because `validation.rs` is
  the tool surface's check.

### D-20 Designs not yet built stay in the task list — *Decided*
- **Alternatives:** record them in ONTOLOGY as `[D, unbuilt]`.
- **Why:** pjpd's backlog already holds them in full, and an ONTOLOGY that describes only
  what the code does never has to be read with a list of exceptions in mind. A design
  enters ONTOLOGY in the same change that builds it.

## Open questions

None.

## Deliberately left out

- Configuration files (D-1).
- A statistics or reporting tool (D-18).
- Performance targets in milliseconds (D-11).
- Due dates, dependencies, hierarchy, and links between ideas and tasks (ONTOLOGY purpose, non-goals).
- Automatic pruning of `bak/`.

## How it's used / how it's abused

**Normal use.** An agent session in a project calls `pjpd_list_tasks`, picks the top task,
works it, and calls `pjpd_mark_task` with state `done`. The person reviews done work and
has it marked `verified`; a task that turns out not to be ready is marked `idea`. New work found along the way goes in with
`pjpd_put_task`; half-formed thoughts go in with `pjpd_put_idea`. The person reads and
edits the same files directly and commits them with the code.

| Abuse or accident | Prevention | Criteria |
|---|---|---|
| A description contains `Status: Done` or `ID: x-abcd` and overrides the header | Header read only from the leading run; blank line written before the description (D-9) | R-3, R-4 |
| A description contains a `----` line and splits the record | Separator-shaped lines indented on write (D-10) | R-1, R-5 |
| A hand edit breaks a record's header | Forgiving parser salvages ID and defaults (D-8) | R-6, R-8 |
| A crash or bad write destroys a file | Atomic rename plus backup in `bak/` (D-12) | R-12 |
| One mistyped ID in a batch half-applies it | All IDs checked before any change (D-16) | R-17, R-22 |
| A disk error partway through a batch half-applies it | Not prevented; tracked as a task (D-16) | — |
| A hand-edited number too large for 64 bits silently becomes the default | Not prevented; tracked as a task (D-8) | — |
