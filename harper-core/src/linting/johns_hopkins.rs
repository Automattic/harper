use crate::{
    CharStringExt, Token,
    expr::{Expr, SequenceExpr},
    linting::expr_linter::Chunk,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct JohnsHopkins {
    expr: Box<dyn Expr>,
}

impl Default for JohnsHopkins {
    fn default() -> Self {
        let expr = SequenceExpr::with(|tok: &Token, src: &[char]| {
            tok.kind.is_proper_noun() && tok.span.get_content(src).eq_ignore_ascii_case_str("john")
        })
        .t_ws()
        .then(|tok: &Token, src: &[char]| {
            tok.kind.is_proper_noun()
                && tok
                    .span
                    .get_content(src)
                    .eq_ignore_ascii_case_str("hopkins")
        });

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for JohnsHopkins {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;
        let template = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str("Johns", template)],
            message: "Use `Johns Hopkins` for this name.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Recommends the proper spelling `Johns Hopkins`."
    }
}

#[cfg(test)]
mod tests {
    use super::JohnsHopkins;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_university_reference() {
        assert_suggestion_result(
            "I applied to John Hopkins University last fall.",
            JohnsHopkins::default(),
            "I applied to Johns Hopkins University last fall.",
        );
    }

    #[test]
    fn corrects_hospital_reference() {
        assert_suggestion_result(
            "She works at the John Hopkins hospital.",
            JohnsHopkins::default(),
            "She works at the Johns Hopkins hospital.",
        );
    }

    #[test]
    fn corrects_standalone_name() {
        assert_suggestion_result(
            "We toured John Hopkins yesterday.",
            JohnsHopkins::default(),
            "We toured Johns Hopkins yesterday.",
        );
    }

    #[test]
    fn corrects_lowercase_usage() {
        assert_suggestion_result(
            "I studied at john hopkins online.",
            JohnsHopkins::default(),
            "I studied at johns hopkins online.",
        );
    }

    #[test]
    fn corrects_across_newline_whitespace() {
        assert_suggestion_result(
            "We met at John\nHopkins for lunch.",
            JohnsHopkins::default(),
            "We met at Johns\nHopkins for lunch.",
        );
    }

    #[test]
    fn corrects_with_trailing_punctuation() {
        assert_suggestion_result(
            "I toured John Hopkins, and it was great.",
            JohnsHopkins::default(),
            "I toured Johns Hopkins, and it was great.",
        );
    }

    #[test]
    fn corrects_before_hyphenated_unit() {
        assert_suggestion_result(
            "She joined the John Hopkins-affiliated lab.",
            JohnsHopkins::default(),
            "She joined the Johns Hopkins-affiliated lab.",
        );
    }

    #[test]
    fn allows_correct_spelling() {
        assert_lint_count(
            "Johns Hopkins University has a great program.",
            JohnsHopkins::default(),
            0,
        );
    }

    #[test]
    fn allows_apostrophized_form() {
        assert_lint_count(
            "John Hopkins's novel won awards.",
            JohnsHopkins::default(),
            0,
        );
    }

    #[test]
    fn allows_reversed_name_order() {
        assert_lint_count("Hopkins, John is a contact.", JohnsHopkins::default(), 0);
    }
}
