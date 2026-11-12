//! Record ID generation and format checks.
//!
//! IDs use the tag-based format `<tag>-XXXX` where XXXX are base32 characters.
//! The legacy format `XX-XXXX-XX` is still generated for malformed records.

/// Base32 alphabet: a-z, 2-9 (excluding 1, l, o for visual clarity).
const BASE32_CHARS: &[u8] = b"abcdefghijkmnpqrstuvwxyz23456789";

fn random_base32(count: usize) -> String {
    (0..count)
        .map(|_| BASE32_CHARS[fastrand::usize(..BASE32_CHARS.len())] as char)
        .collect()
}

/// A tag is 1-12 characters, alphanumeric and hyphens only.
pub fn is_valid_tag(tag: &str) -> bool {
    let len = tag.chars().count();
    (1..=12).contains(&len) && tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// An ID matches `<tag>-XXXX`: a non-empty alphanumeric/hyphen prefix followed
/// by a hyphen and exactly 4 characters from [a-z2-9].
pub fn is_valid_id(id: &str) -> bool {
    let chars: Vec<char> = id.chars().collect();
    if chars.len() < 6 {
        return false;
    }
    let (prefix, tail) = chars.split_at(chars.len() - 5);
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

/// Generate a tag-based record ID in the format `<tag>-XXXX`.
pub fn generate_with_tag(tag: &str) -> Result<String, String> {
    if !is_valid_tag(tag) {
        return Err(
            "Tag must be 1-12 characters long and contain only alphanumeric characters and hyphens"
                .to_string(),
        );
    }
    Ok(format!("{tag}-{}", random_base32(4)))
}
