use crate::linting::expr_linter::Chunk;
use crate::{
    Token,
    expr::{Expr, FirstMatchOf, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

/// Corrects the typo "thing" for "think".
pub struct ThingThink {
    expr: Box<dyn Expr>,
}

impl Default for ThingThink {
    fn default() -> Self {
        let subject_pronouns = WordSet::new(&["I", "you", "we", "they"]);
        let indefinite_pronouns = FirstMatchOf::new(vec![
            Box::new(WordSet::new(&[
                "anybody",
                "anyone",
                "everybody",
                "everyone",
            ])),
            // "Any one thing", "every one thing", "any body thing" cause false positives.
            Box::new(FixedPhrase::from_phrase("every body")),
        ]);
        let pronoun = FirstMatchOf::new(vec![
            Box::new(subject_pronouns),
            Box::new(indefinite_pronouns),
        ]);

        let verb_to = SequenceExpr::word_set(&[
            "have", "had", "has", "having", "need", "needed", "needs", "needing", "want", "wanted",
            "wants", "wanting", "try", "tried", "tries", "trying",
        ])
        .t_ws()
        .t_aco("to");

        let modal = WordSet::new(&[
            "can",
            "cannot",
            "can't",
            "could",
            "couldn't",
            "may",
            "might",
            "mightn't",
            "must",
            "mustn't",
            "shall",
            "shan't",
            "should",
            "shouldn't",
            "will",
            "won't",
        ]);

        let adverb_of_frequency =
            WordSet::new(&["always", "sometimes", "often", "usually", "never"]);

        let pre_context = FirstMatchOf::new(vec![
            Box::new(pronoun),
            Box::new(verb_to),
            Box::new(modal),
            Box::new(adverb_of_frequency),
        ]);

        let pattern = SequenceExpr::with(pre_context).t_ws().t_aco("thing");

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for ThingThink {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let thing_span = toks.last()?.span;

        Some(Lint {
            span: thing_span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case(
                ['t', 'h', 'i', 'n', 'k'].to_vec(),
                thing_span.get_content(src),
            )],
            message: "Did you mean `think`?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects the typo `thing` when it should be `think`."
    }
}

#[cfg(test)]
mod tests {
    use super::ThingThink;
    use crate::linting::tests::assert_suggestion_result;

    // Pronouns

    #[test]
    fn fix_you_thing() {
        assert_suggestion_result(
            "Whad do you thing about tinygo?",
            ThingThink::default(),
            "Whad do you think about tinygo?",
        );
    }

    #[test]
    fn fix_i_thing() {
        assert_suggestion_result(
            "bcz i thing hugging face embeddings and models are very complex",
            ThingThink::default(),
            "bcz i think hugging face embeddings and models are very complex",
        );
    }

    #[test]
    fn fix_we_thing() {
        assert_suggestion_result(
            "which information we thing to be missing",
            ThingThink::default(),
            "which information we think to be missing",
        );
    }

    #[test]
    fn fix_they_thing() {
        assert_suggestion_result(
            "they thing something is a good idea",
            ThingThink::default(),
            "they think something is a good idea",
        );
    }

    #[test]
    fn fix_everyone_thing() {
        assert_suggestion_result(
            "What does everyone thing here?",
            ThingThink::default(),
            "What does everyone think here?",
        );
    }

    #[test]
    fn fix_anyone_thing() {
        assert_suggestion_result(
            "Can anyone thing of a (reasonable) way to align them such that the 'a's in all 4 words will be in (more or less) the same vertical position?",
            ThingThink::default(),
            "Can anyone think of a (reasonable) way to align them such that the 'a's in all 4 words will be in (more or less) the same vertical position?",
        );
    }

    #[test]
    fn fix_anybody_thing() {
        assert_suggestion_result(
            "If anybody thing there is an issue in Karma, please re-open.",
            ThingThink::default(),
            "If anybody think there is an issue in Karma, please re-open.",
        );
    }

    #[test]
    fn fix_every_body_thing() {
        assert_suggestion_result(
            "What does every body thing I should do with my Randy Johnson rookie card.",
            ThingThink::default(),
            "What does every body think I should do with my Randy Johnson rookie card.",
        );
    }

    // Verb to

    #[test]
    fn fix_have_to_thing() {
        assert_suggestion_result(
            "I always have to thing what button does what action.",
            ThingThink::default(),
            "I always have to think what button does what action.",
        );
    }

    #[test]
    fn fix_need_to_thing() {
        assert_suggestion_result(
            "No need to thing about the REGEX.",
            ThingThink::default(),
            "No need to think about the REGEX.",
        );
    }

    #[test]
    fn fix_want_to_thing() {
        assert_suggestion_result(
            "maybe you want to thing of this also as a feature enhancement.",
            ThingThink::default(),
            "maybe you want to think of this also as a feature enhancement.",
        );
    }

    #[test]
    fn fix_having_to_thing() {
        assert_suggestion_result(
            "it has saved me personally hours in combined time not having to thing about whether something is in seconds or milliseconds",
            ThingThink::default(),
            "it has saved me personally hours in combined time not having to think about whether something is in seconds or milliseconds",
        );
    }

    #[test]
    fn fix_needs_to() {
        assert_suggestion_result(
            "When implementing any functionality once needs to thing aboiut how it is going to be used.",
            ThingThink::default(),
            "When implementing any functionality once needs to think aboiut how it is going to be used.",
        );
    }

    #[test]
    fn fix_needed_to() {
        assert_suggestion_result(
            "Even in that case we needed to thing about the syntax so that we wouldn't need to change existing syntax",
            ThingThink::default(),
            "Even in that case we needed to think about the syntax so that we wouldn't need to change existing syntax",
        );
    }

    #[test]
    fn fix_had_to() {
        assert_suggestion_result(
            "I had to thing in ways of making people more interested in it",
            ThingThink::default(),
            "I had to think in ways of making people more interested in it",
        );
    }

    #[test]
    fn fix_trying_to_thing() {
        assert_suggestion_result(
            "Here I'm trying to thing about the following questions:",
            ThingThink::default(),
            "Here I'm trying to think about the following questions:",
        );
    }

    // Modal verbs

    #[test]
    fn fix_can_thing() {
        assert_suggestion_result(
            "The exe file dosen't work allways, because antivirus can thing it is a virus.",
            ThingThink::default(),
            "The exe file dosen't work allways, because antivirus can think it is a virus.",
        );
    }

    #[test]
    fn fix_could_thing() {
        assert_suggestion_result(
            "\"doesNotReturnSameInstanceWhenCalledMultipleTimes\" is a terrible name, but the only one i could thing of immediately.",
            ThingThink::default(),
            "\"doesNotReturnSameInstanceWhenCalledMultipleTimes\" is a terrible name, but the only one i could think of immediately.",
        );
    }

    #[test]
    fn fix_might_thing() {
        assert_suggestion_result(
            "Consider what a reader might thing when reading a switch",
            ThingThink::default(),
            "Consider what a reader might think when reading a switch",
        );
    }

    #[test]
    fn fix_should_thing() {
        assert_suggestion_result(
            "And we should thing to add a flag so the user could decide if internal top level extension functions are ok or not.",
            ThingThink::default(),
            "And we should think to add a flag so the user could decide if internal top level extension functions are ok or not.",
        );
    }

    #[test]
    fn fix_may_thing() {
        assert_suggestion_result(
            "It is easier than you may thing to run both bands with hostapd.",
            ThingThink::default(),
            "It is easier than you may think to run both bands with hostapd.",
        );
    }

    #[test]
    fn fix_cannot_thing() {
        assert_suggestion_result(
            "I cannot thing of a simple way to implement compensation of a change in Fnco.",
            ThingThink::default(),
            "I cannot think of a simple way to implement compensation of a change in Fnco.",
        );
    }

    #[test]
    fn fix_will_thing() {
        assert_suggestion_result(
            "So user will thing that delete operation is fine but its not this code deletes the wrong page and make one extra page which wrong.",
            ThingThink::default(),
            "So user will think that delete operation is fine but its not this code deletes the wrong page and make one extra page which wrong.",
        );
    }

    #[test]
    fn fix_cant_thing() {
        assert_suggestion_result(
            "can't thing of another place, which could have such effect",
            ThingThink::default(),
            "can't think of another place, which could have such effect",
        );
    }

    #[test]
    fn fix_couldnt_thing() {
        assert_suggestion_result(
            "I couldn't thing about a better title, but I run into problems since the new dplyr release.",
            ThingThink::default(),
            "I couldn't think about a better title, but I run into problems since the new dplyr release.",
        );
    }

    #[test]
    fn fix_shouldnt_thing() {
        assert_suggestion_result(
            "When dealing with a multi-tenanted system, users shouldn't thing about 'Databases', they should think about Tenants.",
            ThingThink::default(),
            "When dealing with a multi-tenanted system, users shouldn't think about 'Databases', they should think about Tenants.",
        );
    }

    #[test]
    fn fix_wont_thing() {
        assert_suggestion_result(
            "I think you need to use an io.Pipe so the Go HTTP Request won't thing the buf has been fulling read.",
            ThingThink::default(),
            "I think you need to use an io.Pipe so the Go HTTP Request won't think the buf has been fulling read.",
        );
    }

    // Adverb of frequency

    #[test]
    fn fix_always_thing() {
        assert_suggestion_result(
            "one should always thing of whether the efforts are better targeted to the improvement",
            ThingThink::default(),
            "one should always think of whether the efforts are better targeted to the improvement",
        );
    }

    #[test]
    fn fix_sometimes_thing() {
        assert_suggestion_result(
            "One thing that I sometimes thing would be nice is if I could make different instances",
            ThingThink::default(),
            "One thing that I sometimes think would be nice is if I could make different instances",
        );
    }

    #[test]
    fn fix_often_thing() {
        assert_suggestion_result(
            "When working with workflows on many forms I often thing I need to do the same over and over",
            ThingThink::default(),
            "When working with workflows on many forms I often think I need to do the same over and over",
        );
    }

    #[test]
    fn fix_never_thing() {
        assert_suggestion_result(
            "just use UUIDv7 and never thing about those details again",
            ThingThink::default(),
            "just use UUIDv7 and never think about those details again",
        );
    }

    #[test]
    fn fix_usually_thing() {
        assert_suggestion_result(
            "And the order of that relationship might be reversed from what one might usually thing.",
            ThingThink::default(),
            "And the order of that relationship might be reversed from what one might usually think.",
        );
    }
}
