//! R-15: tool calls with bad arguments (empty description, bad tag, bad ID,
//! both or neither of tag/id, values out of range, an empty ID list, an
//! unknown state or status) fail with a message and change nothing.

#[path = "../common/mod.rs"]
mod common;

use common::{
    assert_failure, create_idea, create_task, list_tasks, mark_tasks, response_json, server,
    snapshot,
};
use pjpd::ids::TAG_MAX_LEN;
use pjpd::server::{
    COUNT_MAX, COUNT_MIN, ListIdeasParams, ListTasksParams, MarkIdeaDoneParams, PjpdServer,
    PutIdeaParams, PutTaskParams, ReprioritizeTaskParams, VALUE_MAX, VALUE_MIN,
};
use pjpd::validation::{
    validate_exactly_one_of_tag_or_id, validate_id, validate_range, validate_tag,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use tempfile::TempDir;

// R-15: tags must be 1..TAG_MAX_LEN chars of [A-Za-z0-9-].
#[test]
fn tag_rules() {
    assert!(validate_tag("fix-bug").is_ok());
    assert!(validate_tag("").is_err());
    assert!(validate_tag(&"t".repeat(TAG_MAX_LEN)).is_ok());
    assert!(validate_tag(&"t".repeat(TAG_MAX_LEN + 1)).is_err());
    assert!(validate_tag("has space").is_err());
}

// R-15: IDs must match the ID pattern.
#[test]
fn id_rules() {
    assert!(validate_id("tag-ab29", "Task ID").is_ok());
    assert!(validate_id("nodash", "Task ID").is_err());
    assert!(validate_id("tag-ABCD", "Task ID").is_err());
}

// R-15: exactly one of tag or id.
#[test]
fn exactly_one_of_tag_or_id() {
    let some = |s: &str| Some(s.to_string());
    assert!(validate_exactly_one_of_tag_or_id(&some("t"), &None).is_ok());
    assert!(validate_exactly_one_of_tag_or_id(&None, &some("t-ab12")).is_ok());
    assert!(validate_exactly_one_of_tag_or_id(&some("t"), &some("t-ab12")).is_err());
    assert!(validate_exactly_one_of_tag_or_id(&None, &None).is_err());
}

// R-15: values outside the allowed range fail.
#[test]
fn ranges() {
    assert!(validate_range(VALUE_MIN, VALUE_MIN, VALUE_MAX, "priority").is_ok());
    assert!(validate_range(VALUE_MAX + 1, VALUE_MIN, VALUE_MAX, "priority").is_err());
    assert!(validate_range(COUNT_MIN - 1, COUNT_MIN, COUNT_MAX, "count").is_err());
}

// R-15: pjpd_put_task with both tag and id fails.
#[tokio::test]
async fn put_task_rejects_tag_and_id_together() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "x".to_string(),
            tag: Some("t".to_string()),
            id: Some("t-ab29".to_string()),
            priority: Some(50),
            verbose: false,
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(error.contains("not both"), "got: {error}");
}

// R-15: pjpd_mark_task with an unknown state fails and changes nothing.
#[tokio::test]
async fn mark_task_rejects_unknown_state() {
    let (_dir, server) = server();
    let id = create_task(&server, "x", "x", 5).await;

    let error = assert_failure(&mark_tasks(&server, &[&id], "finished").await);
    assert!(
        error.contains("'done', 'verified' or 'idea'"),
        "got: {error}"
    );
    assert_eq!(list_tasks(&server, "todo").await["total_tasks"], 1);
}

// R-15: pjpd_list_tasks with an unknown status filter fails.
#[tokio::test]
async fn list_tasks_rejects_unknown_status() {
    let (_dir, server) = server();
    let result = server
        .pjpd_list_tasks(Parameters(ListTasksParams {
            count: 20,
            status: "all".to_string(),
            verbose: false,
        }))
        .await;
    let error = assert_failure(&response_json(&result));
    assert!(
        error.contains("'todo', 'done' or 'verified'"),
        "got: {error}"
    );
}

/// A server holding one task and one idea, so "changes nothing" is checked
/// against real files.
async fn seeded() -> (TempDir, PjpdServer, String, String) {
    let (dir, server) = server();
    let task = create_task(&server, "existing task", "task", 5).await;
    let idea = create_idea(&server, "existing idea", "idea", 5).await;
    (dir, server, task, idea)
}

/// Assert the call failed with a message containing `expected` and left
/// every file under pjpd/ as it was.
fn assert_rejected(
    dir: &TempDir,
    before: &std::collections::BTreeMap<String, String>,
    result: &CallToolResult,
    expected: &str,
) {
    let error = assert_failure(&response_json(result));
    assert!(
        error.contains(expected),
        "expected '{expected}', got: {error}"
    );
    assert_eq!(&snapshot(dir), before);
}

fn put_task_params(description: &str, tag: Option<&str>, id: Option<&str>) -> PutTaskParams {
    PutTaskParams {
        description: description.to_string(),
        tag: tag.map(str::to_string),
        id: id.map(str::to_string),
        priority: None,
        verbose: false,
    }
}

fn put_idea_params(
    description: &str,
    tag: Option<&str>,
    id: Option<&str>,
    score: i64,
) -> PutIdeaParams {
    PutIdeaParams {
        score,
        description: description.to_string(),
        tag: tag.map(str::to_string),
        id: id.map(str::to_string),
        verbose: false,
    }
}

// R-15: pjpd_put_task rejects an empty description, a bad tag, a bad ID,
// neither tag nor id, and a priority outside VALUE_MIN..VALUE_MAX, changing
// nothing.
#[tokio::test]
async fn put_task_rejects_bad_arguments() {
    let (dir, server, task, _) = seeded().await;
    let before = snapshot(&dir);
    let long_tag = "t".repeat(TAG_MAX_LEN + 1);
    let tag_length_error = format!("Tag must be 1-{TAG_MAX_LEN}");
    let cases = [
        (
            put_task_params("", Some("new"), None),
            "Description must not be empty",
        ),
        (
            put_task_params("", None, Some(&task)),
            "Description must not be empty",
        ),
        (
            put_task_params("x", Some("has space"), None),
            "Tag can only contain",
        ),
        (
            put_task_params("x", Some(&long_tag), None),
            &tag_length_error,
        ),
        (
            put_task_params("x", None, Some("nodash")),
            "Task ID must be in format",
        ),
        (put_task_params("x", None, None), "Provide either 'tag'"),
    ];
    for (params, expected) in cases {
        let result = server.pjpd_put_task(Parameters(params)).await;
        assert_rejected(&dir, &before, &result, expected);
    }
    for priority in [VALUE_MIN - 1, VALUE_MAX + 1] {
        let params = PutTaskParams {
            priority: Some(priority),
            ..put_task_params("x", Some("new"), None)
        };
        let result = server.pjpd_put_task(Parameters(params)).await;
        assert_rejected(
            &dir,
            &before,
            &result,
            &format!("priority must be between {VALUE_MIN} and {VALUE_MAX}"),
        );
    }
}

// R-15: pjpd_put_idea rejects an empty description, a bad tag, a bad ID,
// both or neither of tag/id, and a score outside VALUE_MIN..VALUE_MAX,
// changing nothing.
#[tokio::test]
async fn put_idea_rejects_bad_arguments() {
    let (dir, server, _, idea) = seeded().await;
    let before = snapshot(&dir);
    let score_error = format!("score must be between {VALUE_MIN} and {VALUE_MAX}");
    let cases = [
        (
            put_idea_params("", Some("new"), None, 5),
            "Description must not be empty",
        ),
        (
            put_idea_params("", None, Some(&idea), 5),
            "Description must not be empty",
        ),
        (
            put_idea_params("x", Some("bad/tag"), None, 5),
            "Tag can only contain",
        ),
        (
            put_idea_params("x", None, Some("idea-ABCD"), 5),
            "Idea ID must be in format",
        ),
        (
            put_idea_params("x", Some("new"), Some(&idea), 5),
            "not both",
        ),
        (put_idea_params("x", None, None, 5), "Provide either 'tag'"),
        (
            put_idea_params("x", Some("new"), None, VALUE_MIN - 1),
            &score_error,
        ),
        (
            put_idea_params("x", None, Some(&idea), VALUE_MAX + 1),
            &score_error,
        ),
    ];
    for (params, expected) in cases {
        let result = server.pjpd_put_idea(Parameters(params)).await;
        assert_rejected(&dir, &before, &result, expected);
    }
}

// R-15: both list tools reject a count outside COUNT_MIN..COUNT_MAX.
#[tokio::test]
async fn list_tools_reject_count_out_of_range() {
    let (dir, server, _, _) = seeded().await;
    let before = snapshot(&dir);
    let count_error = format!("count must be between {COUNT_MIN} and {COUNT_MAX}");
    for count in [COUNT_MIN - 1, COUNT_MAX + 1] {
        let result = server
            .pjpd_list_tasks(Parameters(ListTasksParams {
                count,
                status: "todo".to_string(),
                verbose: false,
            }))
            .await;
        assert_rejected(&dir, &before, &result, &count_error);
        let result = server
            .pjpd_list_ideas(Parameters(ListIdeasParams {
                count,
                verbose: false,
            }))
            .await;
        assert_rejected(&dir, &before, &result, &count_error);
    }
}

// R-15: every ID-list tool rejects an empty list and a malformed ID.
#[tokio::test]
async fn id_list_tools_reject_empty_or_malformed_lists() {
    let (dir, server, task, idea) = seeded().await;
    let before = snapshot(&dir);
    for state in ["done", "verified", "idea"] {
        let result = mark_tasks(&server, &[], state).await;
        let error = assert_failure(&result);
        assert!(
            error.contains("Task ID list must not be empty"),
            "got: {error}"
        );
        let result = mark_tasks(&server, &[&task, "BAD"], state).await;
        let error = assert_failure(&result);
        assert!(error.contains("Task ID must be in format"), "got: {error}");
        assert_eq!(snapshot(&dir), before);
    }

    let reprioritize = |task_ids: Vec<String>, priority: i64| {
        server.pjpd_reprioritize_task(Parameters(ReprioritizeTaskParams {
            priority,
            task_ids,
            verbose: false,
        }))
    };
    let result = reprioritize(vec![], 5).await;
    assert_rejected(&dir, &before, &result, "Task ID list must not be empty");
    let result = reprioritize(vec![task.clone()], VALUE_MAX + 1).await;
    assert_rejected(
        &dir,
        &before,
        &result,
        &format!("priority must be between {VALUE_MIN} and {VALUE_MAX}"),
    );

    let mark_ideas = |idea_ids: Vec<String>| {
        server.pjpd_mark_idea_done(Parameters(MarkIdeaDoneParams {
            idea_ids,
            verbose: false,
        }))
    };
    let result = mark_ideas(vec![]).await;
    assert_rejected(&dir, &before, &result, "Idea ID list must not be empty");
    let result = mark_ideas(vec![idea, "nodash".to_string()]).await;
    assert_rejected(&dir, &before, &result, "Idea ID must be in format");
}
