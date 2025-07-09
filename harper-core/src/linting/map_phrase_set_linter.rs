use super::{ExprLinter, Lint, LintKind};
use crate::CharStringExt;
use crate::expr::Expr;
use crate::expr::FixedPhrase;
use crate::expr::LongestMatchOf;
use crate::linting::Suggestion;
use crate::{Token, TokenStringExt};

pub struct MapPhraseSetLinter<'a> {
    description: String,
    expr: Box<dyn Expr>,
    wrong_forms_to_correct_forms: &'a [(&'a str, &'a str)],
    message: String,
}

impl<'a> MapPhraseSetLinter<'a> {
    pub fn new(
        wrong_forms_to_correct_forms: &'a [(&'a str, &'a str)],
        message: impl ToString,
        description: impl ToString,
    ) -> Self {
        let expr = Box::new(LongestMatchOf::new(
            wrong_forms_to_correct_forms
                .iter()
                .map(|(wrong_form, _correct_form)| {
                    let expr: Box<dyn Expr> =
                        Box::new(FixedPhrase::from_phrase(wrong_form.as_ref()));
                    expr
                })
                .collect(),
        ));

        Self {
            description: description.to_string(),
            expr,
            wrong_forms_to_correct_forms,
            message: message.to_string(),
        }
    }
}

impl<'a> ExprLinter for MapPhraseSetLinter<'a> {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.span()?;
        let matched_text = span.get_content(source);

        let mut suggestions = Vec::new();

        for (wrong_form, correct_form) in self.wrong_forms_to_correct_forms {
            if matched_text.eq_ignore_ascii_case_str(wrong_form) {
                suggestions.push(Suggestion::replace_with_match_case(
                    correct_form.chars().collect(),
                    matched_text,
                ));
            }
        }

        if suggestions.is_empty() {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions,
            message: self.message.to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        self.description.as_str()
    }
}
