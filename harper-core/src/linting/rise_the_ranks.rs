use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct RiseTheRanks {
    expr: SequenceExpr,
}

impl Default for RiseTheRanks {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&[
                // legit forms of the legit verb "rise"
                "rise", "risen", "rising", "rose",
                // wrong forms of the legit verb "rise"
                "rised", // forms of wrong verbs used instead of forms of "rise"
                        // "arise", "arised", "arisen", "arising", "arose", "raise", "raised", "raises",
                        // "raising",
            ])
            .t_ws()
            // .then_optional(
            //     SequenceExpr::optional(SequenceExpr::aco("up").t_ws())
            //         // modern "through", traditional "from"
            //         // also seen: "in", "up"
            //         .then_preposition()
            //         .t_ws(),
            // )
            .t_aco("the")
            .t_ws()
            // .t_set(&["ranks", "rank"]),
            .t_aco("ranks"),
        }
    }
}

impl ExprLinter for RiseTheRanks {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects the nonstandard phrase `rise the ranks` to the standard `rise through the ranks` or `rise from the ranks`"
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🧵 {}", format_lint_match(toks, ctx, src));

        Some(Lint {
            span: toks[0].span,
            lint_kind: LintKind::Usage,
            suggestions: vec![
                Suggestion::InsertAfter(vec![' ', 't', 'h', 'r', 'o', 'u', 'g', 'h']),
                Suggestion::InsertAfter(vec![' ', 'f', 'r', 'o', 'm']),
            ],
            message: "Use either the modern standard 'rise through the ranks' or the traditional 'rise from the ranks'. 'Rise the ranks' is nonstandard.".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RiseTheRanks;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_rise_the_ranks() {
        assert_suggestion_result(
            "Rise the ranks from Division 8 to the Champion!",
            RiseTheRanks::default(),
            "Rise through the ranks from Division 8 to the Champion!",
        );
    }

    #[test]
    fn fix_rised_the_ranks() {
        assert_suggestion_result(
            "while there I've rised the ranks from L1 Tech > L2 Tech > Team lead and now Manager",
            RiseTheRanks::default(),
            "while there I've rised through the ranks from L1 Tech > L2 Tech > Team lead and now Manager",
        );
    }

    #[test]
    fn fix_risen_the_ranks() {
        assert_suggestion_result(
            "AngularJS has risen the ranks of popularity a lot over the last couple of years.",
            RiseTheRanks::default(),
            "AngularJS has risen through the ranks of popularity a lot over the last couple of years.",
        );
    }

    #[test]
    fn fix_rising_the_ranks() {
        assert_suggestion_result(
            "Rust is new coding language rising the ranks, aiming at safe concurrency, data safe programming at zero abstraction cost.",
            RiseTheRanks::default(),
            "Rust is new coding language rising through the ranks, aiming at safe concurrency, data safe programming at zero abstraction cost.",
        );
    }

    #[test]
    fn fix_rose_the_ranks() {
        assert_suggestion_result(
            "we follow these women as they quickly rose the ranks of NASA alongside many of history's greatest minds",
            RiseTheRanks::default(),
            "we follow these women as they quickly rose through the ranks of NASA alongside many of history's greatest minds",
        );
    }

    #[test]
    fn dont_flag_rise_through_the_ranks() {
        assert_no_lints(
            "I hadn't noticed any particular suspicious behavior during my slow but steady rise through the ranks.",
            RiseTheRanks::default(),
        );
    }

    #[test]
    fn dont_flag_rises_through_the_ranks() {
        assert_no_lints(
            "Vito Scaletta rises through the ranks of the mafia, becoming a powerful don.",
            RiseTheRanks::default(),
        );
    }

    #[test]
    fn dont_flag_rising_from_the_ranks() {
        assert_no_lints(
            "Rising from the ranks of a sailor, he seems to have embodied the American Dream among a navy full of elites.",
            RiseTheRanks::default(),
        );
    }

    #[test]
    fn dont_flag_rising_through_the_ranks() {
        assert_no_lints(
            "From a tranquil town, he discovered his passion in online gaming tournaments, rising through the ranks with unwavering dedication.",
            RiseTheRanks::default(),
        );
    }

    // #[test]
    // fn fix_raise_in_the_ranks() {
    //     assert_suggestion_result(
    //         "I am trying to raise in the ranks to be more helpful so I would appreciate comments, and upvotes if this was helpful.",
    //         RiseTheRanks::default(),
    //         "I am trying to rise through the ranks to be more helpful so I would appreciate comments, and upvotes if this was helpful.",
    //     );
    // }

    // #[test]
    // fn fix_raise_up_in_the_ranks() {
    //     assert_suggestion_result(
    //         "And the rejects that make it back successfully raise up in the ranks.",
    //         RiseTheRanks::default(),
    //         "And the rejects that make it back successfully rise through the ranks.",
    //     );
    // }

    // #[test]
    // fn fix_raise_up_through_the_ranks() {
    //     assert_suggestion_result(
    //         "characters who use their intelligence, charms ETC. to gain power and raise up through the ranks",
    //         RiseTheRanks::default(),
    //         "characters who use their intelligence, charms ETC. to gain power and rise through the ranks",
    //     );
    // }

    // #[test]
    // fn fix_raised_through_the_rank() {
    //     assert_suggestion_result(
    //         "I raised through the rank to be responsible of all the main offensive wars and managed to help my ruler build a respectable kingdom",
    //         RiseTheRanks::default(),
    //         "I rose through the ranks to be responsible of all the main offensive wars and managed to help my ruler build a respectable kingdom",
    //     );
    // }

    // #[test]
    // fn fix_raises_through_the_ranks() {
    //     assert_suggestion_result(
    //         "He raises through the ranks pretty quickly. Ensign. Lieutenant. Senior Lieutenant.",
    //         RiseTheRanks::default(),
    //         "He rises through the ranks pretty quickly. Ensign. Lieutenant. Senior Lieutenant.",
    //     );
    // }

    // #[test]
    // fn fix_raising_through_the_ranks() {
    //     assert_suggestion_result(
    //         "The signals were reinforced by the people whom I saw raising through the ranks while those around them burnt out.",
    //         RiseTheRanks::default(),
    //         "The signals were reinforced by the people whom I saw rising through the ranks while those around them burnt out.",
    //     );
    // }

    // #[test]
    // fn fix_raising_up_in_the_ranks() {
    //     assert_suggestion_result(
    //         "I would have rather stayed as a blacksmith son raising up in the ranks.",
    //         RiseTheRanks::default(),
    //         "I would have rather stayed as a blacksmith son rising through the ranks.",
    //     );
    // }

    // #[test]
    // fn fix_raising_up_through_the_ranks() {
    //     assert_suggestion_result(
    //         "then it's through completing the foraging club missions and raising up through the ranks",
    //         RiseTheRanks::default(),
    //         "then it's through completing the foraging club missions and rising through the ranks",
    //     );
    // }

    // #[test]
    // fn fix_rise_up_in_the_rank() {
    //     assert_suggestion_result(
    //         "Supposing I am a soldier in the Qin military,what should I do to rise up in the rank?",
    //         RiseTheRanks::default(),
    //         "Supposing I am a soldier in the Qin military,what should I do to rise through the ranks?",
    //     );
    // }

    // #[test]
    // fn fix_rise_up_in_the_ranks() {
    //     assert_suggestion_result(
    //         "It requires weeding out those that would stop or rise up in the ranks against communist control.",
    //         RiseTheRanks::default(),
    //         "It requires weeding out those that would stop or rise through the ranks against communist control.",
    //     );
    // }

    // #[test]
    // fn fix_rise_up_the_ranks() {
    //     assert_suggestion_result(
    //         "It is far easier to rise up the ranks quickly in a new space than in an old, crowded space.",
    //         RiseTheRanks::default(),
    //         "It is far easier to rise through the ranks quickly in a new space than in an old, crowded space.",
    //     );
    // }

    // #[test]
    // fn fix_rise_up_through_the_rank() {
    //     assert_suggestion_result(
    //         "Gau really rise up through the rank after they finally settle him on wind/lightning lol.",
    //         RiseTheRanks::default(),
    //         "Gau really rise through the ranks after they finally settle him on wind/lightning lol.",
    //     );
    // }

    // #[test]
    // fn fix_rise_up_through_the_ranks() {
    //     assert_suggestion_result(
    //         "There's no obligation to do this, but it does help users to rise up through the ranks, or sometimes, simply to feel helpful",
    //         RiseTheRanks::default(),
    //         "There's no obligation to do this, but it does help users to rise through the ranks, or sometimes, simply to feel helpful",
    //     );
    // }

    // #[test]
    // fn fix_rised_through_the_ranks() {
    //     assert_suggestion_result(
    //         "The eldest brother quickly rised through the ranks of the legion.",
    //         RiseTheRanks::default(),
    //         "The eldest brother quickly rose through the ranks of the legion.",
    //     );
    // }

    // #[test]
    // fn fix_rised_up_in_the_ranks() {
    //     assert_suggestion_result(
    //         "infiltrated the council and rised up in the ranks but couldn't pretend to be Mystogan for less than a week",
    //         RiseTheRanks::default(),
    //         "infiltrated the council and rose through the ranks but couldn't pretend to be Mystogan for less than a week",
    //     );
    // }

    // #[test]
    // fn fix_risen_in_the_ranks() {
    //     assert_suggestion_result(
    //         "Python has risen in the ranks, surpassing C# this year, much like it surpassed PHP last year.",
    //         RiseTheRanks::default(),
    //         "Python has risen through the ranks, surpassing C# this year, much like it surpassed PHP last year.",
    //     );
    // }

    // #[test]
    // fn fix_rising_up_through_the_ranks() {
    //     assert_suggestion_result(
    //         "It's hard to even imagine rising up through the ranks, grinding out the decades, and then retiring - with a penchant - all at a single company.",
    //         RiseTheRanks::default(),
    //         "It's hard to even imagine rising through the ranks, grinding out the decades, and then retiring - with a penchant - all at a single company.",
    //     );
    // }
}
