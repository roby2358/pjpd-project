//! R-22: `pjpd_mark_idea_done` prefixes the first description line with
//! `(Done) ` once (never doubled) and keeps the score.

#[path = "../common/mod.rs"]
mod common;

use common::{assert_success, backups_of, clear_backups, response_json, server};
use pjpd::ideas::IdeaStore;
use pjpd::server::{ListIdeasParams, MarkIdeaDoneParams, PutIdeaParams};
use rmcp::handler::server::wrapper::Parameters;
use tempfile::TempDir;

// R-22: the tool prefixes the description and keeps the score, in the
// response and on disk.
#[tokio::test]
async fn mark_idea_done_prefixes_description_and_preserves_score() {
    let (dir, server) = server();
    let created = assert_success(&response_json(
        &server
            .pjpd_put_idea(Parameters(PutIdeaParams {
                score: 42,
                description: "keep my score".to_string(),
                tag: Some("keep".to_string()),
                id: None,
                verbose: false,
            }))
            .await,
    ));
    let id = created["id"].as_str().unwrap().to_string();

    let result = server
        .pjpd_mark_idea_done(Parameters(MarkIdeaDoneParams {
            idea_ids: vec![id.clone()],
            verbose: false,
        }))
        .await;
    let body = assert_success(&response_json(&result));
    assert_eq!(body["message"], "Marked 1 idea(s) as done");

    let listed = assert_success(&response_json(
        &server
            .pjpd_list_ideas(Parameters(ListIdeasParams {
                count: 20,
                verbose: false,
            }))
            .await,
    ));
    assert_eq!(listed["ideas"][0]["summary"], "(Done) keep my score");
    assert_eq!(listed["ideas"][0]["score"], 42);

    let on_disk = std::fs::read_to_string(dir.path().join("pjpd/ideas.txt")).unwrap();
    assert!(on_disk.contains("Score:   42"));
    assert!(on_disk.contains("(Done) keep my score"));
}

// R-22: marking an idea done in the store keeps its score.
#[test]
fn store_mark_done_preserves_score() {
    let dir = TempDir::new().unwrap();
    let store = IdeaStore::new(dir.path());

    let idea = store.add("finished thing", 90, "olddone").unwrap();
    store.mark_done(&idea.id).unwrap();

    assert_eq!(store.load()[0].score, 90);
}

// R-22: marking twice doesn't double the prefix.
#[test]
fn mark_done_does_not_double_prefix() {
    let dir = TempDir::new().unwrap();
    let store = IdeaStore::new(dir.path());

    let idea = store.add("once only", 5, "once").unwrap();
    store.mark_done(&idea.id).unwrap();
    store.mark_done(&idea.id).unwrap();

    let reloaded = store.load();
    assert_eq!(reloaded[0].description, "(Done) once only");
}

// R-22: marking several ideas done is one load and one save, so it leaves
// exactly one ideas.txt backup holding the file as it was before (D-16).
#[tokio::test]
async fn batch_saves_ideas_file_once() {
    let (dir, server) = server();
    let store = IdeaStore::new(dir.path());
    let ids: Vec<String> = ["one", "two", "three"]
        .iter()
        .map(|tag| store.add(tag, 5, tag).unwrap().id)
        .collect();

    let before = std::fs::read_to_string(dir.path().join("pjpd/ideas.txt")).unwrap();
    clear_backups(&dir);
    let body = assert_success(&response_json(
        &server
            .pjpd_mark_idea_done(Parameters(MarkIdeaDoneParams {
                idea_ids: ids,
                verbose: false,
            }))
            .await,
    ));
    assert_eq!(body["ideas"].as_array().unwrap().len(), 3);
    assert_eq!(backups_of(&dir, "ideas"), vec![before]);
}
