use crate::expr::Expr;
use crate::expr::MatchInfo;
use crate::expr::SequenceExpr;
use crate::expr::WordExprGroup;
use hashbrown::HashMap;

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::TokenStringExt;

pub struct DotInitialisms {
    expr: Box<dyn Expr>,
    corrections: HashMap<&'static str, &'static str>,
}

impl Default for DotInitialisms {
    fn default() -> Self {
        let mut patterns = WordExprGroup::default();

        let mut corrections = HashMap::new();
        corrections.insert("ie", "i.e.");
        corrections.insert("eg", "e.g.");

        for target in corrections.keys() {
            let pattern = SequenceExpr::default()
                .then_exact_word(target)
                .then_punctuation();

            patterns.add(target, pattern);
        }

        Self {
            expr: Box::new(patterns),
            corrections,
        }
    }
}

impl ExprLinter for DotInitialisms {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, match_info: MatchInfo<'_>, source: &[char]) -> Option<Lint> {
        let matched_tokens = match_info.matched_tokens;
        let found_word_tok = matched_tokens.first()?;
        let found_word = found_word_tok.span.get_content_string(source);

        let correction = self.corrections.get(found_word.as_str())?;

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Formatting,
            suggestions: vec![Suggestion::ReplaceWith(correction.chars().collect())],
            message: "Initialisms should have dot-separated letters.".to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Ensures common initialisms (like \"i.e.\") are properly dot-separated."
    }
}

#[cfg(test)]
mod tests {
    use super::DotInitialisms;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn matches_eg() {
        assert_suggestion_result(
            "Some text here (eg. more text).",
            DotInitialisms::default(),
            "Some text here (e.g. more text).",
        )
    }
}
