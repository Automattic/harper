use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct RackOnesBrain {
    expr: SequenceExpr,
}

impl Default for RackOnesBrain {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set([
                "wrack", "wracked", "wracking", "wracks", "wreck", "wrecked", "wrecking", "wrecks",
            ])
            .t_ws()
            .then_possessive_determiner()
            .t_ws()
            .then_optional(SequenceExpr::default().then_adjective().t_ws())
            .t_set(["brain", "brains"]),
        }
    }
}

impl ExprLinter for RackOnesBrain {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (verbtok, _braintok) = (toks.first()?, toks.last()?);

        let span = verbtok.span;

        let mut correction: Vec<char> = verbtok.get_ch(src).to_vec();
        let w_was_upper = correction.first()?.is_uppercase();
        correction.remove(0);
        let r_was_upper = correction.first()?.is_uppercase();
        let new_r = if w_was_upper { 'R' } else { 'r' };
        if let Some(first) = correction.first_mut() {
            *first = new_r;
        }
        if let Some(vowel) = correction.get_mut(1) {
            *vowel = if r_was_upper { 'A' } else { 'a' };
        }

        // TODO: ensuare singular 'brain' not 'brains'?

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::ReplaceWith(correction)],
            message: "Did you mean to use the word `rack`? While historically debated, standard modern usage dictates `rack one's brain` when describing intense mental effort.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Checks for improper uses of `wrack` or `wreck` instead of `rack` in the idiom `rack one's brain`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::RackOnesBrain;

    // Basic logic tests

    #[test]
    fn wracking_titlecase() {
        assert_suggestion_result(
            "Wracking my brain",
            RackOnesBrain::default(),
            "Racking my brain",
        );
    }

    #[test]
    fn wrecking_titlecase() {
        assert_suggestion_result(
            "Wrecking my brain",
            RackOnesBrain::default(),
            "Racking my brain",
        );
    }

    #[test]
    fn wracked_allcaps() {
        assert_suggestion_result(
            "WRACKED MY BRAIN",
            RackOnesBrain::default(),
            "RACKED MY BRAIN",
        );
    }

    #[test]
    fn wrecked_allcaps() {
        assert_suggestion_result(
            "WRECKED MY BRAIN",
            RackOnesBrain::default(),
            "RACKED MY BRAIN",
        );
    }

    #[test]
    fn wrack_lower() {
        assert_suggestion_result("wrack my brain", RackOnesBrain::default(), "rack my brain");
    }

    #[test]
    fn wreck_lower() {
        assert_suggestion_result("wreck my brain", RackOnesBrain::default(), "rack my brain");
    }

    // Real-world tests

    // Definite wrong cases that should be caught

    #[test]
    fn wracking_my_adj_sg() {
        assert_suggestion_result(
            "I've been wracking my tiny brain re: Parcelables, ContentValues, and inserting into SQLite",
            RackOnesBrain::default(),
            "I've been racking my tiny brain re: Parcelables, ContentValues, and inserting into SQLite",
        );
    }

    #[test]
    fn wracking_my_pl() {
        assert_suggestion_result(
            "I write code and enjoy wracking my brains on research problems.",
            RackOnesBrain::default(),
            "I write code and enjoy racking my brains on research problems.",
        );
    }

    // Examples that seem literal rather than mistakes

    // studying chemistry wrecked my freaking brain imma go sleep
}
