use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

const RARE_BEFORE_ROGUE: &[&str] = &["baton", "cheek", "khmer", "lip", "moulin"];
const RARE_BEFORE_ROUGE: &[&str] = &["go", "goes", "going", "gone", "went"];
const RARE_AFTER_ROGUE: &[&str] = &["lipstick"];
const RARE_AFTER_ROUGE: &[&str] = &[
    "like", "likes", "nation", "nations", "one", "squadron", "trader", "traders", "wave", "waves",
    "who",
];
pub struct RogueRouge {
    expr: FirstMatchOf,
}

impl Default for RogueRouge {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([
                Box::new(
                    SequenceExpr::word_set(RARE_BEFORE_ROGUE)
                        .t_ws()
                        .then_word_seq(&["rogue"]),
                ) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::word_set(RARE_BEFORE_ROUGE)
                        .t_ws()
                        .t_aco("rouge"),
                ),
                Box::new(
                    SequenceExpr::word_seq(&["rogue"])
                        .t_ws()
                        .t_set(RARE_AFTER_ROGUE),
                ),
                Box::new(SequenceExpr::aco("rouge").t_ws().t_set(RARE_AFTER_ROUGE)),
                Box::new(
                    SequenceExpr::aco("rouge")
                        .then_hyphen()
                        .t_set(&["like", "likes"]),
                ),
            ]),
        }
    }
}

impl ExprLinter for RogueRouge {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = find_the_only_token_matching(matched_tokens, source, |t, c| {
            t.get_ch(c)
                .eq_any_ignore_ascii_case_str(&["rogue", "rouge"])
        })?
        .span;

        let mut the_word = span.get_content(source).to_vec();

        the_word.swap(2, 3);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(the_word)],
            message:
                "Are you confusing `rouge` (color/makeup) with `rogue` (untrustworthy person)?"
                    .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects mixing up `rogue` and `rouge`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::RogueRouge;

    #[test]
    fn fix_go_rouge() {
        assert_suggestion_result(
            "Claude can wake up on an idle session easily, go rouge, and blow your cloud credits.",
            RogueRouge::default(),
            "Claude can wake up on an idle session easily, go rogue, and blow your cloud credits.",
        );
    }

    #[test]
    fn fix_goes_rouge() {
        assert_suggestion_result(
            "A \"not considered mature\" incubation stalls out/stops progressing or goes rouge (no longer working with their upstream advisor)?",
            RogueRouge::default(),
            "A \"not considered mature\" incubation stalls out/stops progressing or goes rogue (no longer working with their upstream advisor)?",
        );
    }

    #[test]
    fn fix_going_rouge() {
        assert_suggestion_result(
            "port power negotiation and other devices going rouge on the network throwing me off the trail",
            RogueRouge::default(),
            "port power negotiation and other devices going rogue on the network throwing me off the trail",
        );
    }

    #[test]
    fn fix_gone_rouge() {
        assert_suggestion_result(
            "and now the its long term partner has gone rouge",
            RogueRouge::default(),
            "and now the its long term partner has gone rogue",
        );
    }

    #[test]
    fn fix_baton_rogue() {
        assert_suggestion_result(
            "The programs know you dont care about the bronx or baton rogue, you just want a spot",
            RogueRouge::default(),
            "The programs know you dont care about the bronx or baton rouge, you just want a spot",
        );
    }

    #[test]
    fn fix_khmer_rogue() {
        assert_suggestion_result(
            "After the Khmer Rogue took power they became very aggressive towards Vietnam",
            RogueRouge::default(),
            "After the Khmer Rouge took power they became very aggressive towards Vietnam",
        );
    }

    #[test]
    fn fix_rouge_who() {
        assert_suggestion_result(
            "What backstory could I use for a rouge who isn't a orphan?",
            RogueRouge::default(),
            "What backstory could I use for a rogue who isn't a orphan?",
        );
    }

    #[test]
    fn fix_rouge_squadron() {
        assert_suggestion_result(
            "Hoth from Star Wars Rouge Squadron.",
            RogueRouge::default(),
            "Hoth from Star Wars Rogue Squadron.",
        );
    }

    #[test]
    fn fix_rouge_nation() {
        assert_suggestion_result(
            "In \"Mission Impossible 5: Rouge Nation\" (2015) Benji is seen playing Halo 5.",
            RogueRouge::default(),
            "In \"Mission Impossible 5: Rogue Nation\" (2015) Benji is seen playing Halo 5.",
        );
    }

    #[test]
    fn fix_rouge_nations() {
        assert_suggestion_result(
            "what prevents rouge nations from crediting their banking system with fake USD?",
            RogueRouge::default(),
            "what prevents rogue nations from crediting their banking system with fake USD?",
        );
    }

    #[test]
    fn fix_lip_rogue() {
        assert_suggestion_result(
            "The lip rogue and nail henna as well complement the look.",
            RogueRouge::default(),
            "The lip rouge and nail henna as well complement the look.",
        );
    }

    #[test]
    fn fix_rouge_trader() {
        assert_suggestion_result(
            "I'm one upgrade away from unlocking the rouge trader is it worth doing and is it beneficial at a low level.",
            RogueRouge::default(),
            "I'm one upgrade away from unlocking the rogue trader is it worth doing and is it beneficial at a low level.",
        );
    }

    #[test]
    fn fix_rogue_lipstick() {
        assert_suggestion_result(
            "Love Dior's Rogue Lipstick-- Hate the Case!",
            RogueRouge::default(),
            "Love Dior's Rouge Lipstick-- Hate the Case!",
        );
    }

    #[test]
    fn fix_rouge_like() {
        assert_suggestion_result(
            "Do you guys think any Rouge-like mechanics would be cool???",
            RogueRouge::default(),
            "Do you guys think any Rogue-like mechanics would be cool???",
        );
    }

    #[test]
    fn fix_rouge_likes() {
        assert_suggestion_result(
            "As someone who dissent really play rouge likes, do you think I'll like risk of rain 2?",
            RogueRouge::default(),
            "As someone who dissent really play rogue likes, do you think I'll like risk of rain 2?",
        );
    }

    #[test]
    fn fix_rouge_traders() {
        assert_suggestion_result(
            "How do Pirates or Rouge Traders Power Their Gellar Fields?",
            RogueRouge::default(),
            "How do Pirates or Rogue Traders Power Their Gellar Fields?",
        );
    }
}
