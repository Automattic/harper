use crate::{
    Token, TokenStringExt,
    patterns::{EitherPattern, Pattern, SequencePattern},
};

use super::{Lint, LintKind, PatternLinter, Suggestion};

const EN_DASH: char = '–';
const EM_DASH: char = '—';

pub struct Dashes {
    pattern: Box<dyn Pattern>,
}

impl Default for Dashes {
    fn default() -> Self {
        let en_dash = SequencePattern::default().then_hyphen().then_hyphen();
        let em_dash_or_longer = SequencePattern::default()
            .then_hyphen()
            .then_hyphen()
            .then_one_or_more_hyphens();

        let pattern = EitherPattern::new(vec![Box::new(em_dash_or_longer), Box::new(en_dash)]);

        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl PatternLinter for Dashes {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], _source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let lint_kind = LintKind::Formatting;

        match matched_tokens.len() {
            2 => Some(Lint {
                span,
                lint_kind,
                suggestions: vec![Suggestion::ReplaceWith(vec![EN_DASH])],
                message: "A sequence of hyphens is not an en dash.".to_owned(),
                priority: 63,
            }),
            3 => Some(Lint {
                span,
                lint_kind,
                suggestions: vec![Suggestion::ReplaceWith(vec![EM_DASH])],
                message: "A sequence of hyphens is not an em dash.".to_owned(),
                priority: 63,
            }),
            4.. => None, // Ignore longer hyphen sequences.
            _ => panic!("Received unexpected number of tokens."),
        }
    }

    fn description(&self) -> &'static str {
        "Rather than outright using an em dash or en dash, authors often use a sequence of hyphens, expecting them to be condensed. Use two hyphens to denote an en dash and three to denote an em dash."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_suggestion_count, assert_suggestion_result};

    use super::Dashes;
    use super::{EM_DASH, EN_DASH};

    #[test]
    fn catches_en_dash() {
        assert_suggestion_result(
            "pre--Industrial Revolution",
            Dashes::default(),
            &format!("pre{EN_DASH}Industrial Revolution"),
        );
    }

    #[test]
    fn catches_em_dash() {
        assert_suggestion_result(
            "'There is no box' --- Scott",
            Dashes::default(),
            &format!("'There is no box' {EM_DASH} Scott"),
        );
    }

    #[test]
    fn no_overlaps() {
        assert_suggestion_count("'There is no box' --- Scott", Dashes::default(), 1);
    }

    #[test]
    fn no_lint_for_long_hyphen_sequences() {
        assert_suggestion_count("'There is no box' ------ Scott", Dashes::default(), 0);
    }
}
