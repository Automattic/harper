use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para garantizar que toda oración en español comience con mayúscula inicial.
/// Ej. "ortografía. casa de la madera" -> "ortografía. Casa de la madera".
#[derive(Debug, Default)]
pub struct SpanishSentenceCapitalization;

impl Linter for SpanishSentenceCapitalization {
    fn description(&self) -> &str {
        "Garantiza que la primera palabra de cada oración en español comience con mayúscula inicial tras un punto o al inicio del texto."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence in paragraph.iter_sentences() {
                if let Some(first_word) = sentence.first_non_whitespace() {
                    if !first_word.kind.is_word() {
                        continue;
                    }

                    let word_chars = document.get_span_content(&first_word.span);
                    if let Some(first_char) = word_chars.first() {
                        if first_char.is_alphabetic() && first_char.is_lowercase() {
                            let mut cap_chars = word_chars.to_vec();
                            if let Some(c) = cap_chars.first_mut() {
                                *c = c.to_uppercase().next().unwrap_or(*c);
                            }
                            let capitalized_str: String = cap_chars.into_iter().collect();

                            lints.push(Lint {
                                span: first_word.span,
                                lint_kind: LintKind::Capitalization,
                                message: format!(
                                    "La primera palabra de una oración debe comenzar con mayúscula inicial ('{}').",
                                    capitalized_str
                                ),
                                suggestions: vec![Suggestion::ReplaceWith(
                                    capitalized_str.chars().collect(),
                                )],
                                priority: 10,
                            });
                        }
                    }
                }
            }
        }

        lints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrects_lowercase_sentence_start() {
        let mut linter = SpanishSentenceCapitalization;
        let doc = Document::new_plain_english_curated("hola. casa de madera.");
        let lints = linter.lint(&doc);
        assert!(!lints.is_empty());
        assert!(lints[0].message.contains("mayúscula"));
    }
}
