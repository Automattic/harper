use crate::TokenStringExt;

use super::{Lint, LintKind, Linter, Suggestion};

/// A linter that makes sure you capitalize "I" and its contractions.
#[derive(Default)]
pub struct CapitalizePersonalPronouns;

impl Linter for CapitalizePersonalPronouns {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        document
            .iter_words()
            .filter_map(|tok| {
                let span_content = document.get_span_content(&tok.span);

                if matches!(
                    span_content,
                    ['i']
                        | ['i', '\'', 'd']
                        | ['i', '\'', 'd', '\\', 'v', 'e']
                        | ['i', '\'', 'l', 'l']
                        | ['i', '\'', 'm']
                        | ['i', '\'', 'v', 'e']
                ) {
                    let mut replacement = span_content.to_vec();
                    replacement[0] = 'I';
                    Some(Lint {
                        span: tok.span,
                        lint_kind: LintKind::Capitalization,
                        suggestions: vec![Suggestion::ReplaceWith(replacement)],
                        message: "The first-person singular subject pronoun must be capitalized."
                            .to_string(),
                        priority: 31,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn description(&self) -> &'static str {
        "Forgetting to capitalize personal pronouns, like \"I\" or \"I'm\" is one of the most common errors. This rule helps with that."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    use super::CapitalizePersonalPronouns;

    #[test]
    fn start() {
        assert_suggestion_result("i am hungry", CapitalizePersonalPronouns, "I am hungry");
    }

    #[test]
    fn end() {
        assert_suggestion_result(
            "There is no one stronger than i",
            CapitalizePersonalPronouns,
            "There is no one stronger than I",
        );
    }

    #[test]
    fn middle() {
        assert_suggestion_result(
            "First of all, i am not happy with this.",
            CapitalizePersonalPronouns,
            "First of all, I am not happy with this.",
        );
    }

    #[test]
    fn issue_365() {
        assert_lint_count(
            "access will succeed, unlike with UDEREF/i386.",
            CapitalizePersonalPronouns,
            0,
        );
    }

    #[test]
    fn corrects_id() {
        assert_suggestion_result("i'd", CapitalizePersonalPronouns, "I'd");
    }

    #[test]
    fn correct_real_world_id() {
        assert_suggestion_result(
            "Personal Homebrew tap with tools i'd like to use",
            CapitalizePersonalPronouns,
            "Personal Homebrew tap with tools I'd like to use",
        )
    }

    #[test]
    fn corrects_idve() {
        assert_suggestion_result("i'd've", CapitalizePersonalPronouns, "I'd've");
    }

    #[test]
    fn correct_real_world_idve() {
        assert_suggestion_result(
            "... i'd've loved this even more twice length , but let not get greedy",
            CapitalizePersonalPronouns,
            "... I'd've loved this even more twice length , but let not get greedy",
        )
    }

    #[test]
    fn corrects_ill() {
        assert_suggestion_result("i'll", CapitalizePersonalPronouns, "I'll");
    }

    #[test]
    fn correct_real_world_ill() {
        assert_suggestion_result(
            "Hey i deploy my contract it give me error and i'll match with the script file both are same if someone have idea how i slove this please ...",
            CapitalizePersonalPronouns,
            "Hey I deploy my contract it give me error and I'll match with the script file both are same if someone have idea how I slove this please ...",
        )
    }

    #[test]
    fn corrects_im() {
        assert_suggestion_result("i'm", CapitalizePersonalPronouns, "I'm");
    }

    #[test]
    fn correct_real_world_im() {
        assert_suggestion_result(
            "Grid view not working, i'm not using any template",
            CapitalizePersonalPronouns,
            "Grid view not working, I'm not using any template",
        )
    }

    #[test]
    fn corrects_ive() {
        assert_suggestion_result("i've", CapitalizePersonalPronouns, "I've");
    }

    #[test]
    fn correct_real_world_ive() {
        assert_suggestion_result(
            "Can't use Github Pro although i've verified for student pack",
            CapitalizePersonalPronouns,
            "Can't use Github Pro although I've verified for student pack",
        )
    }
}
