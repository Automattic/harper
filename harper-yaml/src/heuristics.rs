/// Decides whether a YAML scalar value looks like prose worth
/// grammar-checking, as opposed to structural config data
/// (identifiers, enum values, URLs, paths, version strings).
///
/// Comments are never passed through this function — they're
/// always linted, handled separately in `YamlMasker`.
pub(crate) fn is_prose_scalar(text: &str) -> bool {
    let word_count = text.split_whitespace().count();

    word_count >= 3 && !looks_url_path_or_version(text) && !looks_like_identifier(text)
}

/// True only if `text` is, in its entirety, a single whitespace-separated
/// token that is itself URL-, path-, or version-shaped (e.g. `v1.2.3`,
/// `0.15.9`, `https://example.com`, `/usr/local/bin`) — not merely a
/// sentence that happens to mention one (e.g. "Update the library to
/// version 1.2.3" is prose worth checking, not a bare version string).
///
/// A value with 3+ words already fails `is_prose_scalar`'s word-count
/// gate before this function's result can matter, so in practice this
/// only ever returns `true` for single-token values. Kept explicit and
/// correct on its own terms rather than relying on that gate implicitly.
fn looks_url_path_or_version(text: &str) -> bool {
    let mut words = text.split_whitespace();

    let Some(only_word) = words.next() else {
        return false;
    };

    if words.next().is_some() {
        return false;
    }

    only_word.starts_with("http://")
        || only_word.starts_with("https://")
        || only_word.starts_with("ftp://")
        || only_word.contains('/')
        || only_word.contains('\\')
        || is_semver_like(only_word)
}

/// Matches strings shaped like `1.2.3`, `v1.2.3`, or `12.34.56.7`
/// (semver-ish): an optional leading `v`/`V`, then digit groups
/// separated by dots, at least two dots, no other characters.
fn is_semver_like(word: &str) -> bool {
    let trimmed = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    let trimmed = trimmed.strip_prefix(['v', 'V']).unwrap_or(trimmed);
    let parts: Vec<&str> = trimmed.split('.').collect();

    parts.len() >= 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// True if the whole value reads as an identifier/enum/constant
/// rather than a sentence: entirely uppercase, or snake_case, or
/// kebab-case.
fn looks_like_identifier(text: &str) -> bool {
    let has_letters = text.chars().any(|c| c.is_alphabetic());

    if !has_letters {
        return false;
    }

    let is_all_caps = text.chars().any(|c| c.is_alphabetic())
        && text
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase());

    if is_all_caps {
        return true;
    }

    is_snake_or_kebab_case(text)
}

/// True if the entire string is a single token of the shape
/// `word(_word)*` or `word(-word)*` with no spaces — i.e. it reads
/// as one identifier, not a sentence.
fn is_snake_or_kebab_case(text: &str) -> bool {
    if text.contains(char::is_whitespace) {
        return false;
    }

    let is_snake = text.contains('_')
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    let is_kebab = text.contains('-')
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    is_snake || is_kebab
}

#[cfg(test)]
mod tests {
    use super::is_prose_scalar;

    #[test]
    fn accepts_plain_sentence() {
        assert!(is_prose_scalar("The quick brown fox jumps"));
    }

    #[test]
    fn accepts_multiline_description() {
        assert!(is_prose_scalar(
            "This service handles user authentication and session\nmanagement for the platform."
        ));
    }

    #[test]
    fn rejects_single_word() {
        assert!(!is_prose_scalar("enabled"));
    }

    #[test]
    fn rejects_two_words() {
        assert!(!is_prose_scalar("not found"));
    }

    #[test]
    fn rejects_url() {
        assert!(!is_prose_scalar("https://example.com/some/long/path/here"));
    }

    #[test]
    fn accepts_sentence_containing_embedded_url() {
        // A URL mentioned inside a real sentence is prose worth
        // checking, not a bare URL value.
        assert!(is_prose_scalar("See http://example.com for more details"));
    }

    #[test]
    fn rejects_filesystem_path() {
        assert!(!is_prose_scalar("/usr/local/bin/some/long/tool/path"));
    }

    #[test]
    fn rejects_windows_path() {
        assert!(!is_prose_scalar(r"C:\Users\rod\some\long\windows\path"));
    }

    #[test]
    fn accepts_sentence_containing_embedded_path() {
        // A path mentioned inside a real sentence is prose worth
        // checking, not a bare path value.
        assert!(is_prose_scalar(
            "Update the config at /etc/config.yaml please"
        ));
    }

    #[test]
    fn rejects_bare_semver_value() {
        assert!(!is_prose_scalar("1.2.3"));
    }

    #[test]
    fn rejects_bare_v_prefixed_semver_value() {
        assert!(!is_prose_scalar("v1.2.3"));
    }

    #[test]
    fn accepts_sentence_containing_embedded_semver() {
        // A version number mentioned inside a real sentence is prose
        // worth checking, not a bare version string - only a value
        // that is *solely* a version string should be skipped.
        assert!(is_prose_scalar("please upgrade to 1.2.3 today now"));
    }

    #[test]
    fn rejects_all_caps() {
        assert!(!is_prose_scalar("THIS IS A CONSTANT VALUE HERE"));
    }

    #[test]
    fn rejects_snake_case() {
        assert!(!is_prose_scalar("this_is_a_snake_case_identifier"));
    }

    #[test]
    fn rejects_kebab_case() {
        assert!(!is_prose_scalar("this-is-a-kebab-case-identifier"));
    }

    #[test]
    fn accepts_sentence_with_hyphenated_word() {
        assert!(is_prose_scalar(
            "This is a well-known and widely-used approach"
        ));
    }

    #[test]
    fn accepts_sentence_containing_a_period() {
        assert!(is_prose_scalar("Restart the service. It will recover."));
    }

    #[test]
    fn rejects_empty_string() {
        assert!(!is_prose_scalar(""));
    }

    #[test]
    fn rejects_whitespace_only() {
        assert!(!is_prose_scalar("   \n  "));
    }
}
