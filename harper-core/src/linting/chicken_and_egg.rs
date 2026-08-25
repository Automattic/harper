use crate::{
    CharStringExt, Lint, Lrc, Token, TokenStringExt,
    expr::{All, Expr, FirstMatchOf, LongestMatchOf, OwnedExprExt, SequenceExpr, SpaceOrHyphen},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};

const HEAD_NOUNS: &[&str] = &[
    "argument",
    "arguments",
    "conundrum",
    "conundrums",
    "dilemma",
    "dilemmas",
    "problem",
    "problems",
    "question",
    "questions",
    "scenario",
    "scenarios",
    "situation",
    "situations",
    "syndrome",
    "syndromes",
];

pub struct ChickenAndEgg {
    expr: All,
}

impl Default for ChickenAndEgg {
    fn default() -> Self {
        let standard_idiom = SequenceExpr::aco("chicken").then_any_of([
            Box::new(
                SequenceExpr::whitespace()
                    .t_aco("and")
                    .then_hyphen()
                    .t_aco("egg"),
            ),
            Box::new(SequenceExpr::whitespace().t_aco("and").t_ws().t_aco("egg")),
        ]);

        let conj_word = Lrc::new(WordSet::new(&["and", "an", "or", "vs.", "versus"]));
        let conj_sym = |t: &Token, _: &[char]| t.kind.is_ampersand() || t.kind.is_slash();

        let separated_conj = SequenceExpr::default()
            .t_ws_h()
            .then_any_of([
                Box::new(conj_word.clone()) as Box<dyn Expr>,
                Box::new(conj_sym),
            ])
            .t_ws_h();

        let separated_conj_det = SequenceExpr::default()
            .t_ws_h()
            .then_any_of([Box::new(conj_word) as Box<dyn Expr>, Box::new(conj_sym)])
            .t_ws_h()
            .then_determiner()
            .t_ws_h();

        // " and " / "-and-" / " & "
        // "&" / "/"
        // " " / "-"
        let conjunction = FirstMatchOf::new([
            Box::new(separated_conj) as Box<dyn Expr>,
            Box::new(conj_sym),
            Box::new(SpaceOrHyphen::default()),
        ]);

        Self {
            expr: SequenceExpr::word_set(&["chicken", "chickens"])
                .then_optional(LongestMatchOf::new([
                    Box::new(conjunction) as Box<dyn Expr>,
                    Box::new(separated_conj_det),
                ]))
                .t_set(&["egg", "eggs"])
                .t_ws()
                .t_set(HEAD_NOUNS)
                .but_not(standard_idiom),
        }
    }
}

impl ExprLinter for ChickenAndEgg {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // Double-check if it's already a standard variant and abort flagging if so.
        if toks.len() == 7
            && toks[0].kind.is_singular_noun()
            && ((toks[1].kind.is_whitespace() && toks[3].kind.is_whitespace())
                || (toks[1].kind.is_hyphen() && toks[3].kind.is_hyphen()))
            && toks[4].kind.is_singular_noun()
            && toks[0].get_ch(src).eq_str("chicken")
            && toks[2]
                .get_ch(src)
                .eq_any_ignore_ascii_case_str(&["and", "&"])
            && toks[4].get_ch(src).eq_str("egg")
        {
            return None;
        }

        let sep = [" ", "-"][toks[1].kind.is_hyphen() as usize];

        // retain "and" vs "&" and "-" vs " " but avoid "-&-"
        let conj = match toks.len() {
            // chicken and egg X / chicken and the egg X
            7 | 9 => ["&", "and"][toks[2].kind.is_word() as usize],
            // chicken egg X / chicken-egg X / chicken&egg X / chicken/egg X
            5 => ["&", "and"][(toks[1].kind.is_word() || sep == "-") as usize],
            _ => return None,
        };

        let span = toks[..toks.len() - 2].span()?;
        eprintln!("span: {:?}", span.get_content_string(src));

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                &format!("chicken{sep}{conj}{sep}egg"),
                span.get_content(src),
            )],
            message: "If you're referring to “which coame first”, use the standard idiom."
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects wrong variants of the `chicken-and-egg` idiom."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ChickenAndEgg;

    #[test]
    fn fix_chicken_hyphen_egg() {
        assert_suggestion_result(
            "but yep, can confirm the chicken-egg problem and drives me nuts",
            ChickenAndEgg::default(),
            "but yep, can confirm the chicken-and-egg problem and drives me nuts",
        );
    }

    #[test]
    fn dont_flag_hyphenated_and() {
        assert_no_lints(
            "Solving a chicken-and-egg problem when using Flux",
            ChickenAndEgg::default(),
        );
    }

    #[test]
    fn dont_flag_title_case_spaces_ampersand() {
        assert_no_lints(
            "How to solve Chicken & Egg problem",
            ChickenAndEgg::default(),
        )
    }

    #[test]
    fn fix_an_problem() {
        assert_suggestion_result(
            "suspect it's a chicken an egg problem because peerings like subnets are not resources in their own right",
            ChickenAndEgg::default(),
            "suspect it's a chicken and egg problem because peerings like subnets are not resources in their own right",
        );
    }

    #[test]
    fn fix_an_situation() {
        assert_suggestion_result(
            "However, this is a chicken an egg situation in that ruby won't be installed until puppet is",
            ChickenAndEgg::default(),
            "However, this is a chicken and egg situation in that ruby won't be installed until puppet is",
        );
    }

    #[test]
    fn fix_an_argument() {
        assert_suggestion_result(
            "an explanation of what the whole chicken an egg argument is all about",
            ChickenAndEgg::default(),
            "an explanation of what the whole chicken and egg argument is all about",
        );
    }

    #[test]
    #[ignore = "currently requires 'problems' to immediately follow the idiom"]
    fn fix_an_in_double_quotes() {
        assert_suggestion_result(
            "took a crack at bootstrapping UML 2 but I found there were \"chicken an egg\" problems",
            ChickenAndEgg::default(),
            "took a crack at bootstrapping UML 2 but I found there were \"chicken and egg\" problems",
        );
    }

    #[test]
    #[ignore = "currently requires 'scenario' to immediately follow the idiom"]
    fn fix_an_in_single_quotes() {
        assert_suggestion_result(
            "It's an unfalsifiable 'chicken-an-egg' scenario.",
            ChickenAndEgg::default(),
            "It's an unfalsifiable 'chicken-and-egg' scenario.",
        );
    }

    #[test]
    fn fix_vs_in_double_quotes() {
        assert_suggestion_result(
            "0.12upgrade and init have a \"chicken vs egg problem\"",
            ChickenAndEgg::default(),
            "0.12upgrade and init have a \"chicken vs egg problem\"",
        );
    }

    #[test]
    fn fix_or() {
        assert_suggestion_result(
            "It's kind of a chicken or egg situation LOL.",
            ChickenAndEgg::default(),
            "It's kind of a chicken and egg situation LOL.",
        );
    }

    #[test]
    fn fix_slash() {
        assert_suggestion_result(
            "build on payloads, today, to break the chicken/egg situation",
            ChickenAndEgg::default(),
            "build on payloads, today, to break the chicken & egg situation",
        );
    }

    #[test]
    fn fix_or_hyhens() {
        assert_suggestion_result(
            "How to solve this chicken-or-egg conundrum?",
            ChickenAndEgg::default(),
            "How to solve this chicken-and-egg conundrum?",
        );
    }

    #[test]
    fn fix_and_the() {
        assert_suggestion_result(
            "I worked around the chicken and the egg problem by starting kubelet-rubber-stamp outside a container.",
            ChickenAndEgg::default(),
            "I worked around the chicken and egg problem by starting kubelet-rubber-stamp outside a container.",
        );
    }

    #[test]
    fn fix_or_the_egg() {
        assert_suggestion_result(
            "consul/terraform - chicken or the egg dilemma",
            ChickenAndEgg::default(),
            "consul/terraform - chicken and egg dilemma",
        );
    }
}
