//! R-16: `pjpd_list_tasks` lists the tasks saved to the file its `status`
//! filter names (`todo` by default), sorts status `ToDo` first, then priority
//! desc, and truncates to `count` with `info` naming returned and total.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, list_tasks, mark_tasks, response_json, server};
use pjpd::server::ListTasksParams;
use rmcp::handler::server::wrapper::Parameters;

// R-16: highest priority first, truncated to count, info names 2 of 3.
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
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["total_tasks"], 2);
    let descriptions: Vec<&str> = body["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["description"].as_str().unwrap())
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
