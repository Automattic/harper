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
    multi_wrong_forms_to_multi_correct_forms: &'a [(&'a [&'a str], &'a [&'a str])],
    message: String,
    lint_kind: LintKind,
}

impl<'a> MapPhraseSetLinter<'a> {
    pub fn one_to_one(
        wrong_forms_to_correct_forms: &'a [(&'a str, &'a str)],
        message: impl ToString,
        description: impl ToString,
        lint_kind: Option<LintKind>,
    ) -> Self {
        let expr = Box::new(LongestMatchOf::new(
            wrong_forms_to_correct_forms
                .iter()
                .map(|(wrong_form, _correct_form)| {
                    let expr: Box<dyn Expr> = Box::new(FixedPhrase::from_phrase(wrong_form));
                    expr
                })
                .collect(),
        ));

        Self {
            description: description.to_string(),
            expr,
            wrong_forms_to_correct_forms,
            multi_wrong_forms_to_multi_correct_forms: &[],
            message: message.to_string(),
            lint_kind: lint_kind.unwrap_or(LintKind::Miscellaneous),
        }
    }

    pub fn many_to_many(
        multi_wrong_forms_to_multi_correct_forms: &'a [(&'a [&'a str], &'a [&'a str])],
        message: impl ToString,
        description: impl ToString,
        lint_kind: Option<LintKind>,
    ) -> Self {
        let mut lmo = LongestMatchOf::new(Vec::new());
        for (wrong_forms, _correct_forms) in multi_wrong_forms_to_multi_correct_forms {
            for wrong_form in wrong_forms.iter() {
                lmo.add(FixedPhrase::from_phrase(wrong_form));
            }
        }
        let expr = Box::new(lmo);

        Self {
            description: description.to_string(),
            expr,
            wrong_forms_to_correct_forms: &[],
            multi_wrong_forms_to_multi_correct_forms,
            message: message.to_string(),
            lint_kind: lint_kind.unwrap_or(LintKind::Miscellaneous),
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

        let mut suggestions: Vec<_> = self
            .wrong_forms_to_correct_forms
            .iter()
            .filter(|(wrong_form, _)| matched_text.eq_ignore_ascii_case_str(wrong_form))
            .map(|(_, correct_form)| {
                Suggestion::replace_with_match_case(correct_form.chars().collect(), matched_text)
            })
            .collect();

        let many_to_many_suggestions: Vec<_> = self
            .multi_wrong_forms_to_multi_correct_forms
            .iter()
            .flat_map(|(wrong_forms, correct_forms)| {
                wrong_forms
                    .iter()
                    .filter(move |&&wrong_form| matched_text.eq_ignore_ascii_case_str(wrong_form))
                    .flat_map(move |_| {
                        correct_forms.iter().map(move |correct_form| {
                            Suggestion::replace_with_match_case(
                                correct_form.chars().collect(),
                                matched_text,
                            )
                        })
                    })
            })
            .collect();

        suggestions.extend(many_to_many_suggestions);

        if suggestions.is_empty() {
            return None;
        }

        Some(Lint {
            span,
            lint_kind: self.lint_kind,
            suggestions,
            message: self.message.to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        self.description.as_str()
    }
}
