# ProjectMCP (pjpd)

A lightweight, local-first project manager for AI agents, served over the Model Context Protocol (MCP). pjpd keeps the tasks and ideas for one project as plain text files inside that project, rooted at the current working directory.

## Features

- **Local-first**: all data lives in plain text files under `<cwd>/pjpd/`
- **Single-project**: the working directory is the project; no configuration
- **Human-readable**: records you can read, hand-edit, diff and commit with your code
- **Priority-based tasks**: higher numbers are more urgent
- **Scored ideas**: track how promising an idea is before it becomes work
- **Forgiving**: hand edits and odd records survive a load/save round trip
- **Safe writes**: atomic file replacement with timestamped backups in `pjpd/bak/`

## Installation

### Prerequisites

- A Rust toolchain (stable), e.g. via [rustup](https://rustup.rs)

### Build from source

```bash
cd rust
cargo build --release      # binary at rust/target/release/pjpd
```

## Quick Start

### Add to an MCP client

pjpd works on whatever directory it is started in, so start it with your project as the working directory. For Claude Code, from the project root:

```bash
claude mcp add pjpd -- /path/to/pjpd-project/rust/target/release/pjpd
```

For other clients, configure a stdio server whose command is the `pjpd` binary and whose working directory is the project.

### Common operations

Using an MCP client, you can say:

**Tasks:**
- "Add a task: 'Design homepage mockup' with tag 'design' and priority 70"
- "List my tasks"
- "Show me completed tasks too"
- "Mark tasks 'design-a2c4' and 'bug-x3y5' as completed"
- "Bump 'design-a2c4' to priority 90"

**Ideas:**
- "Add a new idea: 'Implement dark mode' with score 80 and tag 'ui'"
- "List my top 10 ideas"
- "Update idea 'ui-a2c4' to have score 90"
- "Mark ideas 'ui-a2c4' and 'ai-bb12' as done"

## Data Format

All data is stored under `<cwd>/pjpd/`:

```
<cwd>/pjpd/
├── tasks.txt          # tasks still to do
├── tasks_done.txt     # done tasks, awaiting review (archive)
├── tasks_verified.txt # verified tasks (archive)
├── ideas.txt          # all ideas
└── bak/               # timestamped backups from atomic writes
```

Records are separated by a `----` line. Each record is a header of `Key: value` lines, a blank line, then a free-form description.

### Tasks

```
Priority:   80
Status: ToDo
ID: design-ab12
Created: 2026-03-28T20:00:00Z
Updated: 2026-03-28T20:00:00Z

Design the new homepage layout.
----
Priority:   10
Status: ToDo
ID: doc-3456
Created: 2026-03-20T14:30:00Z
Updated: 2026-03-25T09:15:00Z

Update documentation for the new API endpoints.
```

### Ideas

```
Score:   75
ID: ai-abcd
Created: 2026-03-28T20:00:00Z
Updated: 2026-03-28T20:00:00Z

Implement experimental AI-assisted code review workflow.
----
Score:    5
ID: ui-klmn
Created: 2026-03-28T20:00:00Z
Updated: 2026-03-28T20:00:00Z

(Done) Investigate alternative color palette for dark mode.
```

IDs are `<tag>-XXXX`: a tag of up to 12 letters, digits or hyphens that describes the record, plus four random characters. A closed idea keeps its score and gets a `(Done)` prefix.

The full format, rules and tool contract are in [ONTOLOGY.md](ONTOLOGY.md).

### Legacy project file warning

If pjpd finds `pjpd/<directory-name>.txt` (from an older multi-project layout), task tool responses include a `"warning"` property so you can migrate those tasks into `tasks.txt`.

## API Reference

See [README_API.md](README_API.md) for every MCP tool and prompt.

## Agent Workflow

[SKILL.md](SKILL.md) is a Claude Code skill (`pjpd-workflow`) that teaches agents the process: interview the user to turn ideas into tasks, work one task per session, mark it done after commit, and mark it verified once the user has checked it. Copy it to `~/.claude/skills/pjpd-workflow/SKILL.md` (or a project's `.claude/skills/pjpd-workflow/`) to use it.

## Usage Examples

### Tasks

1. **Create tasks** (provide `tag` to create):
   ```
   pjpd_put_task(description="Design new homepage layout", tag="design", priority=80)
   pjpd_put_task(description="Update contact form", tag="form")      # priority defaults to 50
   ```

2. **List tasks** by status (`todo` by default, `done` or `verified`), by priority descending:
   ```
   pjpd_list_tasks()
   pjpd_list_tasks(count=10)
   pjpd_list_tasks(status="done")
   ```

3. **Update a task** (provide `id`; omit `priority` to keep it):
   ```
   pjpd_put_task(id="design-ab12", description="Design new homepage layout (v2)")
   ```

4. **Reprioritize without resending the description:**
   ```
   pjpd_reprioritize_task(priority=90, task_ids=["design-ab12", "test-cd34"])
   ```

5. **Mark tasks** done, verified, or back to an idea (all-or-nothing):
   ```
   pjpd_mark_task(task_ids=["design-ab12", "form-cd34"], state="done")      # moves to tasks_done.txt
   pjpd_mark_task(task_ids=["design-ab12"], state="verified")               # moves to tasks_verified.txt
   pjpd_mark_task(task_ids=["form-cd34"], state="idea")                     # becomes a new idea; task removed
   ```

### Ideas

1. **Create ideas:**
   ```
   pjpd_put_idea(score=75, description="Implement AI-powered code review", tag="ai-review")
   ```

2. **Update an idea** (score and description are both required):
   ```
   pjpd_put_idea(id="ai-review-ab12", score=90, description="AI code review with inline suggestions")
   ```

3. **List ideas** (active first, highest score first):
   ```
   pjpd_list_ideas(count=5)
   ```

4. **Mark ideas done** (description prefixed with `(Done)`, score kept):
   ```
   pjpd_mark_idea_done(idea_ids=["ai-review-ab12"])
   ```

## Development

### Project structure

```
pjpd-project/
├── rust/
│   ├── src/
│   │   ├── main.rs          # Entry point: serves over stdio, cwd is the project
│   │   ├── server.rs        # MCP tools, parameters, response envelope
│   │   ├── tasks.rs         # Task records and store (tasks*.txt)
│   │   ├── ideas.rs         # Idea records and store (ideas.txt)
│   │   ├── record.rs        # Shared record shape: header parsing, IDs, timestamps
│   │   ├── textrec.rs       # ----separated records and atomic writes
│   │   ├── ids.rs           # Tag-based ID generation and checks
│   │   └── validation.rs    # Request validation
│   └── tests/
│       ├── R-1/ … R-24/      # One test target per CRITERIA rule (main.rs)
│       ├── common/mod.rs     # Shared setup for the rule tests
│       └── criteria_coverage.rs  # Checks CRITERIA.md against the R-n directories
├── resources/intro.txt      # Text of the pjpd_intro prompt
├── pjpd/                    # This project's own tasks and ideas
├── ONTOLOGY.md              # Implementation reference: entities, rules, interface
├── CRITERIA.md              # How each rule is checked
├── RATIONALE.md             # Why each design choice was made
└── ARCHITECTURE_LOG.md      # Dated history of design changes
```

### Running tests

```bash
cd rust
cargo test                       # all tests
cargo test --test R-10           # one rule's tests (rust/tests/R-10/)
cargo test --test criteria_coverage  # CRITERIA rows vs test directories
```

### Code quality

```bash
cd rust
cargo fmt
cargo clippy
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
