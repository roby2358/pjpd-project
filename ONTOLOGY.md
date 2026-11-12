# pjpd — Ontology

This is the implementation reference for pjpd: what exists, what each thing carries, the
rules, and the MCP interface. It describes the current code in `rust/`. CRITERIA.md says
how each rule is checked, RATIONALE.md says why each choice was made, and
ARCHITECTURE_LOG.md holds the history. Cite stable IDs (`G-n`, `D-n`, `R-n`), never
section numbers.

**Tags:** **[D]** decided by the designer · **[P]** proposed by Claude, pending review ·
**[T]** tunable number · **[D, unbuilt]** decided, not yet in the code ·
**Serves:** the goal or quality an item exists for.

---

## §0 Purpose [D]

pjpd gives an AI agent, and the person working with it, a small backlog of tasks and
ideas for one project, kept as plain text files inside that project and served over MCP
on stdio.

**Qualities**
- **Plain** — the data is text a person reads, edits, diffs and commits like any other file.
- **Forgiving** — hand edits and odd records survive a load/save round trip instead of being rejected.
- **Cheap** — an agent can use it in a loop without the tool surface or its responses costing much.

**Goals, in priority order**
- **G-1 Never lose data.** A write never destroys the previous state: writes are atomic,
  the main files keep timestamped backups, and the parser salvages rather than drops.
- **G-2 The text files are the contract.** The on-disk format is human-readable and stable
  across implementations; any port must read and write it byte-identically.
- **G-3 An agent can run the backlog.** Capture, list, reprioritize and close tasks and
  ideas through a handful of MCP tools with predictable responses and actionable errors.
- **G-4 Stay small.** One project (the working directory), no configuration, few
  commands; extend an existing command before adding one.

**When goals conflict:** G-1 > G-2 > G-3 > G-4.

**Non-goals (for now):** multiple projects or a projects directory; configuration files;
statistics or reporting tools; due dates and time tracking; dependencies, hierarchy, or
links between tasks and ideas; multi-user access or authentication; web UI or REST API;
databases or caching; file watching.

---

## §1 Entities

### Project
*Serves:* G-4. The current working directory at server start. All state lives on disk; the
server holds only paths.

| Field | Type | Notes |
|---|---|---|
| project_dir | path | The working directory when the server started |
| tasks_file | path | `<project_dir>/pjpd/tasks.txt` — tasks neither Done nor Verified |
| done_tasks_file | path | `<project_dir>/pjpd/tasks_done.txt` — Done tasks |
| verified_tasks_file | path | `<project_dir>/pjpd/tasks_verified.txt` — Verified tasks |
| ideas_file | path | `<project_dir>/pjpd/ideas.txt` — all ideas |
| bak dir | path | `<project_dir>/pjpd/bak/` — backups made by atomic writes |

### Task
*Serves:* G-3. Work there is commitment to do.

| Field | Type | Notes |
|---|---|---|
| id | string | `<tag>-XXXX`; a salvaged record may carry the old `XX-XXXX-XX` form until it is saved |
| tag | string | Derived from `id` at load; never written to disk or returned by tools |
| priority | integer | Higher = more urgent |
| status | string | Open string. `ToDo`, `Done` and `Verified` are the values the tools write; others survive round trips |
| description | string | Free text, may span lines; separator-shaped lines are stored indented |
| created | timestamp? | Set once, the first time the record is stamped |
| updated | timestamp? | Refreshed on every write of this record |

### Idea
*Serves:* G-3. Something that might become work; no commitment yet.

| Field | Type | Notes |
|---|---|---|
| id | string | Same format and salvage as Task.id |
| tag | string | Derived from `id`, as for Task |
| score | integer | How promising the idea is. Not a priority |
| description | string | As for Task. Starts with `(Done)` once the idea is closed |
| created | timestamp? | As for Task |
| updated | timestamp? | As for Task |

An idea has no status field. "Done" is derived: the description starts with `(Done)`.

### Stage
*Serves:* G-3. Which task file a task belongs to, derived from its status, compared
case-insensitively: `Done` → done, `Verified` → verified, anything else → todo (so a
hand-edited `done` is done). Saving and the
`pjpd_list_tasks` status filter both use it, so every task lists under exactly one filter.

### TaskPatch
*Serves:* G-3. A partial update to one task; absent fields stay unchanged.

| Field | Type | Notes |
|---|---|---|
| description | string? | Replaces the description |
| priority | integer? | Replaces the priority |
| status | string? | Replaces the status |

### Response
*Serves:* G-3. The JSON text every tool returns.

| Field | Type | Notes |
|---|---|---|
| success | bool | |
| result | object or `""` | `""` on failure. On success always includes `project_file` |
| error | string | `""` on success; an actionable message on failure |

---

## §2 Relations and invariants

- **Stateless server.** Every operation reloads the files it touches; nothing is cached
  between calls. Hand edits made between calls are always seen. *Serves:* G-1, G-2.
- **Stage split.** After any task write, `tasks_done.txt` holds exactly the tasks whose
  status is `Done`, `tasks_verified.txt` exactly those whose status is `Verified`, and
  `tasks.txt` all the rest (case-insensitive match; see Stage). Task load reads all three files.
  *Serves:* G-3.
- **Tasks and ideas are unlinked.** No record refers to another; demoting a task to an idea
  copies its description and leaves no cross-reference. *Serves:* G-4.
- **Round trip.** For any record with an ID, render then parse yields an equal record.
  *Serves:* G-1, G-2.
- **Header isolation.** No line of a description is ever read back as a property, whatever
  the description contains. *Serves:* G-1.
- **No separator in a stored description.** Once a description has entered a store, no
  line of it is separator-shaped. *Serves:* G-1.
- **Tag is derived.** A record's tag is always the one extracted from its ID; tags are not
  stored. *Serves:* G-2.
- **ID uniqueness by entropy.** IDs are random within a tag; nothing checks for collisions,
  and tasks and ideas share one ID space. *Serves:* G-4.
- **All-or-nothing ID lists.** A tool given a list of IDs changes nothing unless every ID
  exists. *Serves:* G-1, G-3.
- **stdout is MCP.** All logging goes to stderr. *Serves:* G-3.

---

## §3 On-disk format [D]

*Serves:* G-2. This is the cross-language contract; CRITERIA pins it.

**Layout**
```
<cwd>/pjpd/
├── tasks.txt          # tasks neither Done nor Verified
├── tasks_done.txt     # Done tasks (archive; no backups)
├── tasks_verified.txt # Verified tasks (archive; no backups)
├── ideas.txt          # all ideas
└── bak/               # timestamped backups from atomic writes
```
Files are UTF-8. `pjpd/` and `tasks.txt` are created on the first task add; `ideas.txt` on
the first idea save; `tasks_done.txt` and `tasks_verified.txt` the first time a save has
tasks of that stage.

**Records.** A file is a list of records separated by `RECORD_SEPARATOR` on its own line.
On read, any line of `SEPARATOR_MIN_HYPHENS` or more hyphens, optionally followed by
trailing whitespace, is a separator; a line with leading whitespace is not. Records are
trimmed; empty records are dropped.

**Record shape.** A header of `Key: value` property lines, a blank line, then the
description. Written in this order:

| Task | Idea |
|---|---|
| `Priority: ` value right-aligned to `NUMBER_WIDTH` | `Score: ` value right-aligned to `NUMBER_WIDTH` |
| `Status: <status>` | `ID: <id>` |
| `ID: <id>` | `Created: <timestamp>` (if set) |
| `Created: <timestamp>` (if set) | `Updated: <timestamp>` (if set) |
| `Updated: <timestamp>` (if set) | |

The blank line and description are omitted when the description is empty.

**Header parsing.** Properties are read only from the leading run of recognized property
lines, in any order. Keys match case-insensitively, keys and values are trimmed, and a
later duplicate in the run overrides an earlier one. Recognized keys:
- Task: `Status`, `ID`, `Tag`, `Created`, `Updated` (any value) and `Priority` (digits only, fitting a signed 64-bit integer).
- Idea: `ID`, `Tag`, `Created`, `Updated` (any value) and `Score` (digits only, fitting a signed 64-bit integer).

The first line that is not a recognized property (no colon, unknown key, or a numeric key
with a non-digit value or one too large for 64 bits) ends the header; it and everything after it are the description.
`Tag:` is read only to rebuild a missing ID and is never written.

**Timestamps.** ISO-8601 UTC, `YYYY-MM-DDTHH:MM:SSZ`. Missing timestamps are absent, not errors.

**IDs.** `<tag>-XXXX`: a tag of 1 to `TAG_MAX_LEN` ASCII letters, digits or hyphens, a
hyphen, and `ID_SUFFIX_LEN` characters from `ID_ALPHABET` (a–z and 2–9 without `l` and
`o`). A valid ID matches `^[a-zA-Z0-9\-]+-[a-z2-9]{4}$`. The old form `XX-XXXX-XX` is
generated only by salvage and rewritten to `legacy-XXXX` on save.

**Backups.** Before replacing `tasks.txt` or `ideas.txt`, the old file is moved to
`bak/<stem>.<YYYYMMDDHHMMSS>.<ext>` (local time). If a backup already has that name (a
write earlier in the same second), the name takes the first free suffix `_01`, `_02`, … (BACKUP_SUFFIX_WIDTH digits):
`bak/<stem>.<YYYYMMDDHHMMSS>_NN.<ext>`. No backup is overwritten, and names sort in write
order. `tasks_done.txt` and `tasks_verified.txt` are replaced without a backup.

---

## §4 Rules

### Split records
*Serves:* G-2. [D]
```
records = []; current = ""
for line in file.lines():
    if trim_end(line) is SEPARATOR_MIN_HYPHENS+ hyphens and nothing else: push(trim(current)); current = ""
    else: current += line + "\n"
push(trim(current))            # push drops empty strings
missing or unreadable file -> []
```

### Parse record
*Serves:* G-1, G-2. [D]
```
lines = trim(text).split("\n")
header = leading lines while line is "key: value" and key (lowercased) is recognized
         and (key not numeric or value is all digits and fits i64)
properties = header as map (later duplicate wins)
description = trim(join(lines[len(header):]))
priority | score = properties.numeric or DEFAULT_LOADED_PRIORITY
task.status = properties.status or "ToDo"
id = salvage_id(properties.id, properties.tag)   # record dropped if None
tag = tag_from_id(id)
```

### Salvage ID
*Serves:* G-1. [D]
```
salvage_id(explicit, tag):
    if explicit non-empty: return explicit
    if tag non-empty: return valid_tag(tag) ? tag + "-" + random(ID_SUFFIX_LEN) : None
    return random(2) + "-" + random(4) + "-" + random(2)       # old form
    log WARN to stderr whenever an ID is generated
tag_from_id(id): first hyphen-separated segment if id has a hyphen and the segment
                 is at most TAG_MAX_LEN chars, else "legacy"
readdress(record):                  # on every save, tasks and ideas
    if id is three hyphen-separated segments of 2, 4 and 2 ID_ALPHABET chars:
        id = "legacy-" + random(ID_SUFFIX_LEN); tag = "legacy"; log WARN to stderr
    nothing else changes (no stamp); no link to the old ID is kept
```

### Render record
*Serves:* G-2. [D]
```
lines = header lines in on-disk format order (Created/Updated only if set)
if trim(description) non-empty: lines += ["", trim(description)]
record text = join(lines, "\n")
file text = join(records, "\n" + RECORD_SEPARATOR + "\n")      # no trailing newline
```

### Indent separator lines
*Serves:* G-1. [D] Write-side only; nothing is unescaped on read.
```
when a description enters a store (task add, task patch, idea add, idea update):
    for each line that is a separator by the Split records test: line = "  " + line
```

### Stamp
*Serves:* G-3. [D]
```
stamp(record): if created empty: created = now_utc(); updated = now_utc()
called on: task add, task patch, idea add, idea update, idea mark done
```

### Atomic write
*Serves:* G-1. [D]
```
write_atomic(path, text):          # tasks.txt, ideas.txt
    if path exists: mkdir bak/; rename path -> backup_path(path)
    atomic_replace(path, text)
write_atomic_no_backup(path, text): atomic_replace(path, text)   # tasks_done.txt, tasks_verified.txt
atomic_replace(path, text): mkdir parent; write sibling temp file; rename over path
backup_path(path): first name not in bak/ among
    <stem>.<timestamp>.<ext>, <stem>.<timestamp>_01.<ext>, <stem>.<timestamp>_02.<ext>, ...
```

### Save tasks
*Serves:* G-1, G-3. [D]
```
readdress(task) for each task
stage(task) = lower(status) == "done" ? done : lower(status) == "verified" ? verified : todo
group(s) = tasks where stage == s, sorted by priority desc (stable)
write_atomic(tasks_file, render(group(todo)))
for (s, file) in [(done, done_tasks_file), (verified, verified_tasks_file)]:
    if group(s) non-empty or file exists: write_atomic_no_backup(file, render(group(s)))
```

### Save ideas
*Serves:* G-3. [D]
```
readdress(idea) for each idea
ideas sorted by (is_done, score desc), stable;  is_done = description starts with "(Done)"
write_atomic(ideas_file, render(ideas))
idea load returns the same order
```

### Create task
*Serves:* G-3. [D]
```
create_task(description, priority, tag):
    mkdir pjpd/; create empty tasks_file if missing
    task = {id: tag + "-" + random(ID_SUFFIX_LEN), priority, status: "ToDo",
            description: indent_separators(description)}; stamp(task)
    save_tasks(load_tasks() + [task])
```

### Patch task
*Serves:* G-3. [D]
```
patch_task(id, patch):
    tasks = load_tasks(); task = find(id) or return not_found
    apply set fields of patch (description through indent_separators); stamp(task)
    save_tasks(tasks)
```

### Bulk task update
*Serves:* G-1, G-3. [D]
```
bulk(ids, patch):
    missing = ids not found by get(id)
    if missing: fail "Tasks not found: [...]. No tasks were modified." ; no change
    tasks = load_tasks()
    for task in tasks whose id is in ids: apply set fields of patch; stamp(task)
    save_tasks(tasks)                           # one load and one save per batch
```

### Demote tasks
*Serves:* G-1, G-3. [D]
```
demote(ids):
    missing = ids not found in load_tasks()
    if missing: fail "Tasks not found: [...]. No tasks were modified." ; no change
    picked = tasks in load order whose id is in ids
    add ideas {id: task.tag + "-" + random(ID_SUFFIX_LEN), score: DEMOTED_IDEA_SCORE,
               description: indent_separators(task.description)} for each; stamp; one save_ideas
    then save_tasks(tasks without picked)       # ideas first: a failure between leaves a duplicate, never a loss
```

### Mark idea done
*Serves:* G-3. [D]
```
mark_ideas_done(ids):                           # after the all-or-nothing ID check
    ideas = load_ideas()
    for idea in ideas whose id is in ids:
        first, rest = split description at first newline
        first = starts_with(first, "(Done)") ? first : (first empty ? "(Done)" : "(Done) " + first)
        description = rest non-empty ? first + "\n" + rest : first
        score unchanged; stamp(idea)
    save_ideas(ideas)                           # one load and one save per batch
```

### Legacy warning
*Serves:* G-3. [D] Migration aid for the old multi-project layout.
```
name = last component of project_dir
if pjpd/<name>.txt exists and is not tasks_file:
    task tool results include "warning": "Legacy project file '<name>.txt' found ..."
```

### Request validation
*Serves:* G-3. [D] Runs before any store is touched; a violation fails the call with no change.
```
description: non-empty                         (put_task, put_idea)
tag: 1..TAG_MAX_LEN chars of [A-Za-z0-9-]
id: matches the valid-ID pattern
exactly one of tag, id                         (put_task, put_idea)
priority, score: VALUE_MIN..VALUE_MAX         (score required on every put_idea)
count: COUNT_MIN..COUNT_MAX
id lists: non-empty, every id valid
state: done | verified | idea, case-insensitive      (mark_task)
status: todo | done | verified, case-insensitive     (list_tasks)
```
The tool parameter JSON schemas advertise the range and length bounds above (`minimum`/
`maximum`, `minLength`/`maxLength`, `minItems`) so a caller sees them before calling (D-22).

---

## §5 Agents

The only actor is the calling model, through the tools listed under Commands. It learns how to use pjpd from
the tool and parameter descriptions and the `pjpd_intro` prompt. There is no background job.

---

## §6 Numbers [T]

Every constant, by name. The rest of the doc set refers to these names, never their values.
Each is one named constant in the code, and every use refers to it.

| Name | Value | Unit | Code | Decision |
|---|---|---|---|---|
| DEFAULT_NEW_TASK_PRIORITY | 50 | priority | `server::DEFAULT_NEW_TASK_PRIORITY` | D-3 |
| DEMOTED_IDEA_SCORE | 50 | score | `server::DEMOTED_IDEA_SCORE` | D-21 |
| DEFAULT_LOADED_PRIORITY | 1 | priority / score | `record::DEFAULT_LOADED_PRIORITY` | D-8 |
| NUMBER_WIDTH | 4 | characters | `record::NUMBER_WIDTH` | D-2 |
| VALUE_MIN, VALUE_MAX | 0, 9999 | priority / score | `server::VALUE_MIN`, `server::VALUE_MAX` | D-5 |
| COUNT_MIN, COUNT_MAX | 1, 1000 | records | `server::COUNT_MIN`, `server::COUNT_MAX` | D-15 |
| DEFAULT_COUNT | 20 | records | `server::DEFAULT_COUNT` | D-15 |
| SUMMARY_MAX_CHARS | 100 | characters | `server::SUMMARY_MAX_CHARS` | D-15 |
| TAG_MAX_LEN | 12 | characters | `ids::TAG_MAX_LEN` | D-13 |
| ID_SUFFIX_LEN | 4 | characters | `ids::ID_SUFFIX_LEN` | D-13 |
| ID_ALPHABET | `abcdefghijkmnpqrstuvwxyz23456789` | — | `ids::ID_ALPHABET` | D-13 |
| RECORD_SEPARATOR | `----` | — | `textrec::RECORD_SEPARATOR` | D-2 |
| SEPARATOR_MIN_HYPHENS | 3 | hyphens | `textrec::SEPARATOR_MIN_HYPHENS` | D-2 |
| BACKUP_SUFFIX_WIDTH | 2 | digits | `textrec::BACKUP_SUFFIX_WIDTH` | D-12 |

---

## §7 Commands (the interface)

pjpd is an MCP server on stdio (server name `projectmcp`). Every tool returns a Response
(see Entities) as JSON text. A legal call returns `success: true` with a result that includes
`project_file`, the full path of the file operated on. Task tools also include `warning`
when the Legacy warning rule fires. The write tools (every tool but the two list tools)
take `verbose` (default false): by default they report each task they touched as its
task summary (id, status, priority) and each idea as its idea summary (id, score), with no
description or timestamps; `verbose: true` returns the full fields instead (D-14). The
two list tools also take `verbose` (default false): by default each record is a listing,
the summary fields plus `summary`, the description's first line (trimmed) cut to
SUMMARY_MAX_CHARS characters with `...` appended when any text was left out; `verbose:
true` returns the full fields (D-15). A
call that fails validation or names unknown IDs returns `success: false`, `result: ""`,
and an `error` naming the next step (which tool to call, what format to use); nothing
changes. Unexpected errors are wrapped as
`"Error <doing X>: <detail>"`.

### Tools

| Command (args) | Legal when | Effect | Success result |
|---|---|---|---|
| `pjpd_put_task(description, tag?, id?, priority?, verbose = false)` | Validation passes; with `id`, the task exists | `tag` → Create task with `priority` or DEFAULT_NEW_TASK_PRIORITY. `id` → Patch task: description replaced, priority replaced only if given | Task summary fields (id, status, priority), or with `verbose` the full task fields (id, priority, status, description, created, updated); `message` |
| `pjpd_list_tasks(count = DEFAULT_COUNT, status = "todo", verbose = false)` | count in range; status is `todo`, `done` or `verified` | None. Filter: tasks whose Stage equals `status`. Sort: status `ToDo` (any case) before other statuses; priority desc; description lowercased. Truncate to `count` | `tasks` (id, priority, status, summary; full task fields with `verbose`), `total_tasks` (number matching the filter, before truncation), `info` when truncated |
| `pjpd_mark_task(task_ids, state, verbose = false)` | state is `done`, `verified` or `idea`; every ID valid and existing | `done` → Bulk task update with status `Done` (moves each to `tasks_done.txt`). `verified` → Bulk task update with status `Verified` (moves each to `tasks_verified.txt`), from any stage. `idea` → Demote tasks | `done`/`verified`: `tasks` (task summaries; full task fields with `verbose`), `message`. `idea`: `ideas` (idea summaries; full idea fields with `verbose`), `project_file` naming `ideas.txt`, `message` |
| `pjpd_reprioritize_task(priority, task_ids, verbose = false)` | priority in range; every ID valid and existing | Bulk task update with the priority | `tasks` (task summaries; full task fields with `verbose`), `message` |
| `pjpd_put_idea(score, description, tag?, id?, verbose = false)` | Validation passes; with `id`, the idea exists | `tag` → create idea. `id` → replace description and score | Idea summary fields (id, score), or with `verbose` the full idea fields (id, score, description, created, updated); `message` |
| `pjpd_list_ideas(count = DEFAULT_COUNT, verbose = false)` | count in range | None. Save-ideas order, truncated to `count` | `ideas` (id, score, summary; full idea fields with `verbose`), `total_ideas` (number of ideas, before truncation), `info` when truncated, `message` |
| `pjpd_mark_idea_done(idea_ids, verbose = false)` | Every ID valid and existing | Mark idea done for all, in one save | `ideas` (idea summaries in save order; full idea fields with `verbose`), `message` |

### Prompts

| Prompt | Returns |
|---|---|
| `pjpd_intro` | `resources/intro.txt`: what pjpd is and which tools exist |

### Events

None. pjpd has one client per process and no push channel; a client sees changes by
calling a list tool.

### Client contract

The client reaches pjpd only through the tools and prompt above. People and other tools
may edit the files between calls; the server rereads them every time.
