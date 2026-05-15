use super::{ExprLinter, Lint, LintKind};
use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::expr::SimilarToPhrase;
use crate::linting::Suggestion;
use crate::linting::expr_linter::Chunk;
use crate::patterns::Word;
use crate::{Token, TokenStringExt};

pub struct MapPhraseLinter {
    description: String,
    expr: Box<dyn Expr>,
    correct_forms: Vec<String>,
    message: String,
    lint_kind: LintKind,
}

impl MapPhraseLinter {
    pub fn new(
        expr: Box<dyn Expr>,
        correct_forms: impl IntoIterator<Item = impl ToString>,
        message: impl ToString,
        description: impl ToString,
        lint_kind: Option<LintKind>,
    ) -> Self {
        Self {
            description: description.to_string(),
            expr,
            correct_forms: correct_forms.into_iter().map(|f| f.to_string()).collect(),
            message: message.to_string(),
            lint_kind: lint_kind.unwrap_or(LintKind::Miscellaneous),
        }
    }

    pub fn new_similar_to_phrase(phrase: &'static str, detectable_distance: u8) -> Self {
        Self::new(
            Box::new(SimilarToPhrase::from_phrase(phrase, detectable_distance)),
            [phrase],
            format!("Did you mean the phrase `{phrase}`?"),
            format!("Looks for slight improper modifications to the phrase `{phrase}`."),
            None,
        )
    }

    pub fn new_fixed_phrase<'a>(
        parts: impl AsRef<[&'a str]>,
        compound: impl ToString,
        message: impl ToString,
        description: impl ToString,
        lint_kind: Option<LintKind>,
    ) -> Self {
        let words = parts.as_ref();
        let expr = words
            .iter()
            .enumerate()
            .fold(SequenceExpr::default(), |mut expr, (i, word)| {
                // Add the word with any capitalization using Word::new
                expr = expr.then(Word::new(word));

                // Add whitespace or hyphen between words, but not after the last word
                if i < words.len() - 1 {
                    expr = expr.then_whitespace_or_hyphen();
                }

                expr
            });

        Self::new(
            Box::new(expr),
            [compound.to_string()],
            message,
            description,
            lint_kind,
        )
    }

    pub fn new_closed_compound<'a>(phrase: impl AsRef<[&'a str]>, compound: impl ToString) -> Self {
        let message = format!(
            "Did you mean the closed compound `{}`?",
            compound.to_string()
        );

        let description = format!(
            "Looks for incorrect spacing inside the closed compound `{}`.",
            compound.to_string()
        );

        Self::new_fixed_phrase(
            phrase,
            compound,
            message,
            description,
            Some(LintKind::Miscellaneous),
        )
    }
}

impl ExprLinter for MapPhraseLinter {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let matched_text = span.get_content(source);

        Some(Lint {
            span,
            lint_kind: self.lint_kind,
            suggestions: self
                .correct_forms
                .iter()
                .map(|correct_form| {
                    Suggestion::replace_with_match_case(
                        correct_form.chars().collect(),
                        matched_text,
                    )
                })
                .collect(),
            message: self.message.to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        self.description.as_str()
    }
}
