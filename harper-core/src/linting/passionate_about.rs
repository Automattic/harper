use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, LongestMatchOf, OwnedExprExt, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, preceded_by_word},
    },
    patterns::{InflectionOfBe, Word},
};

pub struct PassionateAbout {
    expr: LongestMatchOf,
}

impl Default for PassionateAbout {
    fn default() -> Self {
        Self {
            expr: LongestMatchOf::new(vec![
                Box::new(
                    SequenceExpr::aco("passionate")
                        .t_ws()
                        .then_preposition()
                        .but_not(SequenceExpr::anything().t_any().t_aco("about")),
                ) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::default()
                        .then_preposition()
                        .t_ws()
                        .t_aco("which")
                        .t_ws()
                        .then_subject_pronoun()
                        .t_ws()
                        .then(InflectionOfBe::new())
                        .t_ws()
                        .t_aco("passionate")
                        .then_optional(SequenceExpr::whitespace().then_preposition())
                        .but_not(Word::new("about")),
                ),
            ]),
        }
    }
}

impl ExprLinter for PassionateAbout {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        #[derive(Debug)]
        enum PrepInfo {
            Single(usize),
            Dual,
        }

        let prep_info = match toks.len() {
            3 => PrepInfo::Single(2),
            9 => PrepInfo::Single(0),
            11 => PrepInfo::Dual,
            _ => return None,
        };

        match prep_info {
            PrepInfo::Single(idx) => {
                let span = toks[idx].span;

                Some(Lint {
                    span,
                    lint_kind: LintKind::Usage,
                    suggestions: vec![Suggestion::replace_with_match_case_str(
                        "about",
                        span.get_content(src),
                    )],
                    message: "Use `about` instead of `of` with `passionate`".to_owned(),
                    ..Default::default()
                })
            }
            PrepInfo::Dual => {
                // Allow "all/both of which ... about"
                if toks.first()?.get_ch(src).eq_str("of")
                    && toks.last()?.get_ch(src).eq_str("about")
                    && preceded_by_word(ctx, |t| {
                        t.get_ch(src).eq_any_ignore_ascii_case_str(&["all", "both"])
                    })
                {
                    return None;
                }

                let [prep_first, prep_last] =
                    [1..=8, 2..=9].map(|r| toks[r].span().and_then(|s| Some(s.get_content(src))));

                let span = toks.span()?;
                let content = span.get_content(src);

                Some(Lint {
                    span,
                    lint_kind: LintKind::Usage,
                    suggestions: vec![
                        Suggestion::replace_with_match_case(
                            "about".chars().chain(prep_first?.iter().copied()).collect(),
                            content,
                        ),
                        Suggestion::replace_with_match_case(
                            prep_last?.iter().copied().chain("about".chars()).collect(),
                            content,
                        ),
                    ],
                    message: "Use `about` instead of `of` with `passionate`".to_owned(),
                    ..Default::default()
                })
            }
        }
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `passtionate of` to `passionate about`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{
        assert_good_and_bad_suggestions, assert_no_lints, assert_suggestion_result,
    };

    use super::PassionateAbout;

    // Straightforward cases

    #[test]
    fn fix_passionate_of() {
        assert_suggestion_result(
            "if a human being is writing about something you enjoy or are passionate of, you firstly have a community and are able to talk with a actual person ...",
            PassionateAbout::default(),
            "if a human being is writing about something you enjoy or are passionate about, you firstly have a community and are able to talk with a actual person ...",
        );
    }

    #[test]
    fn fix_of_which_we() {
        assert_suggestion_result(
            "... and become a thing of which we are passionate and which, through misapplication ...",
            PassionateAbout::default(),
            "... and become a thing about which we are passionate and which, through misapplication ...",
        );
    }

    #[test]
    fn fix_passionate_of_embodied_ai() {
        assert_suggestion_result(
            "Passionate of Embodied AI from Poland.",
            PassionateAbout::default(),
            "Passionate about Embodied AI from Poland.",
        );
    }

    #[test]
    fn fix_of_which_they() {
        assert_suggestion_result(
            "... part of organizations and causes on campus, of which they are passionate.",
            PassionateAbout::default(),
            "... part of organizations and causes on campus, about which they are passionate.",
        );
    }

    #[test]
    fn fix_passionate_of_tech_ai_math_finance() {
        assert_suggestion_result(
            "Computer scientist, passionate of tech, AI, math finance.",
            PassionateAbout::default(),
            "Computer scientist, passionate about tech, AI, math finance.",
        );
    }

    #[test]
    fn fix_passionate_of_web_mapping() {
        assert_suggestion_result(
            "A geographer turned software engineer, passionate of web mapping, data engineering, analytics, and visualization.",
            PassionateAbout::default(),
            "A geographer turned software engineer, passionate about web mapping, data engineering, analytics, and visualization.",
        );
    }

    #[test]
    fn fix_passionate_of_automation() {
        assert_suggestion_result(
            "I am passionate of automation because: I like to make my life easier; I like to make my work easier. Examples of projects in IT field, of automations I ...Read more",
            PassionateAbout::default(),
            "I am passionate about automation because: I like to make my life easier; I like to make my work easier. Examples of projects in IT field, of automations I ...Read more",
        );
    }

    // Tricky cases

    #[test]
    fn dont_flag_both_of_which_he_was_passionate_about() {
        assert_no_lints(
            "... give up his research on glioma and being a neurosurgeon both of which he was passionate about to go into the black hole that was researching a disease ...",
            PassionateAbout::default(),
        );
    }

    #[test]
    #[ignore = "Both suggestions are broken due to #3741"]
    fn fix_passionate_about_in_parentheses() {
        // Both suggestions are broken due to #3741
        // about whIch i am passionate
        // which i aM passionate about
        assert_good_and_bad_suggestions(
            "Watching/ reading documentaries; anything history-related, especially about Richard III (of which I am passionate about) and ...",
            PassionateAbout::default(),
            &[
                "Watching/ reading documentaries; anything history-related, especially about Richard III (about which I am passionate) and ...",
                "Watching/ reading documentaries; anything history-related, especially about Richard III (which I am passionate about) and ...",
            ],
            &[],
        );
    }

    #[test]
    fn dont_flag_all_of_which_he_was_passionate_about() {
        assert_no_lints(
            "he could be found spending time on the pontoon boat at Little Sebago Lake, all of which he was passionate about.",
            PassionateAbout::default(),
        );
    }

    #[test]
    fn fix_doing_something() {
        assert_good_and_bad_suggestions(
            "If I was going to go out and compete with my brothers, I wanted to make sure I was doing something of which I was passionate.",
            PassionateAbout::default(),
            &[
                "If I was going to go out and compete with my brothers, I wanted to make sure I was doing something about which I was passionate.",
                "If I was going to go out and compete with my brothers, I wanted to make sure I was doing something which I was passionate about.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_environmetal_fields() {
        assert_good_and_bad_suggestions(
            "I choose EEP because I wanted to learn how to better advocate for and make changes in the marine and environmental fields of which I am passionate.",
            PassionateAbout::default(),
            &[
                "I choose EEP because I wanted to learn how to better advocate for and make changes in the marine and environmental fields about which I am passionate.",
                "I choose EEP because I wanted to learn how to better advocate for and make changes in the marine and environmental fields which I am passionate about.",
            ],
            &[],
        );
    }
}
