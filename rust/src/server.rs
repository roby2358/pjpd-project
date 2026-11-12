//! MCP server surface: tool and prompt definitions.
//!
//! Every tool returns `{"success": bool, "result": ..., "error": "..."}` as
//! JSON text content. Every successful response includes a `project_file`
//! property with the full path to the relevant data file.

use std::path::{Path, PathBuf};

use rmcp::handler::server::router::{prompt::PromptRouter, tool::ToolRouter};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, PromptMessage, Role, ServerCapabilities,
    ServerInfo,
};
use rmcp::{ServerHandler, prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::ideas::{Idea, IdeaStore};
use crate::tasks::{STATUS_DONE, STATUS_TODO, STATUS_VERIFIED, Stage, Task, TaskPatch, TaskStore};
use crate::{ids, validation};

const INTRO_TEXT: &str = include_str!("../../resources/intro.txt");

/// Priority assigned to newly created tasks when the caller omits one (D-3).
pub const DEFAULT_NEW_TASK_PRIORITY: i64 = 50;

/// Score given to an idea made by demoting a task (D-21). The task's priority
/// is not carried over: priority and score are distinct (D-4).
pub const DEMOTED_IDEA_SCORE: i64 = 50;

/// Most characters of a description's first line that a list summary shows
/// (D-15).
pub const SUMMARY_MAX_CHARS: usize = 100;

/// VALUE_MIN, VALUE_MAX: the priority and score range a request may set (D-5).
pub const VALUE_MIN: i64 = 0;
pub const VALUE_MAX: i64 = 9999;

/// COUNT_MIN, COUNT_MAX: the `count` range a list request may ask for (D-15).
pub const COUNT_MIN: i64 = 1;
pub const COUNT_MAX: i64 = 1000;

/// DEFAULT_COUNT: records a list returns when `count` is omitted (D-15).
pub const DEFAULT_COUNT: i64 = 20;

fn default_count() -> i64 {
    DEFAULT_COUNT
}

fn default_status() -> String {
    "todo".to_string()
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PutTaskParams {
    /// What needs to be done, in enough detail to act on without additional
    /// context. Self-contained: do not reference other task IDs.
    #[schemars(length(min = 1))]
    pub description: String,
    /// Tag string (1-12 characters, alphanumeric and hyphens only). Provide to
    /// create a new task. Must describe the task itself (e.g. "oplock",
    /// "gitsemantics"), NEVER the project name — tasks already live inside a
    /// project, and the tag makes the task ID self-describing.
    #[serde(default)]
    #[schemars(length(min = 1, max = ids::TAG_MAX_LEN))]
    pub tag: Option<String>,
    /// Existing task ID (format: `<tag>-XXXX`). Provide to update an existing task.
    #[serde(default)]
    pub id: Option<String>,
    /// Priority from 0 (negligible) to 100 (urgent); values up to 9999 are
    /// accepted for exceptional cases. Defaults to 50 when creating a new
    /// task. When updating, omit to preserve the task's existing priority.
    #[serde(default)]
    #[schemars(range(min = VALUE_MIN, max = VALUE_MAX))]
    pub priority: Option<i64>,
    /// Return the full record(s), description and timestamps included.
    /// Defaults to false: the result gives each task's id, status and
    /// priority only, to keep responses short.
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListTasksParams {
    /// Maximum number of tasks to return, 1 to 1000. Defaults to 20.
    #[serde(default = "default_count")]
    #[schemars(range(min = COUNT_MIN, max = COUNT_MAX))]
    pub count: i64,
    /// Which tasks to list: "todo" (not yet done, the default), "done"
    /// (implemented, awaiting review) or "verified" (reviewed by the user).
    #[serde(default = "default_status")]
    pub status: String,
    /// Return the full records, description and timestamps included.
    /// Defaults to false: each task gives its id, priority, status and a
    /// `summary` (the description's first line, cut to 100 characters), to
    /// keep listings short.
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MarkTaskParams {
    /// List of tag-based task IDs (format: `<tag>-XXXX`) to mark.
    #[schemars(length(min = 1))]
    pub task_ids: Vec<String>,
    /// "done" (an agent finished and committed the work; moves to
    /// tasks_done.txt), "verified" (the user reviewed or tried it, from done
    /// or straight from todo; moves to tasks_verified.txt), or "idea" (demote:
    /// each task's description becomes a new idea and the task is removed).
    pub state: String,
    /// Return the full record(s), description and timestamps included.
    /// Defaults to false: the result gives each task's id, status and
    /// priority only, to keep responses short.
    #[serde(default)]
    pub verbose: bool,
}

/// The target state of `pjpd_mark_task`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkState {
    Done,
    Verified,
    Idea,
}

impl MarkState {
    fn parse(value: &str) -> Result<MarkState, String> {
        [
            ("done", MarkState::Done),
            ("verified", MarkState::Verified),
            ("idea", MarkState::Idea),
        ]
        .into_iter()
        .find(|(name, _)| value.eq_ignore_ascii_case(name))
        .map(|(_, state)| state)
        .ok_or_else(|| format!("state must be one of 'done', 'verified' or 'idea' (got '{value}')"))
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ReprioritizeTaskParams {
    /// New priority to apply to every listed task. 0 (negligible) to 100
    /// (urgent); values up to 9999 are accepted for exceptional cases.
    #[schemars(range(min = VALUE_MIN, max = VALUE_MAX))]
    pub priority: i64,
    /// List of tag-based task IDs (format: `<tag>-XXXX`) to reprioritize.
    #[schemars(length(min = 1))]
    pub task_ids: Vec<String>,
    /// Return the full record(s), description and timestamps included.
    /// Defaults to false: the result gives each task's id, status and
    /// priority only, to keep responses short.
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListIdeasParams {
    /// Maximum number of ideas to return, 1 to 1000. Defaults to 20.
    #[serde(default = "default_count")]
    #[schemars(range(min = COUNT_MIN, max = COUNT_MAX))]
    pub count: i64,
    /// Return the full records, description and timestamps included.
    /// Defaults to false: each idea gives its id, score and a `summary` (the
    /// description's first line, cut to 100 characters), to keep listings
    /// short.
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PutIdeaParams {
    /// Relevance score from 0 (trivial) to 100 (critical); values up to 9999
    /// are accepted for exceptional cases.
    #[schemars(range(min = VALUE_MIN, max = VALUE_MAX))]
    pub score: i64,
    /// What the idea entails, in enough detail for a human or model to
    /// understand its purpose and scope.
    #[schemars(length(min = 1))]
    pub description: String,
    /// Tag string (1-12 characters, alphanumeric and hyphens only). Provide to
    /// create a new idea. Must describe the idea itself, NEVER the project
    /// name — ideas already live inside a project, and the tag makes the idea
    /// ID self-describing.
    #[serde(default)]
    #[schemars(length(min = 1, max = ids::TAG_MAX_LEN))]
    pub tag: Option<String>,
    /// Existing idea ID (format: `<tag>-XXXX`). Provide to update an existing idea.
    #[serde(default)]
    pub id: Option<String>,
    /// Return the full record(s), description and timestamps included.
    /// Defaults to false: the result gives each idea's id and score only, to
    /// keep responses short.
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MarkIdeaDoneParams {
    /// List of tag-based idea IDs (format: `<tag>-XXXX`) to mark as done.
    #[schemars(length(min = 1))]
    pub idea_ids: Vec<String>,
    /// Return the full record(s), description and timestamps included.
    /// Defaults to false: the result gives each idea's id and score only, to
    /// keep responses short.
    #[serde(default)]
    pub verbose: bool,
}

fn mcp_success(result: Value) -> CallToolResult {
    to_tool_result(json!({"success": true, "result": result, "error": ""}))
}

fn mcp_failure(error_message: &str) -> CallToolResult {
    to_tool_result(json!({"success": false, "result": "", "error": error_message}))
}

fn to_tool_result(response: Value) -> CallToolResult {
    let text = serde_json::to_string_pretty(&response).unwrap_or_else(|e| {
        format!("{{\"success\": false, \"result\": \"\", \"error\": \"serialization error: {e}\"}}")
    });
    CallToolResult::success(vec![ContentBlock::text(text)])
}

/// A task as a mutating tool reports it (D-14): the full record when
/// `verbose`, else only id, status and priority.
fn task_summary(task: &Task, verbose: bool) -> Value {
    if verbose {
        task.to_json()
    } else {
        json!({"id": task.id, "status": task.status, "priority": task.priority})
    }
}

/// An idea as a mutating tool reports it (D-14): the full record when
/// `verbose`, else only id and score.
fn idea_summary(idea: &Idea, verbose: bool) -> Value {
    if verbose {
        idea.to_json()
    } else {
        json!({"id": idea.id, "score": idea.score})
    }
}

/// The first line of a description, cut to SUMMARY_MAX_CHARS characters, with
/// "..." appended when any text was left out (D-15).
fn description_summary(description: &str) -> String {
    let trimmed = description.trim();
    let first_line = trimmed.lines().next().unwrap_or("");
    let shown: String = first_line.chars().take(SUMMARY_MAX_CHARS).collect();
    if shown.len() < trimmed.len() {
        format!("{}...", shown.trim_end())
    } else {
        shown
    }
}

/// A task as a list tool reports it (D-15): the full record when `verbose`,
/// else id, priority, status and the description summary.
fn task_listing(task: &Task, verbose: bool) -> Value {
    if verbose {
        task.to_json()
    } else {
        json!({
            "id": task.id,
            "priority": task.priority,
            "status": task.status,
            "summary": description_summary(&task.description),
        })
    }
}

/// An idea as a list tool reports it (D-15): the full record when `verbose`,
/// else id, score and the description summary.
fn idea_listing(idea: &Idea, verbose: bool) -> Value {
    if verbose {
        idea.to_json()
    } else {
        json!({
            "id": idea.id,
            "score": idea.score,
            "summary": description_summary(&idea.description),
        })
    }
}

/// Render a string list the way Python's str(list) does: ['a', 'b'].
fn python_list_repr(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| format!("'{s}'")).collect();
    format!("[{}]", quoted.join(", "))
}

pub struct PjpdServer {
    project_dir: PathBuf,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

impl PjpdServer {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            project_dir,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    fn tasks(&self) -> TaskStore {
        TaskStore::new(&self.project_dir)
    }

    fn ideas(&self) -> IdeaStore {
        IdeaStore::new(&self.project_dir)
    }

    /// Add the legacy project file warning to a result map if applicable.
    fn with_warning(&self, mut result: Map<String, Value>) -> Map<String, Value> {
        if let Some(warning) = self.tasks().legacy_warning() {
            result.insert("warning".to_string(), json!(warning));
        }
        result
    }

    fn task_response(&self, task: &Task, verbose: bool, message: &str) -> CallToolResult {
        let result = record_map(
            task_summary(task, verbose),
            &self.tasks().tasks_file,
            message,
        );
        mcp_success(Value::Object(self.with_warning(result)))
    }

    fn put_task_impl(&self, params: PutTaskParams) -> Result<CallToolResult, String> {
        validation::validate_non_empty_description(&params.description)?;
        if let Some(priority) = params.priority {
            validation::validate_range(priority, VALUE_MIN, VALUE_MAX, "priority")?;
        }
        validation::validate_tag_or_id(&params.tag, &params.id, "Task ID")?;

        if let Some(tag) = &params.tag {
            let priority = params.priority.unwrap_or(DEFAULT_NEW_TASK_PRIORITY);
            let task = self.tasks().add(&params.description, priority, tag)?;
            return Ok(self.task_response(&task, params.verbose, "Task created successfully"));
        }

        let id = params.id.as_deref().unwrap_or_default();
        let patch = TaskPatch {
            description: Some(params.description.clone()),
            priority: params.priority,
            status: None,
        };
        match self.tasks().update(id, &patch)? {
            Some(task) => Ok(self.task_response(
                &task,
                params.verbose,
                &format!("Task '{id}' updated successfully"),
            )),
            None => Ok(mcp_failure(&format!(
                "Task '{id}' not found. Use pjpd_list_tasks() to see existing task IDs."
            ))),
        }
    }

    fn list_tasks_impl(&self, params: ListTasksParams) -> Result<CallToolResult, String> {
        validation::validate_range(params.count, COUNT_MIN, COUNT_MAX, "count")?;
        let stage = Stage::parse(&params.status).ok_or_else(|| {
            format!(
                "status must be one of 'todo', 'done' or 'verified' (got '{}')",
                params.status
            )
        })?;

        // Filter by the file each task is saved to (R-16), so every task is
        // listed under exactly one status value.
        let mut tasks = self.tasks().load();
        tasks.retain(|t| t.stage() == stage);

        // Sort: status ToDo (any case, D-6) before other todo-file statuses,
        // then priority desc, then description.
        tasks.sort_by_key(|t| {
            (
                !t.status.eq_ignore_ascii_case(STATUS_TODO),
                std::cmp::Reverse(t.priority),
                t.description.to_lowercase(),
            )
        });

        let total = tasks.len();
        tasks.truncate(params.count as usize);

        let task_dicts: Vec<Value> = tasks
            .iter()
            .map(|t| task_listing(t, params.verbose))
            .collect();

        let mut result = Map::new();
        result.insert("total_tasks".to_string(), json!(total));
        result.insert("tasks".to_string(), json!(task_dicts));
        result.insert(
            "project_file".to_string(),
            json!(self.tasks().tasks_file.to_string_lossy()),
        );
        let mut result = self.with_warning(result);
        if total > params.count as usize {
            result.insert(
                "info".to_string(),
                json!(format!("Returned {} of {} total tasks", tasks.len(), total)),
            );
        }
        Ok(mcp_success(Value::Object(result)))
    }

    /// Failure naming the listed IDs that match no task, if any (D-16).
    fn missing_tasks_failure(&self, task_ids: &[String]) -> Option<CallToolResult> {
        let known: std::collections::HashSet<String> =
            self.tasks().load().into_iter().map(|t| t.id).collect();
        let missing: Vec<String> = task_ids
            .iter()
            .filter(|id| !known.contains(*id))
            .cloned()
            .collect();
        (!missing.is_empty()).then(|| {
            mcp_failure(&format!(
                "Tasks not found: {}. No tasks were modified. \
                 Use pjpd_list_tasks with status \"todo\", \"done\" or \"verified\" to see task IDs.",
                python_list_repr(&missing)
            ))
        })
    }

    /// All-or-nothing bulk task update: if any ID is missing, nothing is modified.
    fn bulk_update_tasks(
        &self,
        task_ids: &[String],
        patch: &TaskPatch,
        verbose: bool,
        message: &str,
    ) -> Result<CallToolResult, String> {
        if let Some(failure) = self.missing_tasks_failure(task_ids) {
            return Ok(failure);
        }

        let store = self.tasks();
        let updated: Vec<Value> = store
            .update_all(task_ids, patch)?
            .iter()
            .map(|t| task_summary(t, verbose))
            .collect();

        let mut result = Map::new();
        result.insert("tasks".to_string(), json!(updated));
        result.insert(
            "project_file".to_string(),
            json!(store.tasks_file.to_string_lossy()),
        );
        result.insert("message".to_string(), json!(message));
        Ok(mcp_success(Value::Object(self.with_warning(result))))
    }

    /// Demote tasks to ideas (D-21), all-or-nothing: each task's description
    /// becomes a new idea with the task's tag and DEMOTED_IDEA_SCORE, then the
    /// tasks are removed. Ideas are saved first, so a failure in between
    /// leaves a duplicate rather than losing the text (G-1).
    fn demote_tasks(&self, task_ids: &[String], verbose: bool) -> Result<CallToolResult, String> {
        if let Some(failure) = self.missing_tasks_failure(task_ids) {
            return Ok(failure);
        }

        let store = self.tasks();
        let entries: Vec<(String, String)> = store
            .load()
            .into_iter()
            .filter(|t| task_ids.contains(&t.id))
            .map(|t| (t.description, t.tag))
            .collect();
        let ideas = self.ideas();
        let added = ideas.add_all(&entries, DEMOTED_IDEA_SCORE)?;
        store.remove(task_ids)?;

        let mut result = Map::new();
        result.insert(
            "ideas".to_string(),
            json!(
                added
                    .iter()
                    .map(|i| idea_summary(i, verbose))
                    .collect::<Vec<Value>>()
            ),
        );
        result.insert(
            "project_file".to_string(),
            json!(ideas.ideas_file.to_string_lossy()),
        );
        result.insert(
            "message".to_string(),
            json!(format!(
                "Demoted {} task(s) to ideas: {}",
                added.len(),
                python_list_repr(task_ids)
            )),
        );
        Ok(mcp_success(Value::Object(self.with_warning(result))))
    }

    fn mark_task_impl(&self, params: MarkTaskParams) -> Result<CallToolResult, String> {
        let state = MarkState::parse(&params.state)?;
        validation::validate_id_list(&params.task_ids, "Task ID")?;
        let count = params.task_ids.len();
        let set_status = |status: &str, message: String| {
            let patch = TaskPatch {
                description: None,
                priority: None,
                status: Some(status.to_string()),
            };
            self.bulk_update_tasks(&params.task_ids, &patch, params.verbose, &message)
        };
        match state {
            MarkState::Done => set_status(STATUS_DONE, format!("Marked {count} task(s) as done")),
            MarkState::Verified => set_status(
                STATUS_VERIFIED,
                format!("Marked {count} task(s) as verified"),
            ),
            MarkState::Idea => self.demote_tasks(&params.task_ids, params.verbose),
        }
    }

    fn list_ideas_impl(&self, params: ListIdeasParams) -> Result<CallToolResult, String> {
        validation::validate_range(params.count, COUNT_MIN, COUNT_MAX, "count")?;

        let mut ideas = self.ideas().load();
        let total = ideas.len();
        ideas.truncate(params.count as usize);
        let idea_dicts: Vec<Value> = ideas
            .iter()
            .map(|i| idea_listing(i, params.verbose))
            .collect();

        let mut result = Map::new();
        result.insert("total_ideas".to_string(), json!(total));
        result.insert("ideas".to_string(), json!(idea_dicts));
        result.insert(
            "project_file".to_string(),
            json!(self.ideas().ideas_file.to_string_lossy()),
        );
        result.insert(
            "message".to_string(),
            json!(format!("Retrieved {} ideas", ideas.len())),
        );
        if total > ideas.len() {
            result.insert(
                "info".to_string(),
                json!(format!("Returned {} of {} total ideas", ideas.len(), total)),
            );
        }
        Ok(mcp_success(Value::Object(result)))
    }

    fn put_idea_impl(&self, params: PutIdeaParams) -> Result<CallToolResult, String> {
        validation::validate_range(params.score, VALUE_MIN, VALUE_MAX, "score")?;
        validation::validate_non_empty_description(&params.description)?;
        validation::validate_tag_or_id(&params.tag, &params.id, "Idea ID")?;

        let store = self.ideas();
        if let Some(tag) = &params.tag {
            let idea = store.add(&params.description, params.score, tag)?;
            return Ok(idea_response(
                &store,
                &idea,
                params.verbose,
                &format!("Idea created successfully with ID '{}'", idea.id),
            ));
        }

        let id = params.id.as_deref().unwrap_or_default();
        match store.update(id, &params.description, params.score)? {
            Some(idea) => Ok(idea_response(
                &store,
                &idea,
                params.verbose,
                &format!("Idea '{id}' updated successfully"),
            )),
            None => Ok(mcp_failure(&format!(
                "Idea '{id}' not found. Use pjpd_list_ideas() to see existing idea IDs."
            ))),
        }
    }

    fn mark_idea_done_impl(&self, params: MarkIdeaDoneParams) -> Result<CallToolResult, String> {
        validation::validate_id_list(&params.idea_ids, "Idea ID")?;

        let store = self.ideas();
        let known: std::collections::HashSet<String> =
            store.load().into_iter().map(|i| i.id).collect();
        let missing: Vec<String> = params
            .idea_ids
            .iter()
            .filter(|id| !known.contains(*id))
            .cloned()
            .collect();
        if !missing.is_empty() {
            return Ok(mcp_failure(&format!(
                "Ideas not found: {}. No ideas were modified. \
                 Use pjpd_list_ideas() to see existing idea IDs.",
                python_list_repr(&missing)
            )));
        }

        let marked: Vec<Value> = store
            .mark_done_all(&params.idea_ids)?
            .iter()
            .map(|i| idea_summary(i, params.verbose))
            .collect();

        Ok(mcp_success(json!({
            "ideas": marked,
            "project_file": store.ideas_file.to_string_lossy(),
            "message": format!("Marked {} idea(s) as done", params.idea_ids.len()),
        })))
    }
}

fn idea_response(store: &IdeaStore, idea: &Idea, verbose: bool, message: &str) -> CallToolResult {
    mcp_success(Value::Object(record_map(
        idea_summary(idea, verbose),
        &store.ideas_file,
        message,
    )))
}

/// The common response body for a single record: its fields plus
/// `project_file` and `message`.
fn record_map(record_json: Value, project_file: &Path, message: &str) -> Map<String, Value> {
    let mut map = match record_json {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    map.insert(
        "project_file".to_string(),
        json!(project_file.to_string_lossy()),
    );
    map.insert("message".to_string(), json!(message));
    map
}

#[tool_router]
impl PjpdServer {
    #[tool(
        name = "pjpd_put_task",
        description = "Create or update a task. Provide `tag` to create a new task, or `id` to update an existing one."
    )]
    pub async fn pjpd_put_task(
        &self,
        Parameters(params): Parameters<PutTaskParams>,
    ) -> CallToolResult {
        self.put_task_impl(params)
            .unwrap_or_else(|e| mcp_failure(&format!("Error putting task: {e}")))
    }

    #[tool(
        name = "pjpd_list_tasks",
        description = "List tasks in one status: \"todo\" (the default), \"done\" or \"verified\". Sorted by priority descending. Each task shows a one-line summary; pass verbose: true for full descriptions."
    )]
    pub async fn pjpd_list_tasks(
        &self,
        Parameters(params): Parameters<ListTasksParams>,
    ) -> CallToolResult {
        self.list_tasks_impl(params)
            .unwrap_or_else(|e| mcp_failure(&format!("Error listing tasks: {e}")))
    }

    #[tool(
        name = "pjpd_mark_task",
        description = "Move one or more tasks to a new state: \"done\" (you finished and committed the work), \"verified\" (only when the user says they reviewed or tried it), or \"idea\" (demote: the task's description becomes a new idea and the task is removed). All IDs must exist or nothing is changed."
    )]
    pub async fn pjpd_mark_task(
        &self,
        Parameters(params): Parameters<MarkTaskParams>,
    ) -> CallToolResult {
        self.mark_task_impl(params)
            .unwrap_or_else(|e| mcp_failure(&format!("Error marking tasks: {e}")))
    }

    #[tool(
        name = "pjpd_reprioritize_task",
        description = "Set the priority of one or more tasks. All IDs must exist or nothing is changed. Use this to shuffle priorities without resending the description text."
    )]
    pub async fn pjpd_reprioritize_task(
        &self,
        Parameters(params): Parameters<ReprioritizeTaskParams>,
    ) -> CallToolResult {
        let run = || -> Result<CallToolResult, String> {
            validation::validate_range(params.priority, VALUE_MIN, VALUE_MAX, "priority")?;
            validation::validate_id_list(&params.task_ids, "Task ID")?;
            let reprioritize = TaskPatch {
                description: None,
                priority: Some(params.priority),
                status: None,
            };
            self.bulk_update_tasks(
                &params.task_ids,
                &reprioritize,
                params.verbose,
                &format!(
                    "Reprioritized {} task(s) to priority {}",
                    params.task_ids.len(),
                    params.priority
                ),
            )
        };
        run().unwrap_or_else(|e| {
            mcp_failure(&format!(
                "Error reprioritizing tasks: {e}. \
                 Possible fixes: 1) Ensure each task_id matches <tag>-XXXX format (e.g. 'task-ab12'), \
                 2) Pass priority as an integer (0-100 typical, higher allowed), \
                 3) Use pjpd_list_tasks with status \"todo\", \"done\" or \"verified\" to confirm the task IDs exist."
            ))
        })
    }

    #[tool(
        name = "pjpd_list_ideas",
        description = "List ideas sorted by score (highest first). Each idea shows a one-line summary; pass verbose: true for full descriptions."
    )]
    pub async fn pjpd_list_ideas(
        &self,
        Parameters(params): Parameters<ListIdeasParams>,
    ) -> CallToolResult {
        self.list_ideas_impl(params)
            .unwrap_or_else(|e| mcp_failure(&format!("Error listing ideas: {e}")))
    }

    #[tool(
        name = "pjpd_put_idea",
        description = "Create or update an idea. Provide `tag` to create a new idea, or `id` to update an existing one."
    )]
    pub async fn pjpd_put_idea(
        &self,
        Parameters(params): Parameters<PutIdeaParams>,
    ) -> CallToolResult {
        self.put_idea_impl(params)
            .unwrap_or_else(|e| mcp_failure(&format!("Error putting idea: {e}")))
    }

    #[tool(
        name = "pjpd_mark_idea_done",
        description = "Mark one or more ideas as done (description prefixed with '(Done)'; the score is preserved so it stays visible even though the idea is off the books). All IDs must exist or nothing is changed."
    )]
    pub async fn pjpd_mark_idea_done(
        &self,
        Parameters(params): Parameters<MarkIdeaDoneParams>,
    ) -> CallToolResult {
        self.mark_idea_done_impl(params)
            .unwrap_or_else(|e| mcp_failure(&format!("Error marking ideas as done: {e}")))
    }
}

#[prompt_router]
impl PjpdServer {
    #[prompt(
        name = "pjpd_intro",
        description = "Return an introductory prompt that describes the ProjectMCP system."
    )]
    pub async fn pjpd_intro(&self) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(Role::User, INTRO_TEXT)]
    }
}

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl ServerHandler for PjpdServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
        )
        .with_server_info(Implementation::new("projectmcp", env!("CARGO_PKG_VERSION")))
    }
}
