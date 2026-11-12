//! Idea records and the store backing pjpd/ideas.txt.

use std::cmp::Reverse;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use crate::record::{
    parse_record, push_description, readdress_legacy, resolve_id, stamp, tag_from_id,
};
use crate::{ids, textrec};

#[derive(Debug, Clone, PartialEq)]
pub struct Idea {
    pub id: String,
    pub tag: String,
    pub score: i64,
    pub description: String,
    pub created: Option<String>,
    pub updated: Option<String>,
}

impl Idea {
    /// An idea is done if its description starts with "(Done)".
    pub fn is_done(&self) -> bool {
        self.description.starts_with("(Done)")
    }

    /// Parse an idea from record text. Returns None only when a missing ID
    /// cannot be regenerated from an invalid tag.
    pub fn from_text(text: &str) -> Option<Idea> {
        let parsed = parse_record(text, &["id", "tag", "created", "updated"], &["score"]);
        let id = resolve_id(parsed.property("id"), parsed.property("tag"), "idea")?;
        let score = parsed.numeric_property("score").unwrap_or(1);
        let created = parsed.owned_property("created");
        let updated = parsed.owned_property("updated");

        Some(Idea {
            tag: tag_from_id(&id),
            id,
            score,
            description: parsed.description,
            created,
            updated,
        })
    }

    /// Set created timestamp if not already set, and refresh updated.
    pub fn stamp(&mut self) {
        stamp(&mut self.created, &mut self.updated);
    }

    /// Replace a legacy `XX-XXXX-XX` ID with a fresh `legacy-XXXX` one (D-8);
    /// any other ID is unchanged.
    pub fn readdress(&mut self) {
        if let Some(id) = readdress_legacy(&self.id, "idea") {
            self.tag = tag_from_id(&id);
            self.id = id;
        }
    }

    /// Render back to on-disk record form: each property on its own line,
    /// then a blank line, then the description.
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("Score: {:4}", self.score),
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
        map.insert("score".to_string(), json!(self.score));
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

/// Store for idea records, reloaded from disk on every operation.
/// Ideas are kept sorted: active first, then done, by score descending within
/// each group.
pub struct IdeaStore {
    pub ideas_file: PathBuf,
}

impl IdeaStore {
    pub fn new(project_dir: &Path) -> IdeaStore {
        IdeaStore {
            ideas_file: project_dir.join("pjpd").join("ideas.txt"),
        }
    }

    pub fn load(&self) -> Vec<Idea> {
        let mut ideas: Vec<Idea> = textrec::read_records(&self.ideas_file)
            .into_iter()
            .filter_map(|record| Idea::from_text(&record))
            .collect();
        Self::sort(&mut ideas);
        ideas
    }

    fn sort(ideas: &mut [Idea]) {
        ideas.sort_by_key(|i| (i.is_done(), Reverse(i.score)));
    }

    fn save(&self, ideas: &mut Vec<Idea>) -> io::Result<()> {
        if let Some(parent) = self.ideas_file.parent() {
            fs::create_dir_all(parent)?;
        }
        ideas.iter_mut().for_each(Idea::readdress);
        Self::sort(ideas);
        let content = textrec::join_records(ideas.iter().map(|i| i.to_text()));
        textrec::write_atomic(&self.ideas_file, &content)
    }

    pub fn add(&self, description: &str, score: i64, tag: &str) -> Result<Idea, String> {
        let mut added = self.add_all(&[(description.to_string(), tag.to_string())], score)?;
        Ok(added.remove(0))
    }

    /// Add one idea per (description, tag) pair, all with the same score, in
    /// one load and one save. Returns the new ideas in input order.
    pub fn add_all(&self, entries: &[(String, String)], score: i64) -> Result<Vec<Idea>, String> {
        let added = entries
            .iter()
            .map(|(description, tag)| {
                let mut idea = Idea {
                    id: ids::generate_with_tag(tag)?,
                    tag: tag.clone(),
                    score,
                    description: textrec::indent_separator_lines(description),
                    created: None,
                    updated: None,
                };
                idea.stamp();
                Ok(idea)
            })
            .collect::<Result<Vec<Idea>, String>>()?;

        let mut ideas = self.load();
        ideas.extend(added.iter().cloned());
        self.save(&mut ideas).map_err(|e| e.to_string())?;
        Ok(added)
    }

    /// Replace an idea's description and score. Returns Ok(None) when the ID
    /// is unknown. Both fields are always provided by the tool surface, so
    /// there is no partial-update variant.
    pub fn update(
        &self,
        idea_id: &str,
        description: &str,
        score: i64,
    ) -> Result<Option<Idea>, String> {
        let mut ideas = self.load();
        let Some(idea) = ideas.iter_mut().find(|i| i.id == idea_id) else {
            return Ok(None);
        };

        idea.description = textrec::indent_separator_lines(description);
        idea.score = score;
        idea.stamp();

        let updated = idea.clone();
        self.save(&mut ideas).map_err(|e| e.to_string())?;
        Ok(Some(updated))
    }

    /// Mark an idea done by prefixing "(Done)" to the first description line.
    /// The score is preserved. Returns false when the ID is unknown.
    pub fn mark_done(&self, idea_id: &str) -> Result<bool, String> {
        Ok(!self.mark_done_all(&[idea_id.to_string()])?.is_empty())
    }

    /// Mark every listed idea done in one load and one save (D-16), returning
    /// the marked ideas in save order. Unknown IDs are ignored; when none
    /// match, nothing is written.
    pub fn mark_done_all(&self, idea_ids: &[String]) -> Result<Vec<Idea>, String> {
        let mut ideas = self.load();
        let touched = ideas
            .iter_mut()
            .filter(|i| idea_ids.contains(&i.id))
            .map(|idea| {
                idea.description = done_description(&idea.description);
                idea.stamp();
            })
            .count();
        if touched == 0 {
            return Ok(Vec::new());
        }
        self.save(&mut ideas).map_err(|e| e.to_string())?;
        Ok(ideas
            .into_iter()
            .filter(|i| idea_ids.contains(&i.id))
            .collect())
    }
}

/// The description with "(Done)" prefixed to its first line, never doubled.
fn done_description(description: &str) -> String {
    let (first_line, rest) = match description.split_once('\n') {
        Some((first, rest)) => (first.to_string(), Some(rest.to_string())),
        None => (description.to_string(), None),
    };

    let first_line = if first_line.starts_with("(Done)") {
        first_line
    } else if first_line.is_empty() {
        "(Done)".to_string()
    } else {
        format!("(Done) {first_line}")
    };

    match rest {
        Some(rest) if !rest.is_empty() => format!("{first_line}\n{rest}"),
        _ => first_line,
    }
}
