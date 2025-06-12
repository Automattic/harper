use super::{Lint, LintKind, Linter};
use crate::CharStringExt;
use crate::Document;
use crate::TokenStringExt;

/// Detect incorrect usage of linking verbs.
///
/// This ensures that linking verbs are preceded by a noun or pronoun.
#[derive(Debug, Clone, Copy, Default)]
pub struct LinkingVerbs;

impl Linter for LinkingVerbs {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut output = Vec::new();

        for chunk in document.iter_chunks() {
            // The word prior to "is" must be a nominal (noun or pronoun).
            for idx in chunk.iter_linking_verb_indices() {
                let linking_verb = &chunk[idx];
                if let Some(prev_word) = &chunk[0..idx].last_word() {
                    if let Some(metadata) = prev_word.kind.as_word().unwrap() {
                        if !metadata.is_nominal() {
                            let linking_verb_text = document.get_span_content(&linking_verb.span);

                            output.push(Lint {
                                span: linking_verb.span,
                                lint_kind: LintKind::Miscellaneous,
                                message: format!(
                                    "Linking verbs like “{}” must be preceded by a noun or pronoun.",
                                    linking_verb_text.to_string()
                                ),
                                ..Default::default()
                            })
                        }
                    }
                }
            }
        }

        output
    }

    fn description(&self) -> &'static str {
        "Linking verbs connect nouns to other ideas. Make sure you do not accidentally link words that aren't nouns."
    }
}

#[cfg(test)]
mod tests {
    use super::LinkingVerbs;
    use crate::linting::tests::assert_lint_count;

    #[test]
    fn dora() {
        assert_lint_count("Dora is a noun.", LinkingVerbs, 0);
    }

    #[test]
    fn working_wrong() {
        assert_lint_count("working is not a noun.", LinkingVerbs, 1);
    }

    #[test]
    fn working_right() {
        assert_lint_count("\"working\" is a noun.", LinkingVerbs, 0);
    }
}
