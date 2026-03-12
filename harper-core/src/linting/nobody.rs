use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct Nobody {
    expr: Box<dyn Expr>,
}

impl Default for Nobody {
    fn default() -> Self {
        let pattern = SequenceExpr::aco("no")
            .then_whitespace()
            .t_aco("body")
            .then_whitespace()
            .then_verb();
        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for Nobody {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens[0..3].span()?;
        let orig_chars = span.get_content(source);

        if next_non_whitespace_char(source, span.end).is_some_and(|ch| ch == ',') {
            return None;
        }

        if next_non_whitespace_word(source, span.end).is_some_and(|word| {
            matches!(
                word.as_str(),
                "is" | "was" | "were" | "be" | "been" | "being"
            )
        }) {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "nobody".chars().collect(),
                orig_chars,
            )],
            message: format!("Did you mean the closed compound `{}`?", "nobody"),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Looks for incorrect spacing inside the closed compound `nobody`."
    }
}

fn next_non_whitespace_char(source: &[char], offset: usize) -> Option<char> {
    source
        .get(offset..)?
        .iter()
        .find(|c| !c.is_whitespace())
        .copied()
}

fn next_non_whitespace_word(source: &[char], offset: usize) -> Option<String> {
    let suffix = source.get(offset..)?;
    let mut iter = suffix
        .iter()
        .enumerate()
        .skip_while(|(_, c)| c.is_whitespace());
    let start = iter.next()?.0;
    let end = suffix[start..]
        .iter()
        .position(|c| c.is_whitespace() || c.is_ascii_punctuation())
        .map(|len| start + len)
        .unwrap_or(suffix.len());

    Some(
        suffix[start..end]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase(),
    )
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::Nobody;

    #[test]
    fn both_valid_and_invalid() {
        assert_suggestion_result(
            "No body told me. I have a head but no body.",
            Nobody::default(),
            "Nobody told me. I have a head but no body.",
        );
    }

    #[test]
    fn ignores_no_body_was_found() {
        assert_lint_count("No body was found after the search.", Nobody::default(), 0);
    }

    #[test]
    fn ignores_no_body_comma() {
        assert_lint_count(
            "No body, no signs of a struggle, no answers.",
            Nobody::default(),
            0,
        );
    }
}
