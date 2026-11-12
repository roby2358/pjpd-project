//! R-7: generated IDs are `<tag>-` plus ID_SUFFIX_LEN chars of ID_ALPHABET;
//! the valid-ID check matches `^[a-zA-Z0-9\-]+-[a-z2-9]{4}$`; tags are
//! 1..TAG_MAX_LEN chars of `[A-Za-z0-9-]`.

use pjpd::ids::{ID_ALPHABET, ID_SUFFIX_LEN, TAG_MAX_LEN, generate_with_tag, is_valid_id};

// R-7: a generated ID is the tag, a hyphen and ID_SUFFIX_LEN ID_ALPHABET
// chars, and validates.
#[test]
fn tag_id_has_expected_shape() {
    let id = generate_with_tag("mytag").unwrap();
    assert!(id.starts_with("mytag-"));
    assert!(is_valid_id(&id), "generated id should validate: {id}");
    let suffix = &id["mytag-".len()..];
    assert_eq!(suffix.len(), ID_SUFFIX_LEN);
    assert!(suffix.bytes().all(|b| ID_ALPHABET.contains(&b)));
}

// R-7: empty, too-long, space and underscore tags fail; hyphens pass.
#[test]
fn rejects_invalid_tags() {
    assert!(generate_with_tag("").is_err());
    assert!(generate_with_tag(&"t".repeat(TAG_MAX_LEN)).is_ok());
    assert!(generate_with_tag(&"t".repeat(TAG_MAX_LEN + 1)).is_err());
    assert!(generate_with_tag("no spaces").is_err());
    assert!(generate_with_tag("under_score").is_err());
    assert!(generate_with_tag("ok-tag").is_ok());
}

// R-7: the valid-ID check matches the pattern exactly.
#[test]
fn id_pattern_matches_python_regex() {
    assert!(is_valid_id("tag-ab29"));
    assert!(is_valid_id("multi-part-tag-wxyz"));
    assert!(is_valid_id("Tag-abcd"));
    assert!(!is_valid_id("tag-AB12")); // uppercase suffix
    assert!(!is_valid_id("tag-ab1")); // suffix too short
    assert!(!is_valid_id("tag-ab120")); // 0 not allowed in suffix
    assert!(!is_valid_id("-abcd")); // empty prefix
    assert!(!is_valid_id("tag_x-abcd")); // underscore in prefix
}
