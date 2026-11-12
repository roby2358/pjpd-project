//! R-23: when `pjpd/<dir-name>.txt` exists, task tool results include
//! `warning`; idea tool results never do; there is no warning when that file
//! is `tasks.txt` itself.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, list_tasks, response_json, server};
use pjpd::server::{
    ListIdeasParams, ListTasksParams, MarkIdeaDoneParams, PjpdServer, PutIdeaParams,
};
use rmcp::handler::server::wrapper::Parameters;
use tempfile::TempDir;

// R-23: a legacy project file makes pjpd_list_tasks carry a warning.
#[tokio::test]
async fn legacy_project_file_produces_warning() {
    let (dir, server) = server();
    let dir_name = dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    std::fs::create_dir_all(dir.path().join("pjpd")).unwrap();
    std::fs::write(
        dir.path().join("pjpd").join(format!("{dir_name}.txt")),
        "old stuff",
    )
    .unwrap();

    create_task(&server, "new task", "new", 5).await;
    let result = server
        .pjpd_list_tasks(Parameters(ListTasksParams {
            count: 20,
            status: "todo".to_string(),
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert!(
        body["warning"]
            .as_str()
            .unwrap()
            .contains("Legacy project file")
    );
}

/// Write the old-layout `pjpd/<dir-name>.txt` file into the project.
fn write_legacy_file(dir: &TempDir) {
    let dir_name = dir
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    std::fs::create_dir_all(dir.path().join("pjpd")).unwrap();
    std::fs::write(
        dir.path().join("pjpd").join(format!("{dir_name}.txt")),
        "old stuff",
    )
    .unwrap();
}

// R-23: with a legacy project file present, no idea tool result carries a
// warning.
#[tokio::test]
async fn idea_tools_never_warn() {
    let (dir, server) = server();
    write_legacy_file(&dir);

    let created = assert_success(&response_json(
        &server
            .pjpd_put_idea(Parameters(PutIdeaParams {
                score: 5,
                description: "an idea".to_string(),
                tag: Some("idea".to_string()),
                id: None,
                verbose: false,
            }))
            .await,
    ));
    let id = created["id"].as_str().unwrap().to_string();
    let updated = assert_success(&response_json(
        &server
            .pjpd_put_idea(Parameters(PutIdeaParams {
                score: 6,
                description: "a better idea".to_string(),
                tag: None,
                id: Some(id.clone()),
                verbose: false,
            }))
            .await,
    ));
    let listed = assert_success(&response_json(
        &server
            .pjpd_list_ideas(Parameters(ListIdeasParams {
                count: 20,
                verbose: false,
            }))
            .await,
    ));
    let marked = assert_success(&response_json(
        &server
            .pjpd_mark_idea_done(Parameters(MarkIdeaDoneParams {
                idea_ids: vec![id],
                verbose: false,
            }))
            .await,
    ));
    for body in [created, updated, listed, marked] {
        assert!(body.get("warning").is_none(), "unexpected warning: {body}");
    }
    // The legacy file is still there, so task tools do warn.
    assert!(list_tasks(&server, "todo").await["warning"].is_string());
}

// R-23: in a project directory named `tasks`, the legacy file name is
// tasks.txt itself, so task tools give no warning.
#[tokio::test]
async fn no_warning_when_legacy_name_is_tasks_file() {
    let parent = TempDir::new().unwrap();
    let project = parent.path().join("tasks");
    std::fs::create_dir_all(&project).unwrap();
    let server = PjpdServer::new(project.clone());

    create_task(&server, "a task", "task", 5).await;
    assert!(project.join("pjpd/tasks.txt").exists());
    let listed = list_tasks(&server, "todo").await;
    assert!(
        listed.get("warning").is_none(),
        "unexpected warning: {listed}"
    );
}
