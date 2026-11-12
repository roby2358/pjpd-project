//! R-16: `pjpd_list_tasks` lists the tasks saved to the file its `status`
//! filter names (`todo` by default), sorts status `ToDo` first, then priority
//! desc, and truncates to `count` with `info` naming returned and total;
//! `total_tasks` counts every matching task. Each task is a summary (id,
//! priority, status, `summary`) unless `verbose` asks for the full record.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, list_tasks, mark_tasks, response_json, server};
use pjpd::server::ListTasksParams;
use rmcp::handler::server::wrapper::Parameters;

// R-16: highest priority first, truncated to count, info names 2 of 3, and
// total_tasks counts all 3.
#[tokio::test]
async fn list_tasks_sorts_and_truncates() {
    let (_dir, server) = server();
    create_task(&server, "low", "low", 1).await;
    create_task(&server, "high", "high", 90).await;
    create_task(&server, "mid", "mid", 40).await;

    let result = server
        .pjpd_list_tasks(Parameters(ListTasksParams {
            count: 2,
            status: "todo".to_string(),
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["total_tasks"], 3);
    let descriptions: Vec<&str> = body["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["summary"].as_str().unwrap())
        .collect();
    assert_eq!(descriptions, vec!["high", "mid"]);
    assert_eq!(body["info"], "Returned 2 of 3 total tasks");
}

// R-16: the filter selects by file: a Blocked status lists under todo (after
// ToDo), Done only under done; the filter value is case-insensitive.
#[tokio::test]
async fn status_filter_selects_by_file() {
    let (dir, server) = server();
    let todo = create_task(&server, "plain", "plain", 1).await;
    let blocked = create_task(&server, "stuck", "stuck", 90).await;
    let done = create_task(&server, "finished", "finished", 50).await;
    assert_success(&mark_tasks(&server, &[&done], "done").await);
    let tasks_file = dir.path().join("pjpd/tasks.txt");
    let text = std::fs::read_to_string(&tasks_file).unwrap();
    std::fs::write(
        &tasks_file,
        text.replace("Status: ToDo\nID: stuck", "Status: Blocked\nID: stuck"),
    )
    .unwrap();

    let default = assert_success(&response_json(
        &server
            .pjpd_list_tasks(Parameters(ListTasksParams {
                count: 20,
                status: "todo".to_string(),
                verbose: false,
            }))
            .await,
    ));
    let ids: Vec<&str> = default["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![todo.as_str(), blocked.as_str()]);

    let finished = list_tasks(&server, "DONE").await;
    assert_eq!(finished["total_tasks"], 1);
    assert_eq!(finished["tasks"][0]["id"], done.as_str());
}

// R-16: omitting status lists todo tasks.
#[test]
fn status_defaults_to_todo() {
    let params: ListTasksParams = serde_json::from_str("{}").unwrap();
    assert_eq!(params.status, "todo");
    assert_eq!(params.count, 20);
}

// R-16: statuses match case-insensitively (D-6): a hand-edited `done` in
// tasks.txt lists under done, not todo, and a `todo` status sorts with ToDo.
#[tokio::test]
async fn hand_edited_status_case_lists_by_stage() {
    let (dir, server) = server();
    let low = create_task(&server, "low", "low", 1).await;
    let lower = create_task(&server, "lower", "lower", 1).await;
    let blocked = create_task(&server, "stuck", "stuck", 90).await;
    let done = create_task(&server, "finished", "finished", 50).await;
    let tasks_file = dir.path().join("pjpd/tasks.txt");
    let text = std::fs::read_to_string(&tasks_file).unwrap();
    let edited = text
        .replace(
            &format!("Status: ToDo\nID: {done}"),
            &format!("Status: done\nID: {done}"),
        )
        .replace(
            &format!("Status: ToDo\nID: {lower}"),
            &format!("Status: todo\nID: {lower}"),
        )
        .replace(
            &format!("Status: ToDo\nID: {blocked}"),
            &format!("Status: Blocked\nID: {blocked}"),
        );
    std::fs::write(&tasks_file, edited).unwrap();

    let todo = list_tasks(&server, "todo").await;
    let ids: Vec<&str> = todo["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![low.as_str(), lower.as_str(), blocked.as_str()]);

    let finished = list_tasks(&server, "done").await;
    assert_eq!(finished["total_tasks"], 1);
    assert_eq!(finished["tasks"][0]["id"], done.as_str());
    assert_eq!(finished["tasks"][0]["status"], "done");
}

// R-16: by default each task is id, priority, status and the description's
// first line cut to 100 characters, "..." marking omitted text; no
// description or timestamps.
#[tokio::test]
async fn default_listing_is_a_summary() {
    let (_dir, server) = server();
    let long = "x".repeat(150);
    create_task(&server, "one line", "short", 5).await;
    create_task(&server, "first line\nsecond line", "multi", 9).await;
    create_task(&server, &long, "long", 1).await;

    let body = list_tasks(&server, "todo").await;
    let tasks = body["tasks"].as_array().unwrap();
    let summaries: Vec<&str> = tasks
        .iter()
        .map(|t| t["summary"].as_str().unwrap())
        .collect();
    assert_eq!(
        summaries,
        vec![
            "first line...".to_string(),
            "one line".to_string(),
            format!("{}...", "x".repeat(100)),
        ]
    );
    let keys: Vec<&String> = tasks[0].as_object().unwrap().keys().collect();
    assert_eq!(keys, vec!["id", "priority", "status", "summary"]);
}

// R-16: verbose returns the full record, timestamps included.
#[tokio::test]
async fn verbose_listing_is_the_full_record() {
    let (_dir, server) = server();
    create_task(&server, "first line\nsecond line", "multi", 9).await;

    let body = assert_success(&response_json(
        &server
            .pjpd_list_tasks(Parameters(ListTasksParams {
                count: 20,
                status: "todo".to_string(),
                verbose: true,
            }))
            .await,
    ));
    let task = &body["tasks"][0];
    assert_eq!(task["description"], "first line\nsecond line");
    assert_eq!(task["priority"], 9);
    assert!(task["created"].is_string());
    assert!(task["updated"].is_string());
    assert!(task.get("summary").is_none());
}
