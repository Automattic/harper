use itertools::Itertools;

use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar discordancia de género evidente en artículos y sustantivos comunes en español.
/// Ejemplos: "la problema" -> "el problema", "el agua fria" -> "el agua fría" / "el casa" -> "la casa".
#[derive(Debug, Default)]
pub struct SpanishGenderAgreement;

impl Linter for SpanishGenderAgreement {
    fn description(&self) -> &str {
        "Detecta discordancias de género evidentes entre artículos y sustantivos en español (ej: \"la problema\" por \"el problema\")."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        const MASCULINE_EXCEPTIONS_ENDING_IN_A: &[&str] = &[
            "problema", "sistema", "tema", "clima", "poema", "idioma", "mapa", "dia", "día",
            "planeta", "dilema", "plasma", "enigma", "fantasma", "teorema", "drama",
        ];

        const FEMININE_EXCEPTIONS_ENDING_IN_O: &[&str] = &["foto", "mano", "moto", "radio"];

        for chunk in document.iter_chunks() {
            for (first_idx, second_idx) in chunk.iter_word_indices().tuple_windows() {
                let first = &chunk[first_idx];
                let second = &chunk[second_idx];

                let first_str = document.get_span_content_str(&first.span).to_lowercase();
                let second_str = document.get_span_content_str(&second.span).to_lowercase();

                // Caso 1: "la" + Sustantivo masculino de origen griego terminado en "a" (ej: "la problema")
                if (first_str == "la" || first_str == "una")
                    && MASCULINE_EXCEPTIONS_ENDING_IN_A.contains(&second_str.as_str())
                {
                    let replacement = if first_str == "la" { "el" } else { "un" };
                    lints.push(Lint {
                        span: first.span,
                        lint_kind: LintKind::Grammar,
                        message: format!(
                            "En español, '{}' es un sustantivo masculino. Usa '{}' en su lugar.",
                            second_str, replacement
                        ),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            replacement.chars().collect(),
                            document.get_span_content(&first.span),
                        )],
                        priority: 31,
                    });
                }
                // Caso 2: "el" + Sustantivo femenino terminado en "o" (ej: "el foto", "el mano")
                else if (first_str == "el" || first_str == "un")
                    && FEMININE_EXCEPTIONS_ENDING_IN_O.contains(&second_str.as_str())
                {
                    let replacement = if first_str == "el" { "la" } else { "una" };
                    lints.push(Lint {
                        span: first.span,
                        lint_kind: LintKind::Grammar,
                        message: format!(
                            "En español, '{}' es un sustantivo femenino. Usa '{}' en su lugar.",
                            second_str, replacement
                        ),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            replacement.chars().collect(),
                            document.get_span_content(&first.span),
                        )],
                        priority: 31,
                    });
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
    fn corrects_la_problema() {
        let mut linter = SpanishGenderAgreement;
        let doc = Document::new_plain_english_curated("la problema");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        let mut text_chars: Vec<char> = "la problema".chars().collect();
        lints[0].suggestions[0].apply(lints[0].span, &mut text_chars);
        let result: String = text_chars.into_iter().collect();
        assert_eq!(result, "el problema");
    }

    #[test]
    fn corrects_el_foto() {
        let mut linter = SpanishGenderAgreement;
        let doc = Document::new_plain_english_curated("el foto");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        let mut text_chars: Vec<char> = "el foto".chars().collect();
        lints[0].suggestions[0].apply(lints[0].span, &mut text_chars);
        let result: String = text_chars.into_iter().collect();
        assert_eq!(result, "la foto");
    }
}
