use harper_brill::UPOS;

use crate::CharStringExt;
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    dict_word_metadata::Person,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct NominalWants {
    expr: Box<dyn Expr>,
}

impl Default for NominalWants {
    fn default() -> Self {
        fn is_applicable_pronoun(tok: &Token, src: &[char]) -> bool {
            if tok.kind.is_pronoun() && tok.kind.is_upos(UPOS::PRON) {
                let pron = tok.span.get_content(src);
                !pron.eq_any_ignore_ascii_case_chars(&[
                    // "That" can act as two kinds of pronoun: demonstrative and relative.
                    // As a demonstrative pronoun, it's third person singular.
                    // As a relative pronoun, it's behaves as any person:
                    // I am the one that wants to. He is the one that wants to.
                    &['t', 'h', 'a', 't'],
                    // Personal pronouns have case. Object case personal pronouns
                    // can come after "want":
                    // Make them want to believe.
                    // Note: "you" and "it" are both subject and object case.
                    &['m', 'e'],
                    &['u', 's'],
                    // "you" is subject and object both OK before "want".
                    &['h', 'i', 'm'],
                    &['h', 'e', 'r'],
                    // "it" is both subject and object. Subject before "wants", object before "want".
                    &['i', 't'],
                    &['t', 'h', 'e', 'm'],
                    &['w', 'h', 'o'],
                ])
            } else {
                false
            }
        }

        let miss = WordSet::new(&["wont", "wonts", "want", "wants"]);
        let pattern = SequenceExpr::with(is_applicable_pronoun)
            .then_whitespace()
            .then(miss);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for NominalWants {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let subject = toks.first()?;
        let offender = &toks.last()?;

        let plural = subject.kind.is_plural_nominal();

        let person = subject
            .kind
            .as_word()
            .unwrap()
            .clone()
            .unwrap()
            .pronoun
            .and_then(|p| p.person)
            .unwrap_or(Person::Third);

        let replacement = if person == Person::Third {
            if plural { "want" } else { "wants" }
        } else {
            "want"
        };

        let replacement_chars: Vec<char> = replacement.chars().collect();

        let offender_span = offender.span;
        let offender_chars = offender_span.get_content(source);

        if offender_chars.eq_ignore_ascii_case_chars(&replacement_chars) {
            return None;
        }

        Some(Lint {
            span: offender_span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case(
                replacement_chars,
                offender_chars,
            )],
            message: format!("Did you mean `{replacement}`?"),
            priority: 55,
        })
    }

    fn description(&self) -> &str {
        "Ensures you use the correct `want` / `wants` after a nominal."
    }
}

#[cfg(test)]
mod tests {
    use super::NominalWants;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn fixes_he_wonts() {
        assert_suggestion_result(
            "He wonts to join us.",
            NominalWants::default(),
            "He wants to join us.",
        );
    }

    #[test]
    #[ignore = "This is not a grammar error if the previous word is `help`, `let`, or `make`."]
    fn fixes_it_wont() {
        assert_suggestion_result(
            "It wont to move forward.",
            NominalWants::default(),
            "It wants to move forward.",
        );
    }

    #[test]
    fn fixes_she_wont() {
        assert_suggestion_result(
            "She wont to leave early.",
            NominalWants::default(),
            "She wants to leave early.",
        );
    }

    #[test]
    fn fixes_i_wont() {
        assert_suggestion_result(
            "I wonts to leave early.",
            NominalWants::default(),
            "I want to leave early.",
        );
    }

    #[test]
    fn allows_you_want() {
        assert_lint_count("What size do you want to be?", NominalWants::default(), 0);
    }

    #[test]
    fn fixes_you_wants() {
        assert_suggestion_result(
            "What do you wants?",
            NominalWants::default(),
            "What do you want?",
        );
    }

    #[test]
    fn ignores_correct_usage_they() {
        assert_lint_count("They want to help.", NominalWants::default(), 0);
    }

    #[test]
    fn ignores_correct_usage_he() {
        assert_lint_count("He wants to help.", NominalWants::default(), 0);
    }

    #[test]
    fn ignores_correct_usage_that_1298() {
        assert_lint_count(
            "The projects that want to take it seriously are the best.",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_make_me() {
        assert_lint_count(
            "Take another person code make me want to die.",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_makes_me() {
        assert_lint_count(
            "It makes me want to not use GitHub at all.",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_make_us() {
        assert_lint_count(
            "... try harder to make us want to implement it.",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_made_us() {
        assert_lint_count(
            "This change made us want to adopt luxon's strict mode",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_help_us() {
        assert_lint_count("... help us want to help you.", NominalWants::default(), 0);
    }

    #[test]
    fn ignores_correct_usage_make_you() {
        assert_lint_count(
            "I can certainly see why that would make you want to ditch Linux packaging.",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_makes_you() {
        assert_lint_count(
            "If something happens that makes you want to scream from the top of your lungs",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_made_you() {
        assert_lint_count(
            "What made you want to leave the LibFuzzer ...",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_make_him() {
        assert_lint_count(
            "make him want to help with your issue",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_make_her() {
        assert_lint_count(
            "... and make her want to get into coding.",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_make_it() {
        assert_lint_count(
            "you just make it want to appear as a drama",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_makes_it() {
        assert_lint_count(
            "using UHD makes it want to put labels in the corner saying UHD",
            NominalWants::default(),
            0,
        );
    }

    #[test]
    fn ignores_correct_usage_make_them() {
        assert_lint_count(
            "And make them want to believe in it.",
            NominalWants::default(),
            0,
        )
    }

    #[test]
    fn ignores_correct_usage_making_them() {
        assert_lint_count(
            "you're annoying ALMOST ALL of the users and making them want to switch to another ...",
            NominalWants::default(),
            0,
        )
    }

    #[test]
    fn ignores_correct_usage_help_them() {
        assert_lint_count("And help them want to do it.", NominalWants::default(), 0)
    }

    #[test]
    fn allows_want_to() {
        assert_no_lints(
            "Harper is a grammar checker for people who want to write fast.",
            NominalWants::default(),
        );
    }

    #[test]
    fn test_2007() {
        assert_no_lints(
            "### 🙌 **We Want to Hear From You!**",
            NominalWants::default(),
        )
    }
}
