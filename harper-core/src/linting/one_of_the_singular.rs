use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    spell::Dictionary,
};

pub struct OneOfTheSingular<D: Dictionary + 'static> {
    expr: Box<dyn Expr>,
    dict: D,
}

pub trait SeqExprExt {
    fn then_my_noun_or_adjective(self) -> Self;
}

impl SeqExprExt for SequenceExpr {
    fn then_my_noun_or_adjective(self) -> Self {
        self.then(|t: &Token, s: &[char]| {
            // We can't restrict to singular nouns because it will match and then flag the part of a phrase
            // leading up to but not including a plural noun, even when that plural noun is the result of
            // this linter having corrected a mistake.
            // eg. "one of the train station" -> "one of the train stations" will now match on "one of the train".
            (t.kind.is_non_possessive_noun() || t.kind.is_adjective())
                && !t.kind.is_preposition() // "in" etc.
                && !t.kind.is_pronoun() // "who" etc.
                && !t
                    .span
                    .get_content(s)
                    .eq_any_ignore_ascii_case_str(&["ah", "few", "first", "said", "uh"])
        })
    }
}

impl<D: Dictionary + 'static> OneOfTheSingular<D> {
    pub fn new(dict: D) -> Self {
        let advs =
            SequenceExpr::default().then_zero_or_more_spaced(SequenceExpr::default().then_adverb());

        let adj_or_nouns = SequenceExpr::default()
            .then_zero_or_more_spaced(SequenceExpr::default().then_my_noun_or_adjective());

        Self {
            expr: Box::new(
                SequenceExpr::fixed_phrase("one of the ")
                    .then(SequenceExpr::optional(advs.t_ws()).then(adj_or_nouns)),
            ),
            dict,
        }
    }
}

impl<D: Dictionary + 'static> ExprLinter for OneOfTheSingular<D> {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects 'one of the [singular]' to 'one of the [plural]'"
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let nountok = toks.last()?;
        // It's only a mistake if the noun phrase ends with a singular noun.
        if !nountok.kind.is_singular_noun()
        // "Being" in eg. one of the most widely used being the bi-directional inference algorithm.
        || nountok.kind.is_verb_progressive_form()
        {
            return None;
        }
        // If any token is a singular noun that's not also a plural noun, don't continue.
        if toks
            .iter()
            .any(|t| t.kind.is_plural_noun() && !t.kind.is_singular_noun())
        {
            return None;
        }
        // But if not if it's part of a hyphenated compound.
        if let Some(next) = ctx?.1.first()
            && (next.kind.is_hyphen() || next.kind.is_comma())
        {
            return None;
        }
        let nounspan = nountok.span;
        let singular = nounspan.get_content(src);
        let mut plural_s = singular.to_vec();
        let mut plural_es = singular.to_vec();
        plural_s.push('s');
        plural_es.extend(['e', 's']);

        let mut suggestions = vec![];

        if self
            .dict
            .get_word_metadata(&plural_s)
            .is_some_and(|m| m.is_plural_noun())
        {
            suggestions.push(Suggestion::replace_with_match_case(plural_s, singular));
        }
        if self
            .dict
            .get_word_metadata(&plural_es)
            .is_some_and(|m| m.is_plural_noun())
        {
            suggestions.push(Suggestion::replace_with_match_case(plural_es, singular));
        }

        if singular.ends_with_ignore_ascii_case_chars(&['y']) {
            // Handle words ending in 'y' -> 'ies' (e.g., "city" -> "cities")
            let mut plural_ies = singular[..singular.len() - 1].to_vec();
            plural_ies.extend(['i', 'e', 's']);
            if self
                .dict
                .get_word_metadata(&plural_ies)
                .is_some_and(|m| m.is_plural_noun())
            {
                suggestions.push(Suggestion::replace_with_match_case(plural_ies, singular));
            }
        }

        if singular.ends_with_ignore_ascii_case_chars(&['f', 'e']) {
            // Handle words ending in 'fe' -> 'ves' (e.g., "wife" -> "wives")
            let mut plural_ves = singular[..singular.len() - 2].to_vec();
            plural_ves.extend(['v', 'e', 's']);
            if self
                .dict
                .get_word_metadata(&plural_ves)
                .is_some_and(|m| m.is_plural_noun())
            {
                suggestions.push(Suggestion::replace_with_match_case(plural_ves, singular));
            }
        }

        Some(Lint {
            span: nounspan,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "The construction `one of the ...` should use a plural noun.".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::OneOfTheSingular;
    use crate::linting::tests::{assert_no_lints, assert_top3_suggestion_result};
    use crate::spell::FstDictionary;

    #[test]
    fn fix_one_of_the_noun() {
        assert_top3_suggestion_result(
            "one of the noun",
            OneOfTheSingular::new(FstDictionary::curated()),
            "one of the nouns",
        );
    }

    #[test]
    fn fix_one_of_the_noun_noun() {
        assert_top3_suggestion_result(
            "one of the car park",
            OneOfTheSingular::new(FstDictionary::curated()),
            "one of the car parks",
        );
    }

    #[test]
    fn fix_one_of_the_adj_noun() {
        assert_top3_suggestion_result(
            "one of the best noun",
            OneOfTheSingular::new(FstDictionary::curated()),
            "one of the best nouns",
        );
    }

    #[test]
    fn fix_one_of_the_adv_adv_adj_adj_noun_noun() {
        assert_top3_suggestion_result(
            "one of the really incredibly big red rubber ball",
            OneOfTheSingular::new(FstDictionary::curated()),
            "one of the really incredibly big red rubber balls",
        );
    }

    #[test]
    fn fix_one_of_the_best_tutorial() {
        assert_top3_suggestion_result(
            "Bro casually dropped one of the best graphics tutorial I've ever seen and thought we wouldn't notice",
            OneOfTheSingular::new(FstDictionary::curated()),
            "Bro casually dropped one of the best graphics tutorials I've ever seen and thought we wouldn't notice",
        );
    }

    #[test]
    fn fix_one_of_the_neat_trick() {
        assert_top3_suggestion_result(
            "One of the neat trick with AVX-512 is that given a mask",
            OneOfTheSingular::new(FstDictionary::curated()),
            "One of the neat tricks with AVX-512 is that given a mask",
        );
    }

    #[test]
    fn fix_one_of_the_latest_version() {
        assert_top3_suggestion_result(
            "Footer line shown since one of the latest version",
            OneOfTheSingular::new(FstDictionary::curated()),
            "Footer line shown since one of the latest versions",
        );
    }

    #[test]
    fn fix_one_of_the_node() {
        assert_top3_suggestion_result(
            "... noticed occasional production issue when one of the node loses connection",
            OneOfTheSingular::new(FstDictionary::curated()),
            "... noticed occasional production issue when one of the nodes loses connection",
        );
    }

    #[test]
    fn fix_one_of_the_unstaged_file() {
        assert_top3_suggestion_result(
            "Sublime Merge hangs if one of the unstaged file is a pretty ...",
            OneOfTheSingular::new(FstDictionary::curated()),
            "Sublime Merge hangs if one of the unstaged files is a pretty ...",
        );
    }

    #[test]
    fn fix_one_of_the_tedious_things() {
        assert_top3_suggestion_result(
            "One of the tedious thing in Stack Overflow is to grab example data provided by users",
            OneOfTheSingular::new(FstDictionary::curated()),
            "One of the tedious things in Stack Overflow is to grab example data provided by users",
        );
    }

    #[test]
    fn fix_one_of_the_brave_process() {
        assert_top3_suggestion_result(
            "One of the Brave Process is consuming almost 170%",
            OneOfTheSingular::new(FstDictionary::curated()),
            "One of the Brave Processes is consuming almost 170%",
        );
    }

    #[test]
    fn fix_one_of_the_most_cumbersome_thing() {
        assert_top3_suggestion_result(
            "One of the most cumbersome thing to create in markdown is a table.",
            OneOfTheSingular::new(FstDictionary::curated()),
            "One of the most cumbersome things to create in markdown is a table.",
        );
    }

    #[test]
    fn fix_one_of_the_test() {
        assert_top3_suggestion_result(
            "Not passing one of the test",
            OneOfTheSingular::new(FstDictionary::curated()),
            "Not passing one of the tests",
        );
    }

    #[test]
    fn fix_one_of_the_process_main_thread() {
        assert_top3_suggestion_result(
            "And those threads life cycle is very long, sometimes, it will be one of the process main thread",
            OneOfTheSingular::new(FstDictionary::curated()),
            "And those threads life cycle is very long, sometimes, it will be one of the process main threads",
        );
    }

    #[test]
    fn dont_flag_being() {
        assert_no_lints(
            "HMMs underlie the functioning of stochastic taggers and are used in various algorithms one of the most widely used being the bi-directional inference algorithm.",
            OneOfTheSingular::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn dont_flag_one_of_the_rabbits_gloves() {
        assert_no_lints(
            "As she said this she looked down at her hands, and was surprised to see that she had put on one of the Rabbit’s little white kid gloves while she was talking.",
            OneOfTheSingular::new(FstDictionary::curated()),
        );
    }
}
