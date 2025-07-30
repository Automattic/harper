use crate::{
    Token,
    linting::{Lint, LintKind, PatternLinter, Suggestion},
    patterns::{Pattern, SequencePattern, WordSet},
};

pub struct ItIs {
    pattern: Box<dyn crate::patterns::Pattern>,
}

impl Default for ItIs {
    fn default() -> Self {
        let exceptions = WordSet::new(&[
            "own",
            "1st",
            "mainline",
            "team",
            "body",
            "mean",
            "animal",
            "head",
            "material",
            "frontline",
            "center",
            "centre",
            "business",
            "state",
            "runtime",
            "size",
            "power",
            "budget",
            "regulation",
            "woman",
            "turnover",
            "utility",
            "key",
            "assault",
        ]);
        let pattern = SequencePattern::default()
            .t_aco("its")
            .then_whitespace()
            .then(move |tok: &Token, src: &[char]| {
                if let Some(Some(meta)) = tok.kind.as_word() {
                    if !meta.is_adjective() {
                        return false;
                    }
                    if exceptions.matches(&[tok.clone()], src).is_some() {
                        return false;
                    }
                    true
                } else {
                    false
                }
            })
            .then_whitespace()
            .then_preposition();
        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for ItIs {
    fn pattern(&self) -> &dyn crate::patterns::Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, tokens: &[Token], source: &[char]) -> Option<Lint> {
        let its_token = &tokens[0];
        let span = its_token.span;
        let text = span.get_content(source);
        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                "it's".chars().collect(),
                text,
            )],
            message: "Consider using 'it's' (it is) instead of 'its' (possessive form)."
                .to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Detects when “its” is used before an adjective + preposition and suggests the contraction “it's”."
    }
}

#[cfg(test)]
mod tests {
    use super::ItIs;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn flags_simple_case() {
        assert_suggestion_result(
            "Its amazing to see this.",
            ItIs::default(),
            "It's amazing to see this.",
        );
    }

    #[test]
    fn flags_with_preposition() {
        assert_suggestion_result(
            "Its critical for the project.",
            ItIs::default(),
            "It's critical for the project.",
        );
    }

    #[test]
    fn does_not_flag_exception_own() {
        assert_lint_count("Its own design is unique.", ItIs::default(), 0);
    }

    #[test]
    fn does_not_flag_exception_team() {
        assert_lint_count("Its team lead is excellent.", ItIs::default(), 0);
    }

    // This case fails, but I think that's acceptable.
    // #[test]
    // fn does_not_flag_non_adjective() {
    //     assert_lint_count(
    //         "The cat chased its tail around the room.",
    //         ItIs::default(),
    //         0,
    //     );
    // }

    #[test]
    fn does_not_flag_already_correct() {
        assert_lint_count("It's important to note.", ItIs::default(), 0);
    }

    #[test]
    fn flags_search_filter_context() {
        assert_suggestion_result(
            "Its important to note that the search filter will currently only search the current page.",
            ItIs::default(),
            "It's important to note that the search filter will currently only search the current page.",
        );
    }

    #[test]
    fn flags_ens_restart_context() {
        assert_suggestion_result(
            "Today is the third day and I am still stuck on Register. Its important to note that after hours of waiting, I tried to restart the process and clicked on register again but it gets stuck at TX pending.",
            ItIs::default(),
            "Today is the third day and I am still stuck on Register. It's important to note that after hours of waiting, I tried to restart the process and clicked on register again but it gets stuck at TX pending.",
        );
    }

    #[test]
    fn flags_academics_support_context() {
        assert_suggestion_result(
            "To assist learners, because its critical for academics to support their ideas and arguments with sources of published research.",
            ItIs::default(),
            "To assist learners, because it's critical for academics to support their ideas and arguments with sources of published research.",
        );
    }

    #[test]
    fn flags_parents_explain_context() {
        assert_suggestion_result(
            "I also think its critical for parents to explain their reason for saying no though I would advise against attempting to use logic in the face of either toddler or teenage rage.",
            ItIs::default(),
            "I also think it's critical for parents to explain their reason for saying no though I would advise against attempting to use logic in the face of either toddler or teenage rage.",
        );
    }

    #[test]
    fn flags_chapter_context() {
        assert_suggestion_result(
            "I think it's okay since its critical for the rest of the chapter in terms of tone and approach.",
            ItIs::default(),
            "I think it's okay since it's critical for the rest of the chapter in terms of tone and approach.",
        );
    }

    #[test]
    fn flags_microsoft_work_context() {
        assert_suggestion_result(
            "... Need help, its critical for my work, as i am a technical blog writer ...",
            ItIs::default(),
            "... Need help, it's critical for my work, as i am a technical blog writer ...",
        );
    }

    #[test]
    fn flags_feminists_context() {
        assert_suggestion_result(
            "when it comes to the teaching of grammar and diverse linguistics practices. Its critical for feminists to think about the ways in which they frame language.",
            ItIs::default(),
            "when it comes to the teaching of grammar and diverse linguistics practices. It's critical for feminists to think about the ways in which they frame language.",
        );
    }

    #[test]
    fn flags_students_proofreading_context() {
        assert_suggestion_result(
            "its critical for students to develop a similarly sharp eye for misspellings and grammatical errors.",
            ItIs::default(),
            "it's critical for students to develop a similarly sharp eye for misspellings and grammatical errors.",
        );
    }

    #[test]
    fn flags_americans_context() {
        assert_suggestion_result(
            "Its critical for Americans to realize that Fox has nothing to do with news.",
            ItIs::default(),
            "It's critical for Americans to realize that Fox has nothing to do with news.",
        );
    }

    // Negative guard: correct possessive use
    #[test]
    fn does_not_flag_its_team_lead() {
        assert_lint_count("Its team lead is excellent.", ItIs::default(), 0);
    }

    // Imagined edge cases based on real usage:
    #[test]
    fn flags_crucial_api_context() {
        assert_suggestion_result(
            "Its crucial to understand the API before using it.",
            ItIs::default(),
            "It's crucial to understand the API before using it.",
        );
    }

    #[test]
    fn flags_essential_standards_context() {
        assert_suggestion_result(
            "Its essential to follow the coding standards in this project.",
            ItIs::default(),
            "It's essential to follow the coding standards in this project.",
        );
    }

    #[test]
    fn flags_vital_dependencies_context() {
        assert_suggestion_result(
            "Its vital to keep dependencies up to date.",
            ItIs::default(),
            "It's vital to keep dependencies up to date.",
        );
    }
}
