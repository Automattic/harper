use crate::expr::Expr;
use crate::expr::LongestMatchOf;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};

const EN_DASH: char = '–';
const EM_DASH: char = '—';

pub struct Dashes {
    expr: Box<dyn Expr>,
}

impl Default for Dashes {
    fn default() -> Self {
        let en_dash = SequenceExpr::default().then_hyphen().then_hyphen();
        let em_dash_or_longer = SequenceExpr::default()
            .then_hyphen()
            .then_hyphen()
            .then_one_or_more_hyphens();

        let pattern = LongestMatchOf::new(vec![Box::new(em_dash_or_longer), Box::new(en_dash)]);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for Dashes {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let lint_kind = LintKind::Formatting;

        match matched_tokens.len() {
            2 => Some(Lint {
                span,
                lint_kind,
                suggestions: vec![Suggestion::ReplaceWith(vec![EN_DASH])],
                message: "Replace these two hyphens with an en dash (–).".to_owned(),
                priority: 63,
            }),
            3 => Some(Lint {
                span,
                lint_kind,
                suggestions: vec![Suggestion::ReplaceWith(vec![EM_DASH])],
                message: "Replace these three hyphens with an em dash (—).".to_owned(),
                priority: 63,
            }),
            4.. => None, // Ignore longer hyphen sequences.
            _ => panic!("Received unexpected number of tokens."),
        }
    }

    fn description(&self) -> &'static str {
        "Writers often type `--` or `---` expecting their editor to convert them into proper dashes. Replace these sequences with the correct characters: use an en dash (–) for ranges or connections and an em dash (—) for a break in thought."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_suggestion_count, assert_suggestion_result};

    use super::Dashes;
    use super::{EM_DASH, EN_DASH};

    #[test]
    fn catches_en_dash() {
        assert_suggestion_result(
            "pre--Industrial Revolution",
            Dashes::default(),
            &format!("pre{EN_DASH}Industrial Revolution"),
        );
    }

    #[test]
    fn catches_em_dash() {
        assert_suggestion_result(
            "'There is no box' --- Scott",
            Dashes::default(),
            &format!("'There is no box' {EM_DASH} Scott"),
        );
    }

    #[test]
    fn no_overlaps() {
        assert_suggestion_count("'There is no box' --- Scott", Dashes::default(), 1);
    }

    #[test]
    fn no_lint_for_long_hyphen_sequences() {
        assert_suggestion_count("'There is no box' ------ Scott", Dashes::default(), 0);
    }
}
