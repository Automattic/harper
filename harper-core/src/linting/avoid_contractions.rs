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
                "isn't",
                "aren't",
                "wasn't",
                "weren't",
                "don't",
                "doesn't",
                "didn't",
                "can't",
                "couldn't",
                "shouldn't",
                "wouldn't",
                "won't",
                "haven't",
                "hasn't",
                "hadn't",
                "mustn't",
                "needn't",
                "I'm",
                "you're",
                "we're",
                "they're",
                "I've",
                "you've",
                "we've",
                "they've",
                "I'll",
                "you'll",
                "we'll",
                "they'll",
                "he'll",
                "she'll",
                "it'll",
            ]),
        }
    }
}

impl AvoidContractions {
    fn expansion(contraction: &str) -> Option<&'static str> {
        match contraction {
            "isn't" => Some("is not"),
            "aren't" => Some("are not"),
            "wasn't" => Some("was not"),
            "weren't" => Some("were not"),
            "don't" => Some("do not"),
            "doesn't" => Some("does not"),
            "didn't" => Some("did not"),
            "can't" => Some("cannot"),
            "couldn't" => Some("could not"),
            "shouldn't" => Some("should not"),
            "wouldn't" => Some("would not"),
            "won't" => Some("will not"),
            "haven't" => Some("have not"),
            "hasn't" => Some("has not"),
            "hadn't" => Some("had not"),
            "mustn't" => Some("must not"),
            "needn't" => Some("need not"),
            "i'm" => Some("i am"),
            "you're" => Some("you are"),
            "we're" => Some("we are"),
            "they're" => Some("they are"),
            "i've" => Some("i have"),
            "you've" => Some("you have"),
            "we've" => Some("we have"),
            "they've" => Some("they have"),
            "i'll" => Some("i will"),
            "you'll" => Some("you will"),
            "we'll" => Some("we will"),
            "they'll" => Some("they will"),
            "he'll" => Some("he will"),
            "she'll" => Some("she will"),
            "it'll" => Some("it will"),
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
            "It's done. He'd go. She's been there.",
            AvoidContractions::default(),
        );
    }
}
