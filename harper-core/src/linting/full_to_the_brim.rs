use crate::{
    CharStringExt, Lint, Token,
    expr::{All, Expr, OwnedExprExt, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

pub struct FullToTheBrim {
    expr: All,
}

impl Default for FullToTheBrim {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&["full", "filled"])
                .t_ws()
                .then_word_seq(&["to", "the", "brim", "of"])
                .but_not(
                    SequenceExpr::default()
                        // This could be cleaner if #3897 is solved
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_any()
                        .t_ws()
                        .t_aco("the"),
                ),
        }
    }
}

impl ExprLinter for FullToTheBrim {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = find_the_only_token_matching(matched_tokens, source, |t, _| {
            t.get_ch(source).eq_str("of")
        })?
        .span;

        Some(Lint {
            span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "with",
                span.get_content(source),
            )],
            message: "The correct preposition with this idiom is 'with', not 'of'".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the wrong preposition in the idiom `full` or `filled to the brim`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::FullToTheBrim;

    // Basic tests

    #[test]
    fn flag_incorrect_idiom_of() {
        assert_suggestion_result(
            "my mug is full to the brim of coffee",
            FullToTheBrim::default(),
            "my mug is full to the brim with coffee",
        );
    }

    #[test]
    fn dont_flag_correct_idiom_with() {
        assert_no_lints(
            "my brain is full to the brim with good ideas",
            FullToTheBrim::default(),
        );
    }

    #[test]
    fn dont_flag_correct_idiom_of_the() {
        assert_no_lints("full to the brim of the cup", FullToTheBrim::default());
    }

    // Real-world tests

    #[test]
    fn fix_vitamin_b() {
        assert_suggestion_result(
            "I feel full to the brim of Vitamin B",
            FullToTheBrim::default(),
            "I feel full to the brim with Vitamin B",
        );
    }

    #[test]
    fn fix_blood_and_guts() {
        assert_suggestion_result(
            "But at the same time it is full to the brim of blood and guts",
            FullToTheBrim::default(),
            "But at the same time it is full to the brim with blood and guts",
        );
    }

    #[test]
    fn fix_bad_back_spirits() {
        assert_suggestion_result(
            "Once the axe was full to the brim of bad back spirits it would be banged on the ...",
            FullToTheBrim::default(),
            "Once the axe was full to the brim with bad back spirits it would be banged on the ...",
        );
    }

    #[test]
    fn fix_glass_full_of_alcohol() {
        assert_suggestion_result(
            "glass full to the brim of alcohol",
            FullToTheBrim::default(),
            "glass full to the brim with alcohol",
        );
    }

    #[test]
    fn fix_heart_full_of_sun_beams() {
        assert_suggestion_result(
            "Heart full to the brim of sun beams. 🥰",
            FullToTheBrim::default(),
            "Heart full to the brim with sun beams. 🥰",
        );
    }

    #[test]
    fn fix_glass_full_of_water() {
        assert_suggestion_result(
            "glass full to the brim of fresh crystal clear transparent water with a frozen sheen on",
            FullToTheBrim::default(),
            "glass full to the brim with fresh crystal clear transparent water with a frozen sheen on",
        );
    }

    #[test]
    fn fix_full_of_new_and_rad_stock() {
        assert_suggestion_result(
            "Full to the brim of new and rad stock!!! Pop past and see for yourself! #treatyoself.",
            FullToTheBrim::default(),
            "Full to the brim with new and rad stock!!! Pop past and see for yourself! #treatyoself.",
        );
    }

    #[test]
    fn fix_startup_filled_with_faang_rejects() {
        assert_suggestion_result(
            "If you join a startup - it is frequently filled full to the brim of FAANG rejects, young people who don't realize what FAANG offers, and people from different ...",
            FullToTheBrim::default(),
            "If you join a startup - it is frequently filled full to the brim with FAANG rejects, young people who don't realize what FAANG offers, and people from different ...",
        );
    }

    #[test]
    fn fix_parsed_json_files_full_of_utf8() {
        assert_suggestion_result(
            "I parsed JSON files full to the brim of UTF-8, but it wasn't a problem.",
            FullToTheBrim::default(),
            "I parsed JSON files full to the brim with UTF-8, but it wasn't a problem.",
        );
    }

    #[test]
    fn fix_put_in_empty_caps_and_bottle_full_of_water() {
        assert_suggestion_result(
            "Put in a few empty caps, and a bottle full to the brim of water also for comparison.",
            FullToTheBrim::default(),
            "Put in a few empty caps, and a bottle full to the brim with water also for comparison.",
        );
    }

    #[test]
    fn fix_ungoliant_full_of_light() {
        assert_suggestion_result(
            "Ungoliant is full to the brim of that light—or would be, if she hadn't kept on growing as she drank, making the concept of “brim” a little hazy!",
            FullToTheBrim::default(),
            "Ungoliant is full to the brim with that light—or would be, if she hadn't kept on growing as she drank, making the concept of “brim” a little hazy!",
        );
    }

    #[test]
    fn fix_open_internet_full_of_bad_actors() {
        assert_suggestion_result(
            "I don't consider this a tacit ad hominem per se, because we must acknowledge the open internet is full to the brim of bad actors and bots and we cannot ...",
            FullToTheBrim::default(),
            "I don't consider this a tacit ad hominem per se, because we must acknowledge the open internet is full to the brim with bad actors and bots and we cannot ...",
        );
    }

    #[test]
    fn fix_internet_full_of_pages() {
        assert_suggestion_result(
            "The Internet is full to the brim of pages explaining it better than I can.",
            FullToTheBrim::default(),
            "The Internet is full to the brim with pages explaining it better than I can.",
        );
    }

    #[test]
    fn fix_poured_hearts_into_javadocs() {
        assert_suggestion_result(
            "we've poured our hearts into making these javadocs full to the brim of examples",
            FullToTheBrim::default(),
            "we've poured our hearts into making these javadocs full to the brim with examples",
        );
    }

    #[test]
    fn fix_spreadsheets_filled_with_unoptimized_formulas() {
        assert_suggestion_result(
            "Imagine 100+ spreadsheets with 1 to 3 sheets on average, each filled to the brim of unoptimized formulas wrote by accountants that have absolutely ZERO ...",
            FullToTheBrim::default(),
            "Imagine 100+ spreadsheets with 1 to 3 sheets on average, each filled to the brim with unoptimized formulas wrote by accountants that have absolutely ZERO ...",
        );
    }

    #[test]
    fn fix_magazines_full_of_intelligent_people() {
        assert_suggestion_result(
            "Magazines, journals, and libraries are full to the brim of intelligent people writing intelligent things.",
            FullToTheBrim::default(),
            "Magazines, journals, and libraries are full to the brim with intelligent people writing intelligent things.",
        );
    }

    #[test]
    fn fix_continuing_to_use_amazon() {
        assert_suggestion_result(
            "Continuing to use Amazon when you know how full to the brim of scams it is, just seems to me like rewarding them for bad behaviour.",
            FullToTheBrim::default(),
            "Continuing to use Amazon when you know how full to the brim with scams it is, just seems to me like rewarding them for bad behaviour.",
        );
    }

    #[test]
    fn fix_if_statement_huge_and_full_of_imports() {
        assert_suggestion_result(
            "The if statement was huge and full to the brim of imports that did nothing, and code that just didn't look right.",
            FullToTheBrim::default(),
            "The if statement was huge and full to the brim with imports that did nothing, and code that just didn't look right.",
        );
    }

    #[test]
    fn fix_he_flung_himself_over_to_the_edge() {
        assert_suggestion_result(
            "He flung himself over to the edge of the rock and plunged the goblet into the lake, bringing it up full to the brim of icy water that did not ...",
            FullToTheBrim::default(),
            "He flung himself over to the edge of the rock and plunged the goblet into the lake, bringing it up full to the brim with icy water that did not ...",
        );
    }

    #[test]
    fn fix_oof_this_is_full_to_the_brim_of_bad_c_practices() {
        assert_suggestion_result(
            "Oof, this is full to the brim of bad C practices.",
            FullToTheBrim::default(),
            "Oof, this is full to the brim with bad C practices.",
        );
    }

    #[test]
    fn fix_cpp_syntax_is_full_to_the_brim_of_ugly_corners() {
        assert_suggestion_result(
            "C++'s syntax is full to the brim of such ugly corners, the standard doesn't even bother to specify a binding BNF on the implementations!",
            FullToTheBrim::default(),
            "C++'s syntax is full to the brim with such ugly corners, the standard doesn't even bother to specify a binding BNF on the implementations!",
        );
    }
    #[test]
    fn fix_mysql_is_full_to_the_brim_of_foot_guns() {
        assert_suggestion_result(
            "MySQL is full to the brim of foot-guns",
            FullToTheBrim::default(),
            "MySQL is full to the brim with foot-guns",
        );
    }

    #[test]
    fn fix_c_is_sadly_filled_to_the_brim_of_footguns() {
        assert_suggestion_result(
            "C is sadly filled to the brim of footguns, and C++ inherited them all and added some more of their own.",
            FullToTheBrim::default(),
            "C is sadly filled to the brim with footguns, and C++ inherited them all and added some more of their own.",
        );
    }

    #[test]
    fn fix_the_field_is_more_full_to_the_brim_of_people_presenting_opinions_as_fact() {
        assert_suggestion_result(
            "The field is more full to the brim of people presenting opinions as fact than I would ever have believed.",
            FullToTheBrim::default(),
            "The field is more full to the brim with people presenting opinions as fact than I would ever have believed.",
        );
    }

    // Examples of why making an except for "the" is a good idea

    #[test]
    fn dont_flag_full_to_the_brim_of_the_bowl() {
        assert_no_lints(
            "For a little person to pour a kymyz full to the brim of the bowl, so guests know that hosts do not skimp on kymyz",
            FullToTheBrim::default(),
        );
    }

    #[test]
    fn dont_flag_filled_to_the_brim_of_the_screen() {
        assert_no_lints(
            "The tv is filled to the brim of the screen with news of the world, all bad tidings for the erstwhile faint-of-heart.",
            FullToTheBrim::default(),
        );
    }

    #[test]
    fn dont_flag_dishes_full_to_the_brim_of_the_sink() {
        assert_no_lints(
            "The kitchen is never clean, an open bin, dishes full to the brim of the sink",
            FullToTheBrim::default(),
        );
    }

    // Examples of why making exceptions for all determiners and not just "the" isn't a great idea

    #[test]
    fn fix_full_to_the_brim_of_our() {
        assert_suggestion_result(
            "I decided to make a ribbon world full to the brim of our true enemy.",
            FullToTheBrim::default(),
            "I decided to make a ribbon world full to the brim with our true enemy.",
        );
    }

    #[test]
    fn fix_full_to_the_brim_of_this() {
        assert_suggestion_result(
            "When you're full to the brim of this goodness, you can sop up that extra sauce with rice, cooked crispy on the same skillet.",
            FullToTheBrim::default(),
            "When you're full to the brim with this goodness, you can sop up that extra sauce with rice, cooked crispy on the same skillet.",
        );
    }

    #[test]
    fn fix_filled_to_the_brim_of_that() {
        assert_suggestion_result(
            "This sub is already filled to the brim of that topic.",
            FullToTheBrim::default(),
            "This sub is already filled to the brim with that topic.",
        );
    }

    #[test]
    fn fix_filled_to_the_brim_of_these() {
        assert_suggestion_result(
            "I'll never forget the time an internet friend of mine showed my a 50 quart bin filled to the brim of these lil buggers.",
            FullToTheBrim::default(),
            "I'll never forget the time an internet friend of mine showed my a 50 quart bin filled to the brim with these lil buggers.",
        );
    }

    #[test]
    fn fix_full_to_the_brim_of_those() {
        assert_suggestion_result(
            "Romance fantasy is full to the brim of those.",
            FullToTheBrim::default(),
            "Romance fantasy is full to the brim with those.",
        );
    }

    // Known false negatives with determiners other than "the"

    #[test]
    #[ignore = "Known false negative"]
    fn fix_filled_to_the_brim_of_his_soul() {
        assert_suggestion_result(
            "It is ingrained into his very being to be the kind of kid who is a leader, a solver of problems, and filled to the brim of his soul with love",
            FullToTheBrim::default(),
            "It is ingrained into his very being to be the kind of kid who is a leader, a solver of problems, and full to the brim of his soul with love",
        );
    }
}
