# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

pjpd is an MCP (Model Context Protocol) server for local project management, written in Rust (crate in `rust/`, built on the official `rmcp` SDK). It manages tasks and ideas for a single project rooted at the current working directory. All data lives under `<cwd>/pjpd/` as plain text files. It runs over stdio.

An earlier Python implementation is archived in `pjpd-python-src.zip` and in git history.

## Design Docs

The design lives in an ontology doc set at the repo root. Cite stable IDs (`G-n`, `D-n`, `R-n`) in code comments, tests and commits, never section numbers.

- **ONTOLOGY.md** — the implementation reference: purpose and goals, entities, invariants, on-disk format, rules as pseudocode, numbers, and the tool interface. Describes the current code only.
- **CRITERIA.md** — how each rule is checked (R/U/X rows); its Status section lists coverage gaps.
- **RATIONALE.md** — why each current decision was made (D-n), with alternatives.
- **ARCHITECTURE_LOG.md** — dated history, newest first; the only doc that says what things used to be.

When a rule, number or tool changes, update in the same change: the ONTOLOGY rule, the RATIONALE decision, the CRITERIA row and its tests, and a new ARCHITECTURE_LOG entry. Designs that aren't built yet stay in `pjpd/tasks.txt` until the change that builds them (D-20).

## Commands

```bash
cd rust
cargo test                 # Run all tests
cargo test --test R-10        # Run one rule's tests (rust/tests/R-10/)
cargo build --release      # Build binary at rust/target/release/pjpd
cargo fmt                  # Format
```

Run the server by executing `rust/target/release/pjpd` with the project root as its working directory.

## Architecture

**Entry point**: `rust/src/main.rs` — serves `PjpdServer` over stdio; `Path::cwd()` is the project root.

**`rust/src/server.rs`** is the MCP surface: parameter structs (serde + schemars) and all tools. Every tool returns `{"success": bool, "result": ..., "error": "..."}` as JSON text content via `mcp_success()`/`mcp_failure()` helpers. Every successful response includes a `project_file` property with the full path to the relevant data file.

**Single-project model**: The cwd is the project. There is one `pjpd/tasks.txt` file — no named projects, no project parameter on task tools.

**Module map** (all under `rust/src/`):
- `tasks.rs` — `Task` records + `TaskStore` backed by `pjpd/tasks.txt` (ToDo), `pjpd/tasks_done.txt` (Done archive) and `pjpd/tasks_verified.txt` (Verified archive). `Stage` maps a status to its file. `TaskPatch` expresses partial updates.
- `ideas.rs` — `Idea` records + `IdeaStore` backed by `pjpd/ideas.txt`.
- `record.rs` — the shared record shape: `Key: value` property lines + description, ID resolution, timestamps.
- `textrec.rs` — `----`-separated records in .txt files (reads 3+ hyphens for backward compat) and atomic writes (timestamped backup to `bak/`, then rename).
- `ids.rs` — tag-based ID generation (`<tag>-XXXX`, base32 alphabet a-z 2-9 excluding 1/l/o) and format checks.
- `validation.rs` — request rules mirroring the original Pydantic models.

**Stores reload from disk on every operation** — state always reflects current file contents.

**Storage layout on disk**:
```
<cwd>/pjpd/
├── tasks.txt          # ToDo tasks for the project
├── tasks_done.txt     # Done tasks (archive; not backed up to bak/)
├── tasks_verified.txt # Verified tasks (archive; not backed up to bak/)
├── ideas.txt          # All ideas
└── bak/               # Timestamped backups from atomic writes
```

## Key Design Decisions

The full reasoning is in RATIONALE.md; these are the ones most easily broken by accident.

- **CWD is the project root**, with no configuration and no multi-project support (D-1). Keep the command count small; extend an existing tool before adding one (D-18).
- **The on-disk text format is the cross-language contract** (D-2): `----` separators, `Priority:`/`Score:` padded to width 4, properties read only from the leading header run, a blank line before the description (D-9), separator-shaped description lines indented two spaces on write (D-10), the ID pattern, and the sort orders. The on-disk format section of ONTOLOGY states it exactly; the tests pin it.
- **Priority (tasks) and score (ideas) are deliberately distinct concepts** (D-4). Don't unify the fields or their vocabulary.
- **Marking tasks**: `pjpd_mark_task(task_ids, state)` (D-21). `done` sets status `Done` and moves the task to `tasks_done.txt`; `verified` sets `Verified` and moves it to `tasks_verified.txt` (D-6); `idea` turns the description into a new, unlinked idea and removes the task. Ideas get a `(Done)` prefix and keep their score (D-7).
- **Forgiving parser**: salvage malformed records; statuses stay open strings so unknown values survive a round trip (D-8).

## Development Preferences

- No one-time test scripts. Tests for a CRITERIA rule live in `rust/tests/R-n/` (one target per rule, each test asserting that one rule, shared setup in `rust/tests/common/mod.rs`); `rust/tests/criteria_coverage.rs` checks the tree against CRITERIA.md (D-19). In-module `#[cfg(test)]` tests are only for helpers that assert no rule; cross-rule walkthroughs go in `rust/tests/*.rs`
- Favor functional constructions over imperative
- No globals — use struct instance properties
- Keep metadata minimal
- Keep implementations simple; ask before extending functionality
- **Exception to "no defaults" rule**: `#[serde(default)]` values are acceptable on MCP parameter structs in `server.rs` so the calling model doesn't have to supply every argument. The no-defaults rule applies to internal function signatures, not the MCP API surface.
