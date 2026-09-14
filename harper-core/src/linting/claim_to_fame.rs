use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

const TO_MAKE: &[&str] = &["make", "made", "makes", "making"];

pub struct ClaimToFame {
    expr: SequenceExpr,
}

impl Default for ClaimToFame {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["claim", "claims"])
                .t_ws()
                .then_word_seq(&["for", "fame"]),
        }
    }
}

impl ExprLinter for ClaimToFame {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if let Some((before, _)) = ctx
            && let [before @ .., word, ws] = before
            && ws.kind.is_whitespace()
        {
            if word.get_ch(src).eq_any_ignore_ascii_case_str(TO_MAKE) {
                return None;
            } else if (word.kind.is_determiner() || word.kind.is_adjective())
                && let [.., word, ws] = before
                && ws.kind.is_whitespace()
                && word.kind.is_word()
                && word.get_ch(src).eq_any_ignore_ascii_case_str(TO_MAKE)
            {
                return None;
            }
        }

        if let Some((_, after)) = ctx
            && let [ws1, word1, ws2, word2, ..] = after
            && ws1.kind.is_whitespace()
            && word1.kind.is_word()
            && ws2.kind.is_whitespace()
            && word2.kind.is_word()
            && word1.get_ch(src).eq_str("and")
            && (!word2.kind.is_conjunction() && word2.kind.is_noun())
        {
            return None;
        }

        let span = toks.get(2)?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Eggcorn,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "to",
                span.get_content(src),
            )],
            message: "The correct idiom is `claim to fame`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `claim for fame` to the idiom `claim to fame`."
    }
}

#[cfg(test)]
mod tests {
    use super::ClaimToFame;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn correct_claim_for_fame() {
        assert_suggestion_result(
            "Although it is nice to go archive diving, the main claim for fame is of course the derivation of modern technologies",
            ClaimToFame::default(),
            "Although it is nice to go archive diving, the main claim to fame is of course the derivation of modern technologies",
        )
    }

    #[test]
    fn correct_claims_for_fame() {
        assert_suggestion_result(
            "we note and dismiss offhand some of the bulimic claims for fame that have been attached to Kuhn's classic",
            ClaimToFame::default(),
            "we note and dismiss offhand some of the bulimic claims to fame that have been attached to Kuhn's classic",
        )
    }

    #[test]
    fn dont_made_claims_for_reasons_including_fame() {
        assert_no_lints(
            "His attorney says that he made those claims for fame and money.",
            ClaimToFame::default(),
        );
        assert_no_lints(
            "denied the allegations, saying Davis has acknowledged making claims for fame and notoriety",
            ClaimToFame::default(),
        );
    }

    #[test]
    fn fix_claim_for_fame_and_pron() {
        assert_suggestion_result(
            "My one claim for fame and I blew it.",
            ClaimToFame::default(),
            "My one claim to fame and I blew it.",
        )
    }

    #[test]
    fn dont_flag_make_grand_claims_for_fame() {
        assert_no_lints(
            "There have always been people throughout history who are innovative and make grand claims for fame.",
            ClaimToFame::default(),
        );
    }

    #[test]
    fn fix_claim_for_fame_and_so() {
        assert_suggestion_result(
            "Alan Holt has placed his claim for fame and so I've corrected this lovely image",
            ClaimToFame::default(),
            "Alan Holt has placed his claim to fame and so I've corrected this lovely image",
        );
    }

    #[test]
    fn cant_flag_ambiguous_and_verb_is_also_adj() {
        assert_suggestion_result(
            "VRAAQ and level criteria reinforced the students' claim for fame and enhanced the establishment of trust",
            ClaimToFame::default(),
            "VRAAQ and level criteria reinforced the students' claim to fame and enhanced the establishment of trust",
        );
    }
}
