//! Record ID generation and format checks.
//!
//! IDs use the tag-based format `<tag>-XXXX` where XXXX are base32 characters.
//! The legacy format `XX-XXXX-XX` is still generated for malformed records,
//! and rewritten to `legacy-XXXX` when the record is saved (D-8).

/// ID_ALPHABET: a-z, 2-9 (excluding 1, l, o for visual clarity) (D-13).
pub const ID_ALPHABET: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";

/// ID_SUFFIX_LEN: characters of ID_ALPHABET after the tag's hyphen (D-13).
pub const ID_SUFFIX_LEN: usize = 4;

/// TAG_MAX_LEN: the longest tag, in characters (D-13).
pub const TAG_MAX_LEN: usize = 12;

fn random_base32(count: usize) -> String {
    (0..count)
        .map(|_| ID_ALPHABET[fastrand::usize(..ID_ALPHABET.len())] as char)
        .collect()
}

/// A tag is 1..TAG_MAX_LEN characters, alphanumeric and hyphens only.
pub fn is_valid_tag(tag: &str) -> bool {
    let len = tag.chars().count();
    (1..=TAG_MAX_LEN).contains(&len) && tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// An ID matches `<tag>-XXXX`: a non-empty alphanumeric/hyphen prefix followed
/// by a hyphen and exactly ID_SUFFIX_LEN characters from [a-z2-9].
pub fn is_valid_id(id: &str) -> bool {
    let chars: Vec<char> = id.chars().collect();
    if chars.len() < ID_SUFFIX_LEN + 2 {
        return false;
    }
    let (prefix, tail) = chars.split_at(chars.len() - (ID_SUFFIX_LEN + 1));
    tail[0] == '-'
        && tail[1..].iter().all(|c| matches!(c, 'a'..='z' | '2'..='9'))
        && !prefix.is_empty()
        && prefix
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || *c == '-')
}

/// Generate a legacy 10-character record ID in the format XX-XXXX-XX.
pub fn generate_legacy() -> String {
    let chars = random_base32(8);
    format!("{}-{}-{}", &chars[0..2], &chars[2..6], &chars[6..8])
}

/// The tag given to a record whose legacy ID is rewritten on save (D-8).
pub const LEGACY_TAG: &str = "legacy";

/// True for the legacy `XX-XXXX-XX` form: segments of 2, 4 and 2 characters
/// from the base32 alphabet.
pub fn is_legacy_id(id: &str) -> bool {
    let segments: Vec<&str> = id.split('-').collect();
    segments.iter().map(|s| s.len()).eq([2, 4, 2])
        && segments
            .iter()
            .all(|s| s.bytes().all(|b| ID_ALPHABET.contains(&b)))
}

/// A fresh `legacy-XXXX` ID to replace a legacy one.
pub fn generate_legacy_replacement() -> String {
    format!("{LEGACY_TAG}-{}", random_base32(ID_SUFFIX_LEN))
}

/// Generate a tag-based record ID in the format `<tag>-XXXX`.
pub fn generate_with_tag(tag: &str) -> Result<String, String> {
    if !is_valid_tag(tag) {
        return Err(format!(
            "Tag must be 1-{TAG_MAX_LEN} characters long and contain only alphanumeric characters and hyphens"
        ));
    }
    Ok(format!("{tag}-{}", random_base32(ID_SUFFIX_LEN)))
}
