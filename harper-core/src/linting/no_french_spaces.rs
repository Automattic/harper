use super::{Lint, LintKind, Linter, Suggestion};
use crate::Document;
use crate::TokenStringExt;

#[derive(Debug, Default)]
pub struct NoFrenchSpaces;

impl Linter for NoFrenchSpaces {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut output = Vec::new();

        for sentence in document.iter_sentences() {
            if let Some(space_idx) = sentence.iter_space_indices().next() {
                let space = &sentence[space_idx];

                if space_idx == 0 && space.span.len() != 1 {
                    output.push(Lint {
                        span: space.span,
                        lint_kind: LintKind::Formatting,
                        suggestions: vec![Suggestion::ReplaceWith(vec![' '])],
                        message: "French spaces are generally not recommended.".to_owned(),
                        priority: 15,
                    })
                }
            }
        }

        output
    }

    fn description(&self) -> &str {
        "Stops users from accidentally inserting French spaces."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::NoFrenchSpaces;

    #[test]
    fn fixes_basic() {
        assert_suggestion_result(
            "This is a short sentence.  This is another short sentence.",
            NoFrenchSpaces::default(),
            "This is a short sentence. This is another short sentence.",
        );
    }
}
