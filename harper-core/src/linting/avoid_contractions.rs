use crate::Token;
use crate::expr::Expr;
use crate::linting::expr_linter::Chunk;
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion};
use crate::patterns::WordSet;

pub struct AvoidContractions {
    expr: WordSet,
}

impl Default for AvoidContractions {
    fn default() -> Self {
        Self {
            expr: WordSet::new(&[
                "aren't",
                "can't",
                "could've",
                "couldn't",
                "couldn't've",
                "didn't",
                "doesn't",
                "don't",
                "hadn't",
                "hasn't",
                "haven't",
                "he'll",
                "how're",
                "i'll",
                "i'm",
                "i've",
                "isn't",
                "it'll",
                "mayn't",
                "might've",
                "mightn't",
                "must've",
                "mustn't",
                "needn't",
                "oughtn't",
                "shan't",
                "she'll",
                "should've",
                "shouldn't",
                "that'll",
                "they'll",
                "they're",
                "they've",
                "wasn't",
                "we'll",
                "we're",
                "we've",
                "weren't",
                "what'll",
                "who'll",
                "who're",
                "who've",
                "won't",
                "would've",
                "wouldn't",
                "you'll",
                "you're",
                "you've",
            ]),
        }
    }
}

impl AvoidContractions {
    fn expansion(contraction: &str) -> Option<&'static str> {
        match contraction {
            "aren't" => Some("are not"),
            "can't" => Some("cannot"),
            "could've" => Some("could have"),
            "couldn't" => Some("could not"),
            "couldn't've" => Some("could not have"),
            "didn't" => Some("did not"),
            "doesn't" => Some("does not"),
            "don't" => Some("do not"),
            "hadn't" => Some("had not"),
            "hasn't" => Some("has not"),
            "haven't" => Some("have not"),
            "he'll" => Some("he will"),
            "how're" => Some("how are"),
            "i'll" => Some("i will"),
            "i'm" => Some("i am"),
            "i've" => Some("i have"),
            "isn't" => Some("is not"),
            "it'll" => Some("it will"),
            "mayn't" => Some("may not"),
            "might've" => Some("might have"),
            "mightn't" => Some("might not"),
            "must've" => Some("must have"),
            "mustn't" => Some("must not"),
            "needn't" => Some("need not"),
            "oughtn't" => Some("ought not"),
            "shan't" => Some("shall not"),
            "she'll" => Some("she will"),
            "should've" => Some("should have"),
            "shouldn't" => Some("should not"),
            "that'll" => Some("that will"),
            "they'll" => Some("they will"),
            "they're" => Some("they are"),
            "they've" => Some("they have"),
            "wasn't" => Some("was not"),
            "we'll" => Some("we will"),
            "we're" => Some("we are"),
            "we've" => Some("we have"),
            "weren't" => Some("were not"),
            "what'll" => Some("what will"),
            "who'll" => Some("who will"),
            "who're" => Some("who are"),
            "who've" => Some("who have"),
            "won't" => Some("will not"),
            "would've" => Some("would have"),
            "wouldn't" => Some("would not"),
            "you'll" => Some("you will"),
            "you're" => Some("you are"),
            "you've" => Some("you have"),
            _ => None,
        }
    }

    fn normalize(contraction: &str) -> String {
        contraction.replace('’', "'").to_lowercase()
    }
}

impl ExprLinter for AvoidContractions {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok = toks.first()?;
        let contraction = tok.get_str(src);
        let expansion = Self::expansion(&Self::normalize(&contraction))?;

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Style,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                expansion,
                tok.get_ch(src),
            )],
            message: "Consider expanding this contraction.".to_string(),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Suggests expanded forms for common contractions, such as `isn't` → `is not` and `we're` → `we are`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::AvoidContractions;

    #[test]
    fn expands_isnt() {
        assert_suggestion_result(
            "This isn't necessary.",
            AvoidContractions::default(),
            "This is not necessary.",
        );
    }

    #[test]
    fn expands_wasnt() {
        assert_suggestion_result(
            "It wasn't ready.",
            AvoidContractions::default(),
            "It was not ready.",
        );
    }

    #[test]
    fn expands_arent() {
        assert_suggestion_result(
            "They aren't coming.",
            AvoidContractions::default(),
            "They are not coming.",
        );
    }

    #[test]
    fn expands_dont() {
        assert_suggestion_result(
            "We don't need it.",
            AvoidContractions::default(),
            "We do not need it.",
        );
    }

    #[test]
    fn expands_doesnt() {
        assert_suggestion_result(
            "She doesn't agree.",
            AvoidContractions::default(),
            "She does not agree.",
        );
    }

    #[test]
    fn expands_didnt() {
        assert_suggestion_result(
            "He didn't answer.",
            AvoidContractions::default(),
            "He did not answer.",
        );
    }

    #[test]
    fn expands_cant() {
        assert_suggestion_result(
            "You can't go.",
            AvoidContractions::default(),
            "You cannot go.",
        );
    }

    #[test]
    fn expands_wont() {
        assert_suggestion_result(
            "They won't stop.",
            AvoidContractions::default(),
            "They will not stop.",
        );
    }

    #[test]
    fn expands_havent() {
        assert_suggestion_result(
            "I haven't finished.",
            AvoidContractions::default(),
            "I have not finished.",
        );
    }

    #[test]
    fn expands_im() {
        assert_suggestion_result("I'm ready.", AvoidContractions::default(), "I am ready.");
    }

    #[test]
    fn expands_youre() {
        assert_suggestion_result(
            "You're right.",
            AvoidContractions::default(),
            "You are right.",
        );
    }

    #[test]
    fn expands_theyll() {
        assert_suggestion_result(
            "They'll arrive soon.",
            AvoidContractions::default(),
            "They will arrive soon.",
        );
    }

    #[test]
    fn expands_additional_unambiguous_contractions() {
        assert_suggestion_result(
            "We could've waited.",
            AvoidContractions::default(),
            "We could have waited.",
        );
        assert_suggestion_result(
            "They couldn't've known.",
            AvoidContractions::default(),
            "They could not have known.",
        );
        assert_suggestion_result(
            "How're you doing?",
            AvoidContractions::default(),
            "How are you doing?",
        );
        assert_suggestion_result(
            "You mayn't agree.",
            AvoidContractions::default(),
            "You may not agree.",
        );
        assert_suggestion_result(
            "I might've helped.",
            AvoidContractions::default(),
            "I might have helped.",
        );
        assert_suggestion_result(
            "We must've missed it.",
            AvoidContractions::default(),
            "We must have missed it.",
        );
        assert_suggestion_result(
            "She oughtn't leave.",
            AvoidContractions::default(),
            "She ought not leave.",
        );
        assert_suggestion_result(
            "That'll work.",
            AvoidContractions::default(),
            "That will work.",
        );
        assert_suggestion_result(
            "What'll happen?",
            AvoidContractions::default(),
            "What will happen?",
        );
        assert_suggestion_result(
            "Who're they?",
            AvoidContractions::default(),
            "Who are they?",
        );
    }

    #[test]
    fn preserves_sentence_initial_case() {
        assert_suggestion_result(
            "Isn't this clear?",
            AvoidContractions::default(),
            "Is not this clear?",
        );
    }

    #[test]
    fn preserves_all_caps() {
        assert_suggestion_result(
            "WE DON'T AGREE.",
            AvoidContractions::default(),
            "WE DO NOT AGREE.",
        );
    }

    #[test]
    fn handles_typographic_apostrophes() {
        assert_suggestion_result(
            "They’re prepared.",
            AvoidContractions::default(),
            "They are prepared.",
        );
    }

    #[test]
    fn does_not_flag_possessives() {
        assert_no_lints("Alice's book is here.", AvoidContractions::default());
    }

    #[test]
    fn does_not_flag_names_with_apostrophes() {
        assert_no_lints("O'Connor arrived.", AvoidContractions::default());
    }

    #[test]
    fn does_not_flag_ambiguous_contractions() {
        assert_no_lints(
            "Ain't that odd? It's done. He'd go. She's been there. Who's ready? We'd left.",
            AvoidContractions::default(),
        );
    }

    #[test]
    fn does_not_flag_non_standard_or_lexicalized_contractions() {
        assert_no_lints(
            "We met y'all near the nor'easter exhibit at five o'clock.",
            AvoidContractions::default(),
        );
    }
}
