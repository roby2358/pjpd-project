//! R-23: when `pjpd/<dir-name>.txt` exists, task tool results include `warning`.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, create_task, response_json, server};
use pjpd::server::ListTasksParams;
use rmcp::handler::server::wrapper::Parameters;

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
