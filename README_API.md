# ProjectMCP API Reference

This document provides the complete API reference for ProjectMCP's MCP tools and prompts.

## API Response Format

All tools return a consistent response structure:

```json
{
  "success": true,
  "result": { ... },
  "error": ""
}
```

On failure, `success` is `false`, `result` is `""`, and `error` contains an actionable message naming concrete next steps (e.g. which tool to call to recover, what format an ID should take).

Every successful result includes a `project_file` property with the full path to the data file being operated on. Task tool results may also include a `warning` property if a legacy project file is detected (see [Legacy Warning](#legacy-project-file-warning)).

> **Note**: Tag fields are used internally for ID generation but are not included in API responses. When creating new tasks or ideas, you must provide a tag, but the tag will not be returned in response data.

## MCP Tools

### Task Management

#### `pjpd_put_task`
Create or update a task. Provide `tag` to create a new task, or `id` to update an existing one. Exactly one of the two must be supplied.

**Parameters:**
- `description` (string, required): What needs to be done
- `tag` (string, conditional): Tag string (1-12 characters, alphanumeric and hyphens only). Provide to create a new task.
- `id` (string, conditional): Existing task ID (format: `<tag>-XXXX`). Provide to update.
- `priority` (integer, optional): Priority from 0 (negligible) to 100 (urgent); values up to 9999 are accepted for exceptional cases. Defaults to 50 when creating. When updating, omit to keep the existing priority.

#### `pjpd_list_tasks`
List the tasks in one status. Sorted with status `ToDo` before other statuses, then by priority descending, then by description.

**Parameters:**
- `count` (integer, optional): Maximum number of tasks to return (default: 20)
- `status` (string, optional): `todo` (default; tasks in `pjpd/tasks.txt`, including unknown statuses such as `Blocked`), `done` (`pjpd/tasks_done.txt`) or `verified` (`pjpd/tasks_verified.txt`). Case-insensitive.

#### `pjpd_mark_task`
Move one or more tasks to a new state. All IDs must exist or nothing is changed.

- `done`: an agent finished and committed the work. Status becomes `Done`; each task moves to `pjpd/tasks_done.txt`. Result: `tasks` (full task fields), `message`.
- `verified`: the user reviewed or tried the work. Allowed from done or straight from todo. Status becomes `Verified`; each task moves to `pjpd/tasks_verified.txt`. Result: `tasks`, `message`.
- `idea`: demote. Each task's description becomes a new idea (a fresh ID with the task's tag, score 50, no link back to the task) and the task is removed from the task files. Result: `ideas` (full idea fields), `project_file` naming `pjpd/ideas.txt`, `message`.

**Parameters:**
- `task_ids` (list of strings, required): Tag-based task IDs (format: `<tag>-XXXX`)
- `state` (string, required): `done`, `verified` or `idea`. Case-insensitive.

#### `pjpd_reprioritize_task`
Set the priority of one or more tasks without resending the description text. All IDs must exist or nothing is changed.

**Parameters:**
- `priority` (integer, required): New priority to apply to every listed task. 0 (negligible) to 100 (urgent); values up to 9999 are accepted for exceptional cases.
- `task_ids` (list of strings, required): Tag-based task IDs (format: `<tag>-XXXX`)

### Idea Management

#### `pjpd_put_idea`
Create or update an idea. Provide `tag` to create a new idea, or `id` to update an existing one. Exactly one of the two must be supplied.

**Parameters:**
- `score` (integer, required, also when updating): How promising the idea is, from 0 (trivial) to 100 (critical); values up to 9999 are accepted for exceptional cases.
- `description` (string, required): What the idea entails
- `tag` (string, conditional): Tag string (1-12 characters, alphanumeric and hyphens only). Provide to create a new idea.
- `id` (string, conditional): Existing idea ID (format: `<tag>-XXXX`). Provide to update.

#### `pjpd_list_ideas`
List ideas: active ideas first, then done ones, each by score (highest first).

**Parameters:**
- `count` (integer, optional): Maximum number of ideas to return (default: 20)

#### `pjpd_mark_idea_done`
Mark one or more ideas as done (description prefixed with `(Done)`; the score is kept). All IDs must exist or nothing is changed.

**Parameters:**
- `idea_ids` (list of strings, required): Tag-based idea IDs (format: `<tag>-XXXX`)

## MCP Prompts

### `pjpd_intro`
Return an introductory description of the ProjectMCP system and its tools.

**Parameters:** None

## Legacy Project File Warning

If pjpd detects a file at `pjpd/<directory-name>.txt` (from the old multi-project format where each project was a separate `.txt` file), task tool responses will include a `warning` property describing the legacy file and how to migrate. Tasks are now stored in `pjpd/tasks.txt`.
