//! Task records and the store backing pjpd/tasks.txt (ToDo),
//! pjpd/tasks_done.txt (Done) and pjpd/tasks_verified.txt (Verified); the two
//! archives are written without bak/ backups (D-6).

use std::cmp::Reverse;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::record::{
    DEFAULT_LOADED_PRIORITY, NUMBER_WIDTH, parse_record, push_description, readdress_legacy,
    resolve_id, stamp, tag_from_id,
};
use crate::{ids, textrec};

pub const STATUS_TODO: &str = "ToDo";
pub const STATUS_DONE: &str = "Done";
pub const STATUS_VERIFIED: &str = "Verified";

/// Which task file a task is saved to, decided by a case-insensitive status
/// match (R-10, D-6). Any status other than `Done` or `Verified` (in any case)
/// is a todo task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Todo,
    Done,
    Verified,
}

impl Stage {
    /// Parse a stage name (`todo`, `done`, `verified`), case-insensitively.
    /// The one comparison rule for statuses: the list filter and a task's
    /// saved status both go through it (D-6).
    pub fn parse(value: &str) -> Option<Stage> {
        [
            ("todo", Stage::Todo),
            ("done", Stage::Done),
            ("verified", Stage::Verified),
        ]
        .into_iter()
        .find(|(name, _)| value.eq_ignore_ascii_case(name))
        .map(|(_, stage)| stage)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: String,
    pub tag: String,
    pub priority: i64,
    pub status: String,
    pub description: String,
    pub created: Option<String>,
    pub updated: Option<String>,
}

impl Task {
    /// Parse a task from record text: a leading run of property lines
    /// (`Priority:`, `Status:`, `ID:`, `Tag:`, `Created:`, `Updated:`) in any
    /// order, with everything from the first non-property line onward forming
    /// the description. Returns None only when a missing ID cannot be
    /// regenerated from an invalid tag.
    pub fn from_text(text: &str) -> Option<Task> {
        let parsed = parse_record(
            text,
            &["status", "id", "tag", "created", "updated"],
            &["priority"],
        );
        let id = resolve_id(parsed.property("id"), parsed.property("tag"), "task")?;
        let priority = parsed
            .numeric_property("priority")
            .unwrap_or(DEFAULT_LOADED_PRIORITY);
        let status = parsed
            .owned_property("status")
            .unwrap_or_else(|| STATUS_TODO.to_string());
        let created = parsed.owned_property("created");
        let updated = parsed.owned_property("updated");

        Some(Task {
            tag: tag_from_id(&id),
            id,
            priority,
            status,
            description: parsed.description,
            created,
            updated,
        })
    }

    /// The task's stage, from its status case-insensitively, so a
    /// hand-edited `done` is archived and listed like `Done` (D-6).
    pub fn stage(&self) -> Stage {
        Stage::parse(&self.status).unwrap_or(Stage::Todo)
    }

    /// Set created timestamp if not already set, and refresh updated.
    pub fn stamp(&mut self) {
        stamp(&mut self.created, &mut self.updated);
    }

    /// Replace a legacy `XX-XXXX-XX` ID with a fresh `legacy-XXXX` one (D-8);
    /// any other ID is unchanged.
    pub fn readdress(&mut self) {
        if let Some(id) = readdress_legacy(&self.id, "task") {
            self.tag = tag_from_id(&id);
            self.id = id;
        }
    }

    /// Render back to on-disk record form: each property on its own line,
    /// then a blank line, then the description.
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("Priority: {:NUMBER_WIDTH$}", self.priority),
            format!("Status: {}", self.status),
            format!("ID: {}", self.id),
        ];
        if let Some(created) = &self.created {
            lines.push(format!("Created: {created}"));
        }
        if let Some(updated) = &self.updated {
            lines.push(format!("Updated: {updated}"));
        }
        push_description(&mut lines, &self.description);
        lines.join("\n")
    }

    /// Convert to the dictionary shape used in API responses.
    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("id".to_string(), json!(self.id));
        map.insert("priority".to_string(), json!(self.priority));
        map.insert("status".to_string(), json!(self.status));
        map.insert("description".to_string(), json!(self.description));
        if let Some(created) = &self.created {
            map.insert("created".to_string(), json!(created));
        }
        if let Some(updated) = &self.updated {
            map.insert("updated".to_string(), json!(updated));
        }
        Value::Object(map)
    }
}

/// Partial update applied to an existing task; None fields are left unchanged.
#[derive(Debug, Clone)]
pub struct TaskPatch {
    pub description: Option<String>,
    pub priority: Option<i64>,
    pub status: Option<String>,
}

impl TaskPatch {
    fn apply_to(&self, task: &mut Task) {
        if let Some(description) = &self.description {
            task.description = textrec::indent_separator_lines(description);
        }
        if let Some(priority) = self.priority {
            task.priority = priority;
        }
        if let Some(status) = &self.status {
            task.status = status.clone();
        }
    }
}

/// Store for the single project's tasks, reloaded from disk on every operation.
pub struct TaskStore {
    pub project_dir: PathBuf,
    pub tasks_file: PathBuf,
    pub done_tasks_file: PathBuf,
    pub verified_tasks_file: PathBuf,
}

impl TaskStore {
    pub fn new(project_dir: &Path) -> TaskStore {
        let pjpd_dir = project_dir.join("pjpd");
        TaskStore {
            project_dir: project_dir.to_path_buf(),
            tasks_file: pjpd_dir.join("tasks.txt"),
            done_tasks_file: pjpd_dir.join("tasks_done.txt"),
            verified_tasks_file: pjpd_dir.join("tasks_verified.txt"),
        }
    }

    /// Warning when a `pjpd/<dir_name>.txt` file from the old multi-project
    /// layout still exists.
    pub fn legacy_warning(&self) -> Option<String> {
        let dir_name = self.project_dir.file_name()?.to_string_lossy().into_owned();
        let legacy_file = self
            .project_dir
            .join("pjpd")
            .join(format!("{dir_name}.txt"));
        if !legacy_file.exists() || legacy_file == self.tasks_file {
            return None;
        }
        Some(format!(
            "Legacy project file '{dir_name}.txt' found in pjpd/. \
             This is from an older multi-project layout. Tasks are now stored in pjpd/tasks.txt. \
             Migrate any tasks from '{dir_name}.txt' into tasks.txt, then delete the legacy file."
        ))
    }

    /// Load all tasks from the todo, done and verified files.
    pub fn load(&self) -> Vec<Task> {
        [
            &self.tasks_file,
            &self.done_tasks_file,
            &self.verified_tasks_file,
        ]
        .iter()
        .flat_map(|path| textrec::read_records(path))
        .filter_map(|record| Task::from_text(&record))
        .collect()
    }

    /// Save todo tasks to tasks.txt (with backup), done tasks to
    /// tasks_done.txt and verified tasks to tasks_verified.txt (no backup),
    /// each sorted by priority descending. An archive file is written only
    /// when it has tasks or already exists. Legacy IDs are rewritten (D-8).
    fn save(&self, tasks: &[Task]) -> io::Result<()> {
        let mut tasks = tasks.to_vec();
        tasks.iter_mut().for_each(Task::readdress);
        let render_stage = |stage: Stage| {
            let mut group: Vec<&Task> = tasks.iter().filter(|t| t.stage() == stage).collect();
            group.sort_by_key(|t| Reverse(t.priority));
            (
                group.is_empty(),
                textrec::join_records(group.iter().map(|t| t.to_text())),
            )
        };

        let (_, todo_content) = render_stage(Stage::Todo);
        textrec::write_atomic(&self.tasks_file, &todo_content)?;

        [
            (Stage::Done, &self.done_tasks_file),
            (Stage::Verified, &self.verified_tasks_file),
        ]
        .into_iter()
        .try_for_each(|(stage, path)| {
            let (empty, content) = render_stage(stage);
            if empty && !path.exists() {
                return Ok(());
            }
            textrec::write_atomic_no_backup(path, &content)
        })
    }

    /// Add a new task, creating the pjpd directory and tasks file if needed.
    pub fn add(&self, description: &str, priority: i64, tag: &str) -> Result<Task, String> {
        self.ensure_files().map_err(|e| e.to_string())?;
        let mut task = Task {
            id: ids::generate_with_tag(tag)?,
            tag: tag.to_string(),
            priority,
            status: STATUS_TODO.to_string(),
            description: textrec::indent_separator_lines(description),
            created: None,
            updated: None,
        };
        task.stamp();

        let mut tasks = self.load();
        tasks.push(task.clone());
        self.save(&tasks).map_err(|e| e.to_string())?;
        Ok(task)
    }

    pub fn get(&self, task_id: &str) -> Option<Task> {
        self.load().into_iter().find(|t| t.id == task_id)
    }

    /// Apply a patch to an existing task. Returns Ok(None) when the ID is unknown.
    pub fn update(&self, task_id: &str, patch: &TaskPatch) -> Result<Option<Task>, String> {
        Ok(self
            .update_all(&[task_id.to_string()], patch)?
            .into_iter()
            .next())
    }

    /// Apply one patch to every listed task in one load and one save (D-16),
    /// returning the updated tasks in the order their IDs are listed. Unknown
    /// IDs are ignored; when none match, nothing is written.
    pub fn update_all(&self, task_ids: &[String], patch: &TaskPatch) -> Result<Vec<Task>, String> {
        let mut tasks = self.load();
        let touched = tasks
            .iter_mut()
            .filter(|t| task_ids.contains(&t.id))
            .map(|task| {
                patch.apply_to(task);
                task.stamp();
            })
            .count();
        if touched == 0 {
            return Ok(Vec::new());
        }
        self.save(&tasks).map_err(|e| e.to_string())?;
        Ok(task_ids
            .iter()
            .filter_map(|id| tasks.iter().find(|t| &t.id == id).cloned())
            .collect())
    }

    /// Remove every task whose ID is listed, in one load and one save, and
    /// return the removed tasks in file order. Unknown IDs are ignored.
    pub fn remove(&self, task_ids: &[String]) -> Result<Vec<Task>, String> {
        let (removed, kept): (Vec<Task>, Vec<Task>) = self
            .load()
            .into_iter()
            .partition(|t| task_ids.contains(&t.id));
        self.save(&kept).map_err(|e| e.to_string())?;
        Ok(removed)
    }

    fn ensure_files(&self) -> io::Result<()> {
        let pjpd_dir = self.project_dir.join("pjpd");
        fs::create_dir_all(&pjpd_dir)?;
        if !self.tasks_file.exists() {
            fs::write(&self.tasks_file, "")?;
        }
        Ok(())
    }
}
