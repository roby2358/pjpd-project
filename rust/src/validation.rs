//! Request validation for the MCP tool surface, mirroring the Python
//! Pydantic models' rules.

use crate::ids;

pub fn validate_tag(tag: &str) -> Result<(), String> {
    let len = tag.chars().count();
    if !(1..=12).contains(&len) {
        return Err("Tag must be 1-12 characters long".to_string());
    }
    if !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Tag can only contain alphanumeric characters and hyphens".to_string());
    }
    Ok(())
}

pub fn validate_id(id: &str, label: &str) -> Result<(), String> {
    if !ids::is_valid_id(id) {
        return Err(format!(
            "{label} must be in format <tag>-XXXX where XXXX is alphanumeric"
        ));
    }
    Ok(())
}

/// Exactly one of `tag` (create) or `id` (update) must be provided.
pub fn validate_exactly_one_of_tag_or_id(
    tag: &Option<String>,
    id: &Option<String>,
) -> Result<(), String> {
    match (tag, id) {
        (Some(_), Some(_)) => {
            Err("Provide either 'tag' (to create) or 'id' (to update), not both".to_string())
        }
        (None, None) => Err("Provide either 'tag' (to create) or 'id' (to update)".to_string()),
        _ => Ok(()),
    }
}

pub fn validate_range(value: i64, min: i64, max: i64, label: &str) -> Result<(), String> {
    if !(min..=max).contains(&value) {
        return Err(format!("{label} must be between {min} and {max}"));
    }
    Ok(())
}

pub fn validate_non_empty_description(description: &str) -> Result<(), String> {
    if description.is_empty() {
        return Err("Description must not be empty".to_string());
    }
    Ok(())
}

/// Validate a non-empty list of record IDs.
pub fn validate_id_list(id_list: &[String], label: &str) -> Result<(), String> {
    if id_list.is_empty() {
        return Err(format!("{label} list must not be empty"));
    }
    id_list.iter().try_for_each(|id| validate_id(id, label))
}

/// Validate the tag/id pair shared by put_task and put_idea.
pub fn validate_tag_or_id(
    tag: &Option<String>,
    id: &Option<String>,
    id_label: &str,
) -> Result<(), String> {
    if let Some(tag) = tag {
        validate_tag(tag)?;
    }
    if let Some(id) = id {
        validate_id(id, id_label)?;
    }
    validate_exactly_one_of_tag_or_id(tag, id)
}
