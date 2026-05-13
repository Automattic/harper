use super::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, Span};

#[derive(Debug, Default)]
pub struct IncludingButNotLimitedTo;

impl Linter for IncludingButNotLimitedTo {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let source = document.get_source();
        let mut lints = Vec::new();
        let mut i = 0;

        while i < source.len() {
            let Some(found) = find_word(source, "including", i) else {
                break;
            };

            if let Some(lint) = lint_match(source, found) {
                i = lint.span.end;
                lints.push(lint);
            } else {
                i = found + 1;
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Adds the commas customarily used around `including, but not limited to,` parenthetical clauses."
    }
}

fn lint_match(source: &[char], start: usize) -> Option<Lint> {
    let mut cursor = start;
    cursor = consume_word(source, cursor, "including")?;

    let has_comma_after_including = consume_comma(source, &mut cursor);
    cursor = consume_required_space(source, cursor)?;
    cursor = consume_word(source, cursor, "but")?;
    cursor = consume_required_space(source, cursor)?;
    cursor = consume_word(source, cursor, "not")?;
    cursor = consume_required_space(source, cursor)?;
    cursor = consume_word(source, cursor, "limited")?;
    cursor = consume_required_space(source, cursor)?;
    cursor = consume_word(source, cursor, "to")?;
    let has_comma_after_to = consume_comma(source, &mut cursor);

    let needs_leading_comma = needs_comma_before(source, start);

    if !needs_leading_comma && has_comma_after_including && has_comma_after_to {
        return None;
    }

    let span_start = if needs_leading_comma {
        source[..start]
            .iter()
            .rposition(|ch| !ch.is_whitespace())
            .map(|idx| idx + 1)
            .unwrap_or(start)
    } else {
        start
    };

    let including: String = source[start..start + "including".chars().count()]
        .iter()
        .collect();
    let mut replacement = String::new();

    if needs_leading_comma {
        replacement.push_str(", ");
    }

    replacement.push_str(&including);
    replacement.push_str(", but not limited to,");

    Some(Lint {
        span: Span::new(span_start, cursor),
        lint_kind: LintKind::Punctuation,
        suggestions: vec![Suggestion::ReplaceWith(replacement.chars().collect())],
        message: "Set off `including, but not limited to,` with commas.".to_owned(),
        priority: 31,
    })
}

fn find_word(source: &[char], word: &str, from: usize) -> Option<usize> {
    (from..source.len()).find(|&idx| consume_word(source, idx, word).is_some())
}

fn consume_word(source: &[char], start: usize, word: &str) -> Option<usize> {
    let chars: Vec<char> = word.chars().collect();
    let end = start.checked_add(chars.len())?;

    if end > source.len() {
        return None;
    }

    if start > 0 && is_word_char(source[start - 1]) {
        return None;
    }

    if end < source.len() && is_word_char(source[end]) {
        return None;
    }

    source[start..end]
        .iter()
        .zip(chars.iter())
        .all(|(actual, expected)| actual.eq_ignore_ascii_case(expected))
        .then_some(end)
}

fn consume_required_space(source: &[char], start: usize) -> Option<usize> {
    let mut cursor = start;

    while source.get(cursor).is_some_and(|ch| ch.is_whitespace()) {
        cursor += 1;
    }

    (cursor > start).then_some(cursor)
}

fn consume_comma(source: &[char], cursor: &mut usize) -> bool {
    let mut scan = *cursor;

    while source.get(scan).is_some_and(|ch| ch.is_whitespace()) {
        scan += 1;
    }

    if source.get(scan) == Some(&',') {
        scan += 1;
        *cursor = scan;
        true
    } else {
        false
    }
}

fn needs_comma_before(source: &[char], start: usize) -> bool {
    let Some(prev) = source[..start].iter().rposition(|ch| !ch.is_whitespace()) else {
        return false;
    };

    !matches!(
        source[prev],
        ',' | '(' | '[' | '{' | ';' | ':' | '-' | '—' | '–' | '`' | '"' | '\''
    )
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '\'' || ch == '-'
}

#[cfg(test)]
mod tests {
    use super::IncludingButNotLimitedTo;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn adds_internal_commas() {
        assert_suggestion_result(
            "This applies to many activities including but not limited to running.",
            IncludingButNotLimitedTo,
            "This applies to many activities, including, but not limited to, running.",
        );
    }

    #[test]
    fn adds_missing_leading_comma() {
        assert_suggestion_result(
            "There are many activities including, but not limited to, running.",
            IncludingButNotLimitedTo,
            "There are many activities, including, but not limited to, running.",
        );
    }

    #[test]
    fn adds_missing_trailing_comma() {
        assert_suggestion_result(
            "The work includes, but is not limited to, maintenance including, but not limited to cleanup.",
            IncludingButNotLimitedTo,
            "The work includes, but is not limited to, maintenance, including, but not limited to, cleanup.",
        );
    }

    #[test]
    fn allows_fully_punctuated_phrase() {
        assert_no_lints(
            "There are many activities, including, but not limited to, running.",
            IncludingButNotLimitedTo,
        );
    }

    #[test]
    fn preserves_initial_capitalization_at_sentence_start() {
        assert_suggestion_result(
            "Including but not limited to running, the list is long.",
            IncludingButNotLimitedTo,
            "Including, but not limited to, running, the list is long.",
        );
    }
}
