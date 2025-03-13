use itertools::Itertools;

use super::{Lint, LintKind, Linter, Suggestion};
use crate::TokenStringExt;

/// A linter that fixes common comma errors such as no space after, erroneous
///  space before, etc, Asian commas instead of English commas, etc.
#[derive(Debug, Default)]
pub struct CommaFixes;

impl Linter for CommaFixes {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let mut lints: Vec<Lint> = Vec::new();

        for tok in document.iter_commas() {
            let tok_content = document.get_span_content(tok.span);

            if tok_content.is_empty() || tok_content.first().cloned() == Some(',') {
                continue;
            }

            lints.push(Lint {
                span: tok.span,
                lint_kind: LintKind::Formatting,
                suggestions: vec![Suggestion::ReplaceWith(vec![','])],
                message: "Avoid East Asian commas in English contexts.".to_string(),
                priority: 32,
            })
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Fix common comma errors such as no space after, erroneous space before, etc, Asian commas instead of English commas, etc."
    }
}

#[cfg(test)]
mod tests {
    use super::CommaFixes;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn allows_english_comma_atomic() {
        assert_lint_count(",", CommaFixes, 0);
    }

    #[test]
    fn flags_fullwidth_comma_atomic() {
        assert_lint_count("，", CommaFixes, 1);
    }

    #[test]
    fn flags_ideographic_comma_atomic() {
        assert_lint_count("、", CommaFixes, 1);
    }

    #[test]
    fn corrects_fullwidth_comma_real_world() {
        assert_suggestion_result(
            "higher 2 bits of the number of nodes， whether abandoned or not decided by .index section",
            CommaFixes,
            "higher 2 bits of the number of nodes, whether abandoned or not decided by .index section",
        );
    }

    #[test]
    fn corrects_ideographic_comma_real_world() {
        assert_suggestion_result("cout、endl、string", CommaFixes, "cout,endl,string")
    }
}
