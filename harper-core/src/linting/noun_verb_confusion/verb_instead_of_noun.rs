use crate::{
    Lrc, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

use super::NOUN_VERB_PAIRS;

pub struct VerbInsteadOfNoun {
    expr: Box<dyn Expr>,
}

impl Default for VerbInsteadOfNoun {
    fn default() -> Self {
        let verbs = Lrc::new(WordSet::new(
            &NOUN_VERB_PAIRS
                .iter()
                .map(|&(_, verb)| verb)
                .collect::<Vec<_>>(),
        ));
        Self {
            expr: Box::new(
                SequenceExpr::default()
                    .then_adjective()
                    .then_whitespace()
                    .then(verbs.clone()),
            ),
        }
    }
}

impl ExprLinter for VerbInsteadOfNoun {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let adj_tok = &toks.first()?;
        let verb_tok = &toks.last()?;

        let adj_text = adj_tok.span.get_content_string(src);
        let verb_text = verb_tok.span.get_content_string(src);
        let verb_lower = verb_text.to_lowercase();

        let noun = NOUN_VERB_PAIRS
            .iter()
            .find(|(_, verb)| *verb == verb_lower)
            .map(|(noun, _)| noun)?;

        // Don't flag "so I better advise you", "you'd better believe this", "you'd best listen to me".
        if adj_text == "better" || adj_text == "best" {
            return None;
        }

        // "Sound" is both adjectve and noun. We want to flag the common "sound advise"
        // But not "sound affect", which is just as correct as "sound effect".
        if adj_text == "sound" && verb_text == "affect" {
            return None;
        }

        Some(Lint {
            span: verb_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                noun.chars().collect(),
                verb_tok.span.get_content(src),
            )],
            message: format!("`{verb_text}` is a verb, the noun should be `{noun}`."),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects verbs used instead of nouns when the two are related."
    }
}

#[cfg(test)]
mod tests {
    use super::VerbInsteadOfNoun;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_good_advise() {
        assert_suggestion_result("Good advise", VerbInsteadOfNoun::default(), "Good advice");
    }

    #[test]
    fn corrects_bad_advise() {
        assert_suggestion_result(
            "I just wanted to bring attention to this because it stood out to me as potentially bad advise.",
            VerbInsteadOfNoun::default(),
            "I just wanted to bring attention to this because it stood out to me as potentially bad advice.",
        );
    }

    #[test]
    fn dont_flag_correct_better_advise() {
        assert_lint_count(
            "Hello! I am an engineer at Plexon and am conducting tests with Kilosort4 so we can better advise our clients.",
            VerbInsteadOfNoun::default(),
            0,
        );
    }

    #[test]
    #[ignore = "'better advise' can be correct as above, or a mistake like here"]
    fn correct_better_advise() {
        assert_suggestion_result(
            "Maybe this will be a decent idea, .or anybody has better advise :)",
            VerbInsteadOfNoun::default(),
            "Maybe this will be a decent idea, .or anybody has better advice :)",
        );
    }

    #[test]
    fn dont_flag_correct_better_believe() {
        assert_lint_count(
            "You'd better believe this is bbedit-gist-maker.",
            VerbInsteadOfNoun::default(),
            0,
        );
    }

    #[test]
    fn correct_great_affect() {
        assert_suggestion_result(
            "badges that they provide to users to allow them to promote their projects to great affect",
            VerbInsteadOfNoun::default(),
            "badges that they provide to users to allow them to promote their projects to great effect",
        );
    }

    #[test]
    fn correct_strong_believe() {
        assert_suggestion_result(
            "cause my strong believe is that we must give any user to describe whether a post is meant factual",
            VerbInsteadOfNoun::default(),
            "cause my strong belief is that we must give any user to describe whether a post is meant factual",
        );
    }

    #[test]
    fn dont_flag_best_affect() {
        assert_lint_count(
            "Using linear regression to predict and understand what factors best affect house price",
            VerbInsteadOfNoun::default(),
            0,
        );
    }

    #[test]
    fn correct_deep_breathe() {
        assert_suggestion_result(
            "Take deep breathe and Do it again!",
            VerbInsteadOfNoun::default(),
            "Take deep breath and Do it again!",
        );
    }

    #[test]
    fn dont_flag_sound_affect() {
        assert_lint_count(
            "The goal of this study was to learn what properties of sound affect human focus the most.",
            VerbInsteadOfNoun::default(),
            0,
        );
    }

    #[test]
    #[ignore = "`Affect` is both verb and noun, so we can't disambiguate without more context"]
    fn correct_sound_affect() {
        assert_suggestion_result(
            "Diesel Generator's animation returns to 'idle' state, but it's sound affect remains in the 'work' state.",
            VerbInsteadOfNoun::default(),
            "Diesel Generator's animation returns to 'idle' state, but it's sound effect remains in the 'work' state.",
        );
    }

    #[test]
    fn correct_bad_intend() {
        assert_suggestion_result(
            "What do you do if you only see slightly longer posts that may still be acceptable (and not bad intend from the poster)",
            VerbInsteadOfNoun::default(),
            "What do you do if you only see slightly longer posts that may still be acceptable (and not bad intent from the poster)",
        );
    }
}
