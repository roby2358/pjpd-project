---
name: pjpd-workflow
description: How to run a project's backlog with the pjpd MCP tools. Covers turning ideas into tasks by interviewing the user, working one task per session, marking tasks done and verified, and filing new tasks and ideas. Use when the user asks what's next, asks you to work the backlog or a task, wants to file or flesh out an idea, or mentions pjpd, tasks.txt or ideas.txt.
---

# pjpd workflow

pjpd keeps a project's ideas and tasks in plain text under `pjpd/` in the project root. You work with them through the `pjpd_*` tools. Work moves through four stages:

1. **Idea.** Something worth doing that isn't yet clear enough to do. You interview the user until it becomes one or more tasks.
2. **Task (todo).** Ready to work. You do the work.
3. **Done.** You finished and committed it. The user reviews it or tries it out.
4. **Verified.** The user confirmed it works. Nothing more to do.

Ideas and tasks are never linked. A task does not refer to the idea it came from, and an idea does not list its tasks.

## Ideas and tasks

A **task** is ready to work. It needs three things:

- **Goal:** what changes, and why.
- **Settled design:** every design call is already made. A task never says "decide whether..." or "consider...".
- **Done when:** a concrete check that tells you the work is finished, such as tests that pass or a behavior the user can see.

An **idea** is anything that lacks one of those. Ideas have a **score**, meaning how promising the idea is (0 to 100, higher is better). An idea may contradict the project's current design. That is fine. It only means the conflict gets settled before the idea becomes a task.

## Turning an idea into tasks

When the user asks you to flesh out an idea:

1. Read the idea with `pjpd_list_ideas`. Read the code and design docs the idea touches.
2. Ask the user questions until every design call is settled. Ask a few at a time, and offer a recommended answer for each.
3. When each task would have a goal, settled design and a done-when check, stop asking. File the tasks with `pjpd_put_task` (pass `tag`, `description` and `priority`). You don't need to show the user drafts first.
4. Close the idea with `pjpd_mark_idea_done`.

One idea can become several tasks. Make each task small enough for one agent session. Write each description so an agent with no memory of the interview can do the work from the description alone.

## Tags

Every new task or idea needs a tag. The ID becomes `<tag>-XXXX`.

- The tag describes the task or idea itself, never the project. All records already live in one project.
- 1 to 12 characters, letters, digits and hyphens only.
- Good: `donecase`, `listsummary`, `bulkatomic`. Bad: `pjpd`, `task1`, `fix`.

## Priorities

A task's priority is the order to do the work in, not how urgent it is. Higher numbers go first.

- Only implement tasks with priority 50 or above. Tasks below 50 are backlog: kept for later, not to be worked.
- A prerequisite ranks above the tasks that depend on it.
- File a task at 50 or above when it should be worked in turn. New tasks default to 50. File it below 50 when it belongs in the backlog.
- When you add a task, place it in the existing order. Use `pjpd_reprioritize_task` to move other tasks if needed. It changes priorities without resending descriptions.

## Working a task

Spawn an agent to work one task.

1. Take the highest-priority todo task from `pjpd_list_tasks`, and never take a task below priority 50, unless the user names one specifically.
2. Do the work the description asks for, and nothing more.
3. Run the done-when check.
4. Commit the work, following the project's commit rules.
5. Right after the commit, mark the task done with `pjpd_mark_task(task_ids, state: "done")`.
6. Report what you did, any judgment calls you made, and anything you filed.

### Open questions while working

Most questions that come up during a task are small enough to settle yourself. Make the call, then either:

- record the decision in the project's design docs (such as RATIONALE.md), or
- if it deserves more thought later, file it as an idea and carry on.

Rarely, you will hit a decision too far-reaching for you to make. Then demote the task with `pjpd_mark_task(task_ids, state: "idea")`. Its full text becomes a new idea and the task is removed. Use this very sparingly. Tell the user why you demoted it.

### Things outside the task

When you notice something outside the task's scope, don't do it. File it instead:

- A clear-cut bug or small fix with an obvious solution: file a task.
- Anything fuzzier: file an idea.

List what you filed in your report.

## Done and verified

- **Done** means an agent finished the work and committed it. You mark tasks done.
- **Verified** means the user reviewed the work or tried it out and is satisfied. Mark a task verified with `pjpd_mark_task(task_ids, state: "verified")` only when the user tells you to. Never mark verified on your own judgment. A task can go straight from todo to verified if the user says so.

If the user finds a problem with a done task, leave that task as it is. File a new task that describes what's wrong and what the fix should do.

To see tasks by stage, call `pjpd_list_tasks` with `status` set to `todo` (the default), `done` or `verified`.

## Tool summary

| Tool | Use it to |
|---|---|
| `pjpd_list_tasks` | See tasks, by status |
| `pjpd_put_task` | Create a task (`tag`) or rewrite one (`id`) |
| `pjpd_reprioritize_task` | Change the priority of one or more tasks |
| `pjpd_mark_task` | Move tasks to `done`, `verified` or `idea` |
| `pjpd_list_ideas` | See ideas, most promising first |
| `pjpd_put_idea` | Create an idea (`tag`) or rewrite one (`id`) |
| `pjpd_mark_idea_done` | Close an idea, such as after turning it into tasks |

Every tool returns `{success, result, error}`. On failure, the error says what to do next. Mark commands are all-or-nothing: if any ID is unknown, nothing changes.
