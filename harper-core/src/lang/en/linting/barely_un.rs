use crate::{
    CharStringExt, Lint, Lrc, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    spell::Dictionary,
};

pub struct BarelyUn<D: Dictionary + 'static> {
    expr: All,
    dict: Lrc<D>,
}

impl<D: Dictionary + 'static> BarelyUn<D> {
    pub fn new(dict: D) -> Self {
        let dict = Lrc::new(dict);
        let dict_clone = Lrc::clone(&dict);
        Self {
            expr: SequenceExpr::aco("barely")
                .t_ws()
                .then_positive_adjective()
                .and(
                    SequenceExpr::anything()
                        .t_any()
                        .then(move |t: &Token, s: &[char]| {
                            t.get_ch(s)
                                .strip_prefix_ignore_ascii_case_chars(&['u', 'n'])
                                .and_then(|stem| dict_clone.get_word_metadata(stem))
                                .map(|md| md.is_positive_adjective())
                                .unwrap_or(false)
                        }),
                ),
            dict,
        }
    }
}

impl<D: Dictionary> ExprLinter for BarelyUn<D> {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let last = toks.last()?;

        let correction = last
            .get_ch(src)
            .strip_prefix_ignore_ascii_case_chars(&['u', 'n'])?;

        let span = last.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous, // as per `Oxymorons`
            suggestions: vec![Suggestion::replace_with_match_case(
                correction.to_vec(),
                span.get_content(src),
            )],
            message: "Using `barely` with a negative adjective is a kind of double negative"
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags using `barely` with a negative adjective starting with `un-` (`barely unusable`, etc.), which is a kind of double negative."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::BarelyUn;
    use crate::spell::FstDictionary;

    #[test]
    fn fix_unafraid() {
        assert_suggestion_result(
            "drone footage capturing me barely unafraid of rattlesnakes",
            BarelyUn::new(FstDictionary::curated()),
            "drone footage capturing me barely afraid of rattlesnakes",
        );
    }

    #[test]
    fn fix_unaware() {
        assert_suggestion_result(
            "Many Christians claim to be followers of Jesus Christ but wander in the darkness barely unaware of the dangerous ramifications",
            BarelyUn::new(FstDictionary::curated()),
            "Many Christians claim to be followers of Jesus Christ but wander in the darkness barely aware of the dangerous ramifications",
        );
    }

    #[test]
    fn fix_unbelievable() {
        assert_suggestion_result(
            "with the locals then taking a barely unbelievable lead when Romarinho netted five minutes before half-time",
            BarelyUn::new(FstDictionary::curated()),
            "with the locals then taking a barely believable lead when Romarinho netted five minutes before half-time",
        );
    }

    #[test]
    fn fix_uncommon() {
        assert_suggestion_result(
            "I'm not seeing how this is an artifact. It's barely uncommon.",
            BarelyUn::new(FstDictionary::curated()),
            "I'm not seeing how this is an artifact. It's barely common.",
        );
    }

    #[test]
    fn fix_unconscious() {
        assert_suggestion_result(
            "That night, the only dreams that troubled his barely unconscious sleeping mind, were of The Ravens Gate Bridge, all dark and foreboding.",
            BarelyUn::new(FstDictionary::curated()),
            "That night, the only dreams that troubled his barely conscious sleeping mind, were of The Ravens Gate Bridge, all dark and foreboding.",
        );
    }

    #[test]
    fn fix_unconscious_capitalized() {
        assert_suggestion_result(
            "I remember sessions of my youth when I was dropped barely unconscious and sat out the rest of the battle",
            BarelyUn::new(FstDictionary::curated()),
            "I remember sessions of my youth when I was dropped barely conscious and sat out the rest of the battle",
        );
    }

    #[test]
    fn fix_unhealthy() {
        assert_suggestion_result(
            "This could be explained by the fact that the administration of a large number of cells in a barely unhealthy animal might trigger an immune response",
            BarelyUn::new(FstDictionary::curated()),
            "This could be explained by the fact that the administration of a large number of cells in a barely healthy animal might trigger an immune response",
        );
    }

    fn fix_unhealthy_range() {
        assert_suggestion_result(
            "a score of 50 and below is very unhealthy,50 to 59 is unhealthy, to 69 is barely unhealthy",
            BarelyUn::new(FstDictionary::curated()),
            "a score of 50 and below is very healthy,50 to 59 is healthy, to 69 is barely healthy",
        );
    }

    #[test]
    fn fix_unknown() {
        assert_suggestion_result(
            "the electrocatalytic activity in CNFs is barely unknown",
            BarelyUn::new(FstDictionary::curated()),
            "the electrocatalytic activity in CNFs is barely known",
        );
    }

    #[test]
    fn fix_unlikable() {
        assert_suggestion_result(
            "It's confusing, weird with barely unlikable characters.",
            BarelyUn::new(FstDictionary::curated()),
            "It's confusing, weird with barely likable characters.",
        );
    }

    #[test]
    fn fix_unlivable() {
        assert_suggestion_result(
            "So yes the hotel isnt 5* but its barely unlivable!",
            BarelyUn::new(FstDictionary::curated()),
            "So yes the hotel isnt 5* but its barely livable!",
        );
    }
}
