/// Decides whether a YAML scalar value looks like prose worth
/// grammar-checking, as opposed to structural config data
/// (identifiers, enum values, etc).
///
/// Comments are never passed through this function — they're
/// always linted, handled separately in `YamlMasker`.
pub(crate) fn is_prose_scalar(text: &str) -> bool {
    let text = strip_block_scalar_indicator(text);
    let word_count = text.split_whitespace().count();

    word_count >= 3 && !looks_like_identifier(text)
}

/// YAML block scalars (`|`, `>`, and their chomping/indent variants
/// like `|-`, `>+`, `|2`, `>-2`) are masked including their leading
/// indicator line. That indicator line is not part of the prose body
/// and shouldn't count toward the word-count gate, so strip it off
/// before measuring.
///
/// Only strips when the trimmed first line is *entirely* a block
/// scalar header token — a plain scalar that merely contains a
/// literal `|` or `>` mid-sentence is left untouched.
fn strip_block_scalar_indicator(text: &str) -> &str {
    let (first_line, rest) = match text.split_once('\n') {
        Some((first, rest)) => (first, Some(rest)),
        None => (text, None),
    };

    let trimmed = first_line.trim();

    let Some(after_marker) = trimmed
        .strip_prefix('|')
        .or_else(|| trimmed.strip_prefix('>'))
    else {
        return text;
    };

    let is_header = after_marker
        .chars()
        .all(|c| c == '+' || c == '-' || c.is_ascii_digit());

    if !is_header {
        return text;
    }

    rest.unwrap_or("")
}

/// True if the whole value reads as an identifier/enum/constant
/// rather than a sentence: entirely uppercase.
fn looks_like_identifier(text: &str) -> bool {
    let has_letters = text.chars().any(|c| c.is_alphabetic());

    if !has_letters {
        return false;
    }

    text.chars()
        .filter(|c| c.is_alphabetic())
        .all(|c| c.is_uppercase())
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

    #[test]
    fn rejects_block_scalar_with_short_body() {
        assert!(!is_prose_scalar("|\n  hello world"));
    }

    #[test]
    fn accepts_block_scalar_with_prose_body() {
        assert!(is_prose_scalar("|\n  the quick brown fox"));
    }

    #[test]
    fn rejects_folded_block_scalar_with_chomping_and_short_body() {
        assert!(!is_prose_scalar(">-\n  hello world"));
    }

    #[test]
    fn accepts_sentence_with_literal_pipe_not_a_block_header() {
        // The `|` here is mid-sentence, not a block scalar header on
        // its own line, so it must not be stripped.
        assert!(is_prose_scalar("run foo | grep bar please"));
    }
}
