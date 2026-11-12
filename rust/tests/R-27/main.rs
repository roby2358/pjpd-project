//! R-27: mutating tools return a lean result by default (each task's id,
//! status and priority; each idea's id and score; plus `project_file` and
//! `message`) and the full records only with `verbose: true` (D-14).

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, response_json, server};
use pjpd::server::{
    MarkIdeaDoneParams, MarkTaskParams, PjpdServer, PutIdeaParams, PutTaskParams,
    ReprioritizeTaskParams,
};
use rmcp::handler::server::wrapper::Parameters;
use serde_json::Value;

/// The sorted keys of a JSON object.
fn keys(value: &Value) -> Vec<String> {
    let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

async fn put_task(server: &PjpdServer, verbose: bool) -> Value {
    let result = server
        .pjpd_put_task(Parameters(PutTaskParams {
            description: "a long description".to_string(),
            tag: Some("lean".to_string()),
            id: None,
            priority: Some(40),
            verbose,
        }))
        .await;
    assert_success(&response_json(&result))
}

async fn put_idea(server: &PjpdServer, verbose: bool) -> Value {
    let result = server
        .pjpd_put_idea(Parameters(PutIdeaParams {
            score: 30,
            description: "an idea".to_string(),
            tag: Some("lean".to_string()),
            id: None,
            verbose,
        }))
        .await;
    assert_success(&response_json(&result))
}

async fn mark(server: &PjpdServer, id: &str, state: &str, verbose: bool) -> Value {
    let result = server
        .pjpd_mark_task(Parameters(MarkTaskParams {
            task_ids: vec![id.to_string()],
            state: state.to_string(),
            verbose,
        }))
        .await;
    assert_success(&response_json(&result))
}

// R-27: pjpd_put_task returns only id, status, priority, project_file and
// message unless verbose.
#[tokio::test]
async fn put_task_is_lean_unless_verbose() {
    let (_dir, server) = server();
    let lean = put_task(&server, false).await;
    assert_eq!(
        keys(&lean),
        names(&["id", "message", "priority", "project_file", "status"])
    );
    assert_eq!(lean["status"], "ToDo");
    assert_eq!(lean["priority"], 40);

    let full = put_task(&server, true).await;
    assert_eq!(full["description"], "a long description");
    assert!(full["created"].is_string() && full["updated"].is_string());
}

// R-27: pjpd_reprioritize_task lists each task by id, status and priority
// unless verbose.
#[tokio::test]
async fn reprioritize_is_lean_unless_verbose() {
    let (_dir, server) = server();
    let id = create_task(&server, "reprioritize me", "lean", 5).await;
    let call = |verbose| {
        server.pjpd_reprioritize_task(Parameters(ReprioritizeTaskParams {
            priority: 60,
            task_ids: vec![id.clone()],
            verbose,
        }))
    };

    let lean = assert_success(&response_json(&call(false).await));
    assert_eq!(keys(&lean), names(&["message", "project_file", "tasks"]));
    assert_eq!(
        keys(&lean["tasks"][0]),
        names(&["id", "priority", "status"])
    );
    assert_eq!(lean["tasks"][0]["priority"], 60);

    let full = assert_success(&response_json(&call(true).await));
    assert_eq!(full["tasks"][0]["description"], "reprioritize me");
}

// R-27: pjpd_mark_task done lists each task by id, status and priority
// unless verbose.
#[tokio::test]
async fn mark_task_done_is_lean_unless_verbose() {
    let (_dir, server) = server();
    let first = create_task(&server, "first", "lean", 5).await;
    let second = create_task(&server, "second", "lean", 6).await;

    let lean = mark(&server, &first, "done", false).await;
    assert_eq!(
        keys(&lean["tasks"][0]),
        names(&["id", "priority", "status"])
    );
    assert_eq!(lean["tasks"][0]["status"], "Done");

    let full = mark(&server, &second, "done", true).await;
    assert_eq!(full["tasks"][0]["description"], "second");
}

// R-27: pjpd_mark_task idea lists each new idea by id and score unless verbose.
#[tokio::test]
async fn demote_is_lean_unless_verbose() {
    let (_dir, server) = server();
    let first = create_task(&server, "first", "lean", 5).await;
    let second = create_task(&server, "second", "lean", 6).await;

    let lean = mark(&server, &first, "idea", false).await;
    assert_eq!(keys(&lean["ideas"][0]), names(&["id", "score"]));

    let full = mark(&server, &second, "idea", true).await;
    assert_eq!(full["ideas"][0]["description"], "second");
}

// R-27: pjpd_put_idea returns only id, score, project_file and message unless
// verbose.
#[tokio::test]
async fn put_idea_is_lean_unless_verbose() {
    let (_dir, server) = server();
    let lean = put_idea(&server, false).await;
    assert_eq!(
        keys(&lean),
        names(&["id", "message", "project_file", "score"])
    );
    assert_eq!(lean["score"], 30);

    let full = put_idea(&server, true).await;
    assert_eq!(full["description"], "an idea");
}

// R-27: pjpd_mark_idea_done lists each idea by id and score unless verbose.
#[tokio::test]
async fn mark_idea_done_is_lean_unless_verbose() {
    let (_dir, server) = server();
    let id = put_idea(&server, false).await["id"]
        .as_str()
        .unwrap()
        .to_string();
    let call = |verbose| {
        server.pjpd_mark_idea_done(Parameters(MarkIdeaDoneParams {
            idea_ids: vec![id.clone()],
            verbose,
        }))
    };

    let lean = assert_success(&response_json(&call(false).await));
    assert_eq!(keys(&lean), names(&["ideas", "message", "project_file"]));
    assert_eq!(keys(&lean["ideas"][0]), names(&["id", "score"]));
    assert_eq!(lean["ideas"][0]["id"], id.as_str());

    let full = assert_success(&response_json(&call(true).await));
    assert_eq!(full["ideas"][0]["description"], "(Done) an idea");
}
