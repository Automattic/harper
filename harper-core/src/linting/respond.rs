use std::sync::Arc;

use crate::Token;
use crate::expr::{Expr, ExprMap, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};

pub struct Respond {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for Respond {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let helper_verb = |tok: &Token, src: &[char]| {
            if tok.kind.is_auxiliary_verb() {
                return true;
            }

            if !tok.kind.is_verb() {
                return false;
            }

            let lower = tok.span.get_content_string(src).to_lowercase();
            matches!(
                lower.as_str(),
                "do" | "did" | "does" | "won't" | "don't" | "didn't" | "doesn't"
            )
        };

        map.insert(
            SequenceExpr::default()
                .then_nominal()
                .t_ws()
                .then(helper_verb)
                .t_ws()
                .t_aco("response"),
            4,
        );

        map.insert(
            SequenceExpr::default()
                .then_nominal()
                .t_ws()
                .then(helper_verb)
                .t_ws()
                .then_adverb()
                .t_ws()
                .t_aco("response"),
            6,
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for Respond {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let response_index = *self.map.lookup(0, matched_tokens, source)?;
        let response_token = matched_tokens.get(response_index)?;

        Some(Lint {
            span: response_token.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "respond",
                response_token.span.get_content(source),
            )],
            message: "Use the verb `respond` here.".to_owned(),
            priority: 40,
        })
    }

    fn description(&self) -> &'static str {
        "Flags uses of the noun `response` where the verb `respond` is needed after an auxiliary."
    }
}

#[cfg(test)]
mod tests {
    use super::Respond;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn fixes_will_response() {
        assert_suggestion_result(
            "He will response soon.",
            Respond::default(),
            "He will respond soon.",
        );
    }

    #[test]
    fn fixes_can_response() {
        assert_suggestion_result(
            "They can response to the survey.",
            Respond::default(),
            "They can respond to the survey.",
        );
    }

    #[test]
    fn fixes_did_not_response() {
        assert_suggestion_result(
            "I did not response yesterday.",
            Respond::default(),
            "I did not respond yesterday.",
        );
    }

    #[test]
    fn fixes_might_quickly_response() {
        assert_suggestion_result(
            "She might quickly response to feedback.",
            Respond::default(),
            "She might quickly respond to feedback.",
        );
    }

    #[test]
    fn fixes_wont_response() {
        assert_suggestion_result(
            "They won't response in time.",
            Respond::default(),
            "They won't respond in time.",
        );
    }

    #[test]
    fn fixes_would_response() {
        assert_suggestion_result(
            "We would response if we could.",
            Respond::default(),
            "We would respond if we could.",
        );
    }

    #[test]
    fn fixes_should_response() {
        assert_suggestion_result(
            "You should response politely.",
            Respond::default(),
            "You should respond politely.",
        );
    }

    #[test]
    fn does_not_flag_correct_respond() {
        assert_no_lints("Please respond when you can.", Respond::default());
    }

    #[test]
    fn does_not_flag_noun_use() {
        assert_no_lints("The response time was great.", Respond::default());
    }

    #[test]
    fn does_not_flag_question_subject() {
        assert_lint_count("Should response times be logged?", Respond::default(), 0);
    }

    #[test]
    fn does_not_flag_response_as_object() {
        assert_no_lints("I have no response for that.", Respond::default());
    }
}
