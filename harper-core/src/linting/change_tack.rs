use crate::{
    Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
};

pub struct ChangeTack {
    expr: Box<dyn Expr>,
}

impl Default for ChangeTack {
    fn default() -> Self {
        let verb_forms = &["change", "changes", "changing", "changed"];
        let noun_forms = &verb_forms[..3];
        let eggcorns = &["tact", "tacks", "tacts"];

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(
                    SequenceExpr::longest_of(vec![
                        Box::new(SequenceExpr::word_set(verb_forms).then_optional(
                            SequenceExpr::default().t_ws().then_any_of(vec![
                                Box::new(SequenceExpr::default().then_possessive_determiner()),
                                Box::new(Word::new("it's")),
                            ]),
                        )),
                        Box::new(SequenceExpr::word_set(noun_forms).t_ws().t_aco("of")),
                    ])
                    .t_ws()
                    .then_word_set(eggcorns),
                ),
                Box::new(SequenceExpr::aco("different").t_ws().t_aco("tact")),
            ])),
        }
    }
}

impl ExprLinter for ChangeTack {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tact_tok = toks.last()?;
        let tact_span = tact_tok.span;
        let tact_chars = tact_span.get_content(src);

        Some(Lint {
            span: tact_span,
            lint_kind: LintKind::Eggcorn,
            suggestions: vec![Suggestion::replace_with_match_case(
                ['t', 'a', 'c', 'k'].to_vec(),
                tact_chars,
            )],
            message: "A change in direction or approach is a change of `tack`. Not `tact` (or `tacks` or `tacts`).".to_owned(),
            priority: 32,
        })
    }

    fn description(&self) -> &'static str {
        "Locates errors in the idioms `to change tack` and `change of tack` to convey the correct meaning of altering one's course or strategy."
    }
}

#[cfg(test)]
mod tests {
    use super::ChangeTack;
    use crate::linting::tests::assert_suggestion_result;

    // Verbs: change tack

    #[test]
    fn change_tact_atomic() {
        assert_suggestion_result("change tact", ChangeTack::default(), "change tack");
    }

    #[test]
    fn changed_tacks_atomic() {
        assert_suggestion_result("changed tacks", ChangeTack::default(), "changed tack");
    }

    #[test]
    fn changes_tacts_atomic() {
        assert_suggestion_result("changes tacts", ChangeTack::default(), "changes tack");
    }

    #[test]
    fn changing_tact_atomic() {
        assert_suggestion_result("changing tact", ChangeTack::default(), "changing tack");
    }

    // Nouns: change of tack

    #[test]
    fn change_of_tacks_atomic() {
        assert_suggestion_result("change of tacks", ChangeTack::default(), "change of tack");
    }

    #[test]
    fn change_of_tact_real_world() {
        assert_suggestion_result(
            "Change of tact : come give your concerns - Death Knight",
            ChangeTack::default(),
            "Change of tack : come give your concerns - Death Knight",
        );
    }

    #[test]
    fn change_of_tacts_real_world() {
        assert_suggestion_result(
            "2013.08.15 - A Change of Tacts | Hero MUX Wiki | Fandom",
            ChangeTack::default(),
            "2013.08.15 - A Change of Tack | Hero MUX Wiki | Fandom",
        );
    }

    #[test]
    fn changing_of_tacks_real_world() {
        assert_suggestion_result(
            "Duffy's changing of tacks hidden in her poetry collection ...",
            ChangeTack::default(),
            "Duffy's changing of tack hidden in her poetry collection ...",
        );
    }

    #[test]
    fn changes_of_tact_real_world() {
        assert_suggestion_result(
            "While the notes and the changes of tact started to ...",
            ChangeTack::default(),
            "While the notes and the changes of tack started to ...",
        );
    }

    // With possessive determiners

    #[test]
    fn changed_my_tact() {
        assert_suggestion_result(
            "I have changed my tact this year, and have two second dates in the next week.",
            ChangeTack::default(),
            "I have changed my tack this year, and have two second dates in the next week.",
        );
    }

    #[test]
    fn changed_our_tact() {
        assert_suggestion_result(
            "That being said we have changed our tact slightly and gone for making all UI elements lazy.",
            ChangeTack::default(),
            "That being said we have changed our tack slightly and gone for making all UI elements lazy.",
        );
    }

    #[test]
    fn change_your_tact() {
        assert_suggestion_result(
            "If you've ever heard the phrase “you've got to change your tact”, this is probably where it comes from.",
            ChangeTack::default(),
            "If you've ever heard the phrase “you've got to change your tack”, this is probably where it comes from.",
        );
    }

    #[test]
    fn change_his_tact() {
        assert_suggestion_result(
            "Why did Sephiroth change his tact with Cloud midway through the game?",
            ChangeTack::default(),
            "Why did Sephiroth change his tack with Cloud midway through the game?",
        );
    }

    #[test]
    fn changed_her_tact() {
        assert_suggestion_result(
            "Only the last commitment ceremony I think she changed her tact and went on about George needing to be the real George.",
            ChangeTack::default(),
            "Only the last commitment ceremony I think she changed her tack and went on about George needing to be the real George.",
        );
    }

    #[test]
    fn change_its_tact() {
        assert_suggestion_result(
            "The show seems to change its tact depending on the episode.",
            ChangeTack::default(),
            "The show seems to change its tack depending on the episode.",
        );
    }

    #[test]
    fn changing_its_tact_apostrophe() {
        assert_suggestion_result(
            "FYI, USL is changing it's tact internally about MLS II teams.",
            ChangeTack::default(),
            "FYI, USL is changing it's tack internally about MLS II teams.",
        );
    }

    #[test]
    fn changes_their_tact() {
        assert_suggestion_result(
            "As we become inoculated to attention grifts, the grifter changes their tact.",
            ChangeTack::default(),
            "As we become inoculated to attention grifts, the grifter changes their tack.",
        );
    }

    #[test]
    fn different_tact() {
        assert_suggestion_result(
            "So, I recently took a different tact: I put all my models etc. in a single folder.",
            ChangeTack::default(),
            "So, I recently took a different tack: I put all my models etc. in a single folder.",
        );
    }
}
