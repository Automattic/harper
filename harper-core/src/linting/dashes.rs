use crate::expr::Expr;
use crate::expr::LongestMatchOf;
use crate::expr::SequenceExpr;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

const EN_DASH: char = '–';
const EM_DASH: char = '—';

pub struct Dashes {
    expr: LongestMatchOf,
}

impl Default for Dashes {
    fn default() -> Self {
        let en_dash = SequenceExpr::default().then_hyphen().then_hyphen();
        let em_dash_or_longer = SequenceExpr::default()
            .then_hyphen()
            .then_hyphen()
            .then_one_or_more_hyphens();

        let pattern = LongestMatchOf::new(vec![Box::new(em_dash_or_longer), Box::new(en_dash)]);

        Self { expr: pattern }
    }
}

impl ExprLinter for Dashes {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        _source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // The hyphens in ASCII arrows like `-->` and `<--` aren't dashes.
        if let Some((before, after)) = context
            && (before.last().is_some_and(|t| t.kind.is_less_than())
                || after.first().is_some_and(|t| t.kind.is_greater_than()))
        {
            return None;
        }

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
    use crate::linting::tests::{
        assert_no_lints, assert_suggestion_count, assert_suggestion_result,
    };

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

    #[test]
    fn ignores_right_arrow() {
        assert_no_lints("Go from A --> B now.", Dashes::default());
    }

    #[test]
    fn ignores_left_arrow() {
        assert_no_lints("Go from A <-- B now.", Dashes::default());
    }

    #[test]
    fn ignores_bidirectional_arrow() {
        assert_no_lints("Go from A <--> B now.", Dashes::default());
    }

    #[test]
    fn ignores_long_right_arrow() {
        assert_no_lints("Go from A ---> B now.", Dashes::default());
    }

    #[test]
    fn ignores_long_left_arrow() {
        assert_no_lints("Go from A <--- B now.", Dashes::default());
    }

    #[test]
    fn ignores_long_bidirectional_arrow() {
        assert_no_lints("Go from A <---> B now.", Dashes::default());
    }

    #[test]
    fn ignores_longer_right_arrow() {
        assert_no_lints("Go from A ----> B now.", Dashes::default());
    }

    #[test]
    fn ignores_single_hyphen_arrow() {
        assert_no_lints("Go from A -> B now.", Dashes::default());
    }

    #[test]
    fn ignores_unspaced_arrow() {
        assert_no_lints("Click File-->Save now.", Dashes::default());
    }

    #[test]
    fn ignores_arrow_at_start() {
        assert_no_lints("--> Start here.", Dashes::default());
    }

    #[test]
    fn ignores_arrow_at_end() {
        assert_no_lints("Start here -->", Dashes::default());
    }

    #[test]
    fn catches_dash_separated_from_angle_bracket() {
        assert_suggestion_result(
            "The result -- > 5 in most cases",
            Dashes::default(),
            &format!("The result {EN_DASH} > 5 in most cases"),
        );
    }

    #[test]
    fn catches_dash_alongside_arrow() {
        assert_suggestion_result(
            "Go from A --> B -- quickly",
            Dashes::default(),
            &format!("Go from A --> B {EN_DASH} quickly"),
        );
    }
}
