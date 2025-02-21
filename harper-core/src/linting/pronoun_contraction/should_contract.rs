use crate::{
    CharStringExt, Token,
    patterns::{Pattern, SequencePattern, WordSet},
};

use crate::Lint;
use crate::linting::{LintKind, PatternLinter, Suggestion};

pub struct ShouldContract {
    pattern: Box<dyn Pattern>,
}

impl Default for ShouldContract {
    fn default() -> Self {
        Self {
            pattern: Box::new(
                SequencePattern::default()
                    .then(WordSet::new(&["your", "were"]))
                    .then_whitespace()
                    .t_aco("the")
                    .then_whitespace()
                    .then_nominal(),
            ),
        }
    }
}

impl ShouldContract {
    fn mistake_to_correct(mistake: &str) -> impl Iterator<Item = Vec<char>> {
        match mistake.to_lowercase().as_str() {
            "your" => vec!["you're", "you are"],
            "were" => vec!["we're", "we are"],
            _ => panic!("The pattern in this linter should make a fall-through impossible."),
        }
        .into_iter()
        .map(|v| v.chars().collect())
    }
}

impl PatternLinter for ShouldContract {
    fn pattern(&self) -> &dyn Pattern {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let mistake = matched_tokens[0].span.get_content(source);

        Some(Lint {
            span: matched_tokens[0].span,
            lint_kind: LintKind::WordChoice,
            suggestions: Self::mistake_to_correct(&mistake.to_lower().to_string())
                .map(|v| Suggestion::replace_with_match_case(v, mistake))
                .collect(),
            message: "Use the contraction or separate the words instead.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Neglecting the apostrophe when contracting pronouns with \"are\" (like \"your\" and \"you are\") is a fatal, but extremely common mistake to make."
    }
}
