use super::{Lint, LintKind, PatternLinter, Suggestion};
use crate::{
    patterns::{All, EitherPattern, Pattern, SequencePattern, Word}, Lrc, Token, TokenStringExt
};

pub struct Everyday {
    pattern: Box<dyn Pattern>,
}

impl Default for Everyday {
    fn default() -> Self {
        let everyday = Word::new("everyday");
        // let every_day = Lrc::new(SequencePattern::default()
        //     .t_aco("every")
        //     .t_ws()
        //     .t_aco("day"));

        let x_then_everyday = All::new(vec![
            Box::new(
                SequencePattern::default()
                    .then_any_word()
                    .t_ws()
                    .then(everyday.clone()),
            ),
            // context
            Box::new(
                SequencePattern::default()
                    .then_verb()
                    .then_anything()
                    .then_anything()
            ),
        ]);

        // context for adjective everyday
        // ... noun $       "They are everyday tasks."
        // ❌... noun       "Everyday tasks are boring." vs "Every day tasks get completed."
        // poss. ...        "My everyday tasks ..."
        // ... pres.part    "Everyday coding projects"
        // pres.part ... nom    "For manipulating everday objects"

        // context for adverb every day
        // nom. ...         "Do it every day." "Firewall bricks itself every day"
        // ... $            "Write code every day."
        // ... ","          "Every day, a new concept..." ❌ commas delimit chunks, PatternLinter can't see them
        // pst. ...         "Git commands used every day", "... what I learned every day"
        // "almost" ...     "Almost every day"
        // ... prep.        "Map every day with OSM"
        // ... conj.        "Pick a different test item every day and confirm it is present"

        Self {
            pattern: Box::new(EitherPattern::new(vec![
                Box::new(x_then_everyday),
            ])),
        }
    }
}

impl PatternLinter for Everyday {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let wrong_word_span = matched_tokens[2].span;
        let template = wrong_word_span.get_content(source);
        let suggestion = Suggestion::replace_with_match_case_str("every day", template);
        let suggestions = vec![suggestion];
        let message =
            "You probably mean the adverb `every day` here.".to_owned();
        
        Some(Lint {
            span: wrong_word_span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message,
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "This rule tries to sort out confusing the adjective `everyday` and the adverb `every day`."
    }
}

#[cfg(test)]
mod tests {
    use super::Everyday;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn dont_flag_lone_adjective() {
        assert_lint_count("everyday", Everyday::default(), 0);
    }

    #[test]
    fn dont_flag_lone_adverb() {
        assert_lint_count("every day", Everyday::default(), 0);
    }

    #[test]
    fn correct_after_past_verb() {
        assert_suggestion_result(
            "Some git commands used everyday",
            Everyday::default(),
            "Some git commands used every day",
        );
    }
}
