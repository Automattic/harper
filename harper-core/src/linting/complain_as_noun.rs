use crate::{
    Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ComplainAsNoun {
    expr: All,
}

impl Default for ComplainAsNoun {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_determiner()
                .t_ws()
                .t_set(&["complain", "complains"])
                .but_not(SequenceExpr::anything().t_any().t_any().t_ws().then_noun()),
        }
    }
}

impl ExprLinter for ComplainAsNoun {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let idx = match matched_tokens.len() {
            3 => 2,
            _ => return None,
        };

        let mut noun: &[char] = &['c', 'o', 'm', 'p', 'l', 'a', 'i', 'n', 't', 's'];

        if matched_tokens[idx]
            .get_ch(source)
            .last()
            .map(|c| c.to_ascii_lowercase())
            != Some('s')
        {
            noun = &noun[..noun.len() - 1];
        }

        Some(Lint {
            span: matched_tokens[idx].span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case(
                noun.to_vec(),
                matched_tokens[idx].get_ch(source),
            )],
            message: "The noun form is `complaint`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the use of `complain` as a noun."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ComplainAsNoun;

    #[test]
    fn got_a_complain() {
        assert_suggestion_result(
            "Got a complain abount 'direct reference to field' even using the method",
            ComplainAsNoun::default(),
            "Got a complaint abount 'direct reference to field' even using the method",
        );
    }

    #[test]
    fn lodge_a_complain_and_his_complain() {
        assert_suggestion_result(
            "To solve most of these things, requires the one to lodge a complain as well as follow up on his complain to ensure it has been resolved.",
            ComplainAsNoun::default(),
            "To solve most of these things, requires the one to lodge a complaint as well as follow up on his complaint to ensure it has been resolved.",
        );
    }

    #[test]
    fn the_complains() {
        assert_suggestion_result(
            "This platform provides a all in one solution for the complaining and solving of the complains through public forums.",
            ComplainAsNoun::default(),
            "This platform provides a all in one solution for the complaining and solving of the complaints through public forums.",
        );
    }

    #[test]
    fn my_complain() {
        assert_suggestion_result(
            "My complain about: esp-idf/examples/peripherals/lcd/i2c_oled",
            ComplainAsNoun::default(),
            "My complaint about: esp-idf/examples/peripherals/lcd/i2c_oled",
        );
    }

    #[test]
    fn your_complain() {
        assert_suggestion_result(
            "In a customer, you just have to submit your complain.",
            ComplainAsNoun::default(),
            "In a customer, you just have to submit your complaint.",
        );
    }

    #[test]
    fn raise_a_complain() {
        assert_suggestion_result(
            "I raise a complain about the documentation.",
            ComplainAsNoun::default(),
            "I raise a complaint about the documentation.",
        );
    }

    #[test]
    fn many_complains() {
        assert_suggestion_result(
            "Many complains were raised about the documentation.",
            ComplainAsNoun::default(),
            "Many complaints were raised about the documentation.",
        );
    }

    // Avoid false positives

    #[test]
    fn dont_flag_the_complain_flag() {
        assert_no_lints(
            "The idea is to only have the complain flag defined in the manifest",
            ComplainAsNoun::default(),
        );
    }
}
