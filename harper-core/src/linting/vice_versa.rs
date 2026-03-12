use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::{Token, TokenStringExt};

fn matches_hyphen(token: &Token, _source: &[char]) -> bool {
    token.kind.is_hyphen()
}

fn replacement_for(template: &[char]) -> Vec<char> {
    let mut replacement = "vice versa".chars().collect::<Vec<_>>();

    let mut has_upper = false;
    let mut has_lower = false;
    let mut first_alpha_upper = None;

    for ch in template.iter().copied() {
        if !ch.is_alphabetic() {
            continue;
        }

        has_upper |= ch.is_uppercase();
        has_lower |= ch.is_lowercase();

        if first_alpha_upper.is_none() {
            first_alpha_upper = Some(ch.is_uppercase());
        }
    }

    if has_upper && !has_lower {
        for ch in replacement.iter_mut() {
            if ch.is_alphabetic() {
                *ch = ch.to_ascii_uppercase();
            }
        }

        return replacement;
    }

    if !has_upper {
        return replacement;
    }

    if first_alpha_upper.unwrap_or(false) {
        let mut capitalized_first = false;

        for ch in replacement.iter_mut() {
            if !ch.is_alphabetic() {
                continue;
            }

            if !capitalized_first {
                *ch = ch.to_ascii_uppercase();
                capitalized_first = true;
            } else {
                *ch = ch.to_ascii_lowercase();
            }
        }

        return replacement;
    }

    for ch in replacement.iter_mut() {
        if ch.is_alphabetic() {
            *ch = ch.to_ascii_lowercase();
        }
    }

    replacement
}

pub struct ViceVersa {
    expr: Box<dyn Expr>,
}

impl Default for ViceVersa {
    fn default() -> Self {
        let expr = SequenceExpr::word_set(&["vice", "vise"])
            .then(matches_hyphen)
            .then_optional(SequenceExpr::aco("a").then(matches_hyphen))
            .t_aco("versa");

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ViceVersa {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let template = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![Suggestion::ReplaceWith(replacement_for(template))],
            message: "The expression \"vice versa\" is spelled without hyphens.".to_owned(),
            priority: 60,
        })
    }

    fn description(&self) -> &str {
        "Recommends writing ‘vice versa’ without hyphens."
    }
}

#[cfg(test)]
mod tests {
    use super::ViceVersa;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_basic_hyphenated() {
        assert_suggestion_result(
            "We swapped the arguments vice-versa this time.",
            ViceVersa::default(),
            "We swapped the arguments vice versa this time.",
        );
    }

    #[test]
    fn corrects_leading_capitalization() {
        assert_suggestion_result(
            "Vice-Versa, the movie, was interesting.",
            ViceVersa::default(),
            "Vice versa, the movie, was interesting.",
        );
    }

    #[test]
    fn corrects_all_caps() {
        assert_suggestion_result(
            "They agreed VICE-VERSA on the clause.",
            ViceVersa::default(),
            "They agreed VICE VERSA on the clause.",
        );
    }

    #[test]
    fn corrects_with_extra_a() {
        assert_suggestion_result(
            "The logic works vice-a-versa as well.",
            ViceVersa::default(),
            "The logic works vice versa as well.",
        );
    }

    #[test]
    fn corrects_vise_variant() {
        assert_suggestion_result(
            "The rule applies vise-versa too.",
            ViceVersa::default(),
            "The rule applies vice versa too.",
        );
    }

    #[test]
    fn corrects_vise_extra_a_variant() {
        assert_suggestion_result(
            "The rule applies Vise-A-Versa too.",
            ViceVersa::default(),
            "The rule applies Vice versa too.",
        );
    }

    #[test]
    fn corrects_with_trailing_suffix() {
        assert_suggestion_result(
            "That was a vice-versa-like transformation.",
            ViceVersa::default(),
            "That was a vice versa-like transformation.",
        );
    }

    #[test]
    fn allows_correct_spelling() {
        assert_lint_count(
            "We swapped the arguments vice versa this time.",
            ViceVersa::default(),
            0,
        );
    }

    #[test]
    fn allows_sentence_case() {
        assert_lint_count(
            "Vice versa, the movie, was interesting.",
            ViceVersa::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_unrelated_words() {
        assert_lint_count(
            "Their service-versa mapping was custom.",
            ViceVersa::default(),
            0,
        );
    }
}
