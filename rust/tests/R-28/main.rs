//! R-28: the tool parameter JSON schemas advertise the bounds that request
//! validation enforces (VALUE_MIN..VALUE_MAX, COUNT_MIN..COUNT_MAX, tag length
//! 1..TAG_MAX_LEN, non-empty description and ID lists).

use pjpd::ids::TAG_MAX_LEN;
use pjpd::server::{
    COUNT_MAX, COUNT_MIN, ListIdeasParams, ListTasksParams, MarkIdeaDoneParams, MarkTaskParams,
    PutIdeaParams, PutTaskParams, ReprioritizeTaskParams, VALUE_MAX, VALUE_MIN,
};
use rmcp::handler::server::common::schema_for_type;
use schemars::JsonSchema;
use serde_json::{Value, json};

/// The schema of one property, as rmcp advertises it in the tool list.
fn property<T: JsonSchema + 'static>(name: &str) -> Value {
    let schema = schema_for_type::<T>();
    schema["properties"][name].clone()
}

fn assert_bound<T: JsonSchema + 'static>(name: &str, key: &str, value: i64) {
    let prop = property::<T>(name);
    assert_eq!(prop[key], json!(value), "{name}.{key} in {prop}");
}

// R-28: priority and score advertise VALUE_MIN..VALUE_MAX.
#[test]
fn priority_and_score_ranges() {
    assert_bound::<PutTaskParams>("priority", "minimum", VALUE_MIN);
    assert_bound::<PutTaskParams>("priority", "maximum", VALUE_MAX);
    assert_bound::<ReprioritizeTaskParams>("priority", "minimum", VALUE_MIN);
    assert_bound::<ReprioritizeTaskParams>("priority", "maximum", VALUE_MAX);
    assert_bound::<PutIdeaParams>("score", "minimum", VALUE_MIN);
    assert_bound::<PutIdeaParams>("score", "maximum", VALUE_MAX);
}

// R-28: count advertises COUNT_MIN..COUNT_MAX on both list tools.
#[test]
fn count_ranges() {
    assert_bound::<ListTasksParams>("count", "minimum", COUNT_MIN);
    assert_bound::<ListTasksParams>("count", "maximum", COUNT_MAX);
    assert_bound::<ListIdeasParams>("count", "minimum", COUNT_MIN);
    assert_bound::<ListIdeasParams>("count", "maximum", COUNT_MAX);
}

// R-28: tag advertises length 1..TAG_MAX_LEN; description at least 1.
#[test]
fn string_lengths() {
    assert_bound::<PutTaskParams>("tag", "minLength", 1);
    assert_bound::<PutTaskParams>("tag", "maxLength", TAG_MAX_LEN as i64);
    assert_bound::<PutIdeaParams>("tag", "minLength", 1);
    assert_bound::<PutIdeaParams>("tag", "maxLength", TAG_MAX_LEN as i64);
    assert_bound::<PutTaskParams>("description", "minLength", 1);
    assert_bound::<PutIdeaParams>("description", "minLength", 1);
}

// R-28: ID lists advertise at least one item.
#[test]
fn id_lists_non_empty() {
    assert_bound::<MarkTaskParams>("task_ids", "minItems", 1);
    assert_bound::<ReprioritizeTaskParams>("task_ids", "minItems", 1);
    assert_bound::<MarkIdeaDoneParams>("idea_ids", "minItems", 1);
}

// R-28: descriptions no longer say values outside 0-100 are unbounded.
#[test]
fn descriptions_name_the_enforced_range() {
    let descriptions = [
        property::<PutTaskParams>("priority"),
        property::<ReprioritizeTaskParams>("priority"),
        property::<PutIdeaParams>("score"),
    ];
    descriptions.iter().for_each(|prop| {
        let text = prop["description"].as_str().unwrap();
        assert!(text.contains(&VALUE_MAX.to_string()), "{text}");
        assert!(!text.contains("outside"), "{text}");
    });
}
