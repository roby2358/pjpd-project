//! R-24: every response is `{success, result, error}` and every success result
//! includes `project_file` with the full path of the file operated on.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, response_json, server};
use pjpd::server::PutTaskParams;
use rmcp::handler::server::wrapper::Parameters;

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
