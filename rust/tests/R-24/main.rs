//! R-24: every response is `{success, result, error}` and every success result
//! includes `project_file` with the full path of the file operated on;
//! failures have `result: ""`.

#[path = "../common/mod.rs"]
mod common;

use common::{
    assert_success, create_idea, create_task, list_tasks, mark_tasks, response_json, server,
};
use pjpd::server::{
    ListIdeasParams, ListTasksParams, MarkIdeaDoneParams, PutIdeaParams, PutTaskParams,
    ReprioritizeTaskParams,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use serde_json::Value;

// R-24: a pjpd_put_task success names pjpd/tasks.txt as its project_file.
#[tokio::test]
async fn put_task_result_includes_project_file() {
    let (_dir, server) = server();
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "Write the docs".to_string(),
            tag: Some("docs".to_string()),
            id: None,
            priority: Some(7),
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert!(
        body["project_file"]
            .as_str()
            .unwrap()
            .ends_with("pjpd/tasks.txt")
    );
}

// R-24: pjpd_list_tasks names the file it listed: tasks.txt for todo,
// tasks_done.txt for done, tasks_verified.txt for verified, even before an
// archive file exists.
#[tokio::test]
async fn list_tasks_names_the_listed_file() {
    let (dir, server) = server();
    for (status, file) in [
        ("todo", "tasks.txt"),
        ("done", "tasks_done.txt"),
        ("verified", "tasks_verified.txt"),
    ] {
        let body = list_tasks(&server, status).await;
        let expected = dir.path().join("pjpd").join(file);
        assert_eq!(body["project_file"], expected.to_string_lossy().as_ref());
        assert!(file == "tasks.txt" || !expected.exists());
    }
}

// R-24: pjpd_mark_task names the file the tasks moved to: tasks_done.txt for
// done and tasks_verified.txt for verified, whichever file each task came
// from, and ideas.txt for idea.
#[tokio::test]
async fn mark_task_names_the_destination_file() {
    let (dir, server) = server();
    let a = create_task(&server, "a", "a", 5).await;
    let b = create_task(&server, "b", "b", 5).await;
    let c = create_task(&server, "c", "c", 5).await;
    for (ids, state, file) in [
        (vec![&a, &b], "done", "tasks_done.txt"),
        (vec![&a], "verified", "tasks_verified.txt"),
        (vec![&b, &c], "verified", "tasks_verified.txt"),
        (vec![&a], "idea", "ideas.txt"),
    ] {
        let ids: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();
        let body = assert_success(&mark_tasks(&server, &ids, state).await);
        let expected = dir.path().join("pjpd").join(file);
        assert_eq!(
            body["project_file"],
            expected.to_string_lossy().as_ref(),
            "{state}"
        );
    }
}

/// The response with its keys checked: exactly success, result and error.
fn envelope(result: &CallToolResult) -> Value {
    let response = response_json(result);
    let mut keys: Vec<&String> = response.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys, vec!["error", "result", "success"]);
    response
}

// R-24: every tool's success result names the full path of the file it
// operated on: tasks.txt for task tools, tasks_done.txt for marking and
// listing done, tasks_verified.txt for marking verified, ideas.txt for idea
// tools and for demoting a task.
#[tokio::test]
async fn every_tool_success_names_its_project_file() {
    let (dir, server) = server();
    let tasks_file = dir
        .path()
        .join("pjpd/tasks.txt")
        .to_string_lossy()
        .into_owned();
    let done_tasks_file = dir
        .path()
        .join("pjpd/tasks_done.txt")
        .to_string_lossy()
        .into_owned();
    let verified_tasks_file = dir
        .path()
        .join("pjpd/tasks_verified.txt")
        .to_string_lossy()
        .into_owned();
    let ideas_file = dir
        .path()
        .join("pjpd/ideas.txt")
        .to_string_lossy()
        .into_owned();
    let a = create_task(&server, "a", "a", 5).await;
    let b = create_task(&server, "b", "b", 5).await;
    let c = create_task(&server, "c", "c", 5).await;
    let idea = create_idea(&server, "an idea", "idea", 5).await;
    let body = |r: &CallToolResult| assert_success(&envelope(r));

    let results: Vec<(&str, Value, &str)> = vec![
        (
            "put_task update",
            body(
                &server
                    .pjpd_put_task(Parameters(PutTaskParams {
                        description: "a2".to_string(),
                        tag: None,
                        id: Some(a.clone()),
                        priority: None,
                        verbose: false,
                    }))
                    .await,
            ),
            &tasks_file,
        ),
        ("list_tasks", list_tasks(&server, "todo").await, &tasks_file),
        (
            "reprioritize",
            body(
                &server
                    .pjpd_reprioritize_task(Parameters(ReprioritizeTaskParams {
                        priority: 9,
                        task_ids: vec![a.clone()],
                        verbose: false,
                    }))
                    .await,
            ),
            &tasks_file,
        ),
        (
            "mark done",
            assert_success(&mark_tasks(&server, &[&a], "done").await),
            &done_tasks_file,
        ),
        (
            "mark verified",
            assert_success(&mark_tasks(&server, &[&b], "verified").await),
            &verified_tasks_file,
        ),
        (
            "list done",
            list_tasks(&server, "done").await,
            &done_tasks_file,
        ),
        (
            "mark idea",
            assert_success(&mark_tasks(&server, &[&c], "idea").await),
            &ideas_file,
        ),
        (
            "put_idea create",
            body(
                &server
                    .pjpd_put_idea(Parameters(PutIdeaParams {
                        score: 5,
                        description: "another".to_string(),
                        tag: Some("another".to_string()),
                        id: None,
                        verbose: false,
                    }))
                    .await,
            ),
            &ideas_file,
        ),
        (
            "put_idea update",
            body(
                &server
                    .pjpd_put_idea(Parameters(PutIdeaParams {
                        score: 6,
                        description: "an idea, revised".to_string(),
                        tag: None,
                        id: Some(idea.clone()),
                        verbose: false,
                    }))
                    .await,
            ),
            &ideas_file,
        ),
        (
            "list_ideas",
            body(
                &server
                    .pjpd_list_ideas(Parameters(ListIdeasParams {
                        count: 20,
                        verbose: false,
                    }))
                    .await,
            ),
            &ideas_file,
        ),
        (
            "mark_idea_done",
            body(
                &server
                    .pjpd_mark_idea_done(Parameters(MarkIdeaDoneParams {
                        idea_ids: vec![idea],
                        verbose: false,
                    }))
                    .await,
            ),
            &ideas_file,
        ),
    ];

    for (call, result, expected) in results {
        assert_eq!(result["project_file"], expected, "{call}");
    }
}

// R-24: a failure from every tool has `result: ""` and a non-empty error.
#[tokio::test]
async fn every_tool_failure_has_empty_result() {
    let (_dir, server) = server();
    let ghost = || vec!["ghost-ab29".to_string()];
    let failures = [
        server
            .pjpd_put_task(Parameters(PutTaskParams {
                description: "".to_string(),
                tag: Some("t".to_string()),
                id: None,
                priority: None,
                verbose: false,
            }))
            .await,
        server
            .pjpd_list_tasks(Parameters(ListTasksParams {
                count: 0,
                status: "todo".to_string(),
                verbose: false,
            }))
            .await,
        server
            .pjpd_reprioritize_task(Parameters(ReprioritizeTaskParams {
                priority: 5,
                task_ids: ghost(),
                verbose: false,
            }))
            .await,
        server
            .pjpd_list_ideas(Parameters(ListIdeasParams {
                count: 0,
                verbose: false,
            }))
            .await,
        server
            .pjpd_put_idea(Parameters(PutIdeaParams {
                score: 5,
                description: "x".to_string(),
                tag: None,
                id: Some("ghost-ab29".to_string()),
                verbose: false,
            }))
            .await,
        server
            .pjpd_mark_idea_done(Parameters(MarkIdeaDoneParams {
                idea_ids: ghost(),
                verbose: false,
            }))
            .await,
    ];
    let mark = mark_tasks(&server, &["ghost-ab29"], "done").await;
    let responses = failures.iter().map(envelope).chain(std::iter::once(mark));
    for response in responses {
        assert_eq!(response["success"], false, "{response}");
        assert_eq!(response["result"], "", "{response}");
        assert!(
            !response["error"].as_str().unwrap().is_empty(),
            "{response}"
        );
    }
}
