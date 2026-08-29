use crate::{
    Lint, Token, TokenStringExt,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct Ajar {
    expr: All,
}

impl Default for Ajar {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["door", "doors", "gate", "gates", "mouth", "slightly"])
                .t_ws_h()
                .t_aco("a")
                .t_ws_h()
                .t_aco("jar")
                .but_not(
                    SequenceExpr::anything()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_ws()
                        .t_set(&["containing", "contains", "full", "of"]),
                ),
        }
    }
}

impl ExprLinter for Ajar {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.get(2..=4)?.span()?;

        let a = toks.get(2)?;
        let jar = toks.get(4)?;

        let fix: Vec<char> = a
            .get_ch(src)
            .into_iter()
            .copied()
            .chain(jar.get_ch(src).into_iter().copied())
            .collect();

        let suggestions = vec![Suggestion::ReplaceWith(fix)];

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "Did you mean `ajar` (slightly open)?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `a jar` to `ajar`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Ajar;

    #[test]
    fn fix_door_a_jar_hyphen() {
        assert_suggestion_result(
            "not simply breaking into the house of a man they suspected of a crime but also them leaving his door a-jar after they left",
            Ajar::default(),
            "not simply breaking into the house of a man they suspected of a crime but also them leaving his door ajar after they left",
        );
    }

    #[test]
    fn fix_slightly_a_jar_hyphen() {
        assert_suggestion_result(
            "A door to democratizing investing that's slightly a-jar.",
            Ajar::default(),
            "A door to democratizing investing that's slightly ajar.",
        );
    }

    #[test]
    fn fix_door_a_jar_space() {
        assert_suggestion_result(
            "so it's easier just to read my book indoors with the door a jar so ...",
            Ajar::default(),
            "so it's easier just to read my book indoors with the door ajar so ...",
        );
    }

    #[test]
    fn fix_car_door() {
        assert_suggestion_result(
            "Why is my car calling my door a jar? Is it stupid?",
            Ajar::default(),
            "Why is my car calling my door ajar? Is it stupid?",
        );
    }

    #[test]
    fn fix_mouth() {
        assert_suggestion_result(
            "I dropped my book on the ground mouth a jar when it happened.",
            Ajar::default(),
            "I dropped my book on the ground mouth ajar when it happened.",
        );
    }

    #[test]
    fn fix_rational_doors() {
        assert_suggestion_result(
            "It keeps my rational doors a jar.",
            Ajar::default(),
            "It keeps my rational doors ajar.",
        );
    }

    #[test]
    fn fix_gate() {
        assert_suggestion_result(
            "Leaving the garden gate a jar ...",
            Ajar::default(),
            "Leaving the garden gate ajar ...",
        );
    }
}
