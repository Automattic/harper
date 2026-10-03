use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar discordancia de número y género en frases nominales compuestas en español.
/// Ejemplo:
/// - "las rápidas y efectivas soluciones" -> VÁLIDA (concordancia entre determinante plural, adjetivos y sustantivo plural).
/// - "las rápida soluciones" -> "las rápidas soluciones"
/// - "los soluciones" -> "las soluciones"
#[derive(Debug, Default)]
pub struct SpanishPluralAgreement;

impl Linter for SpanishPluralAgreement {
    fn description(&self) -> &str {
        "Detecta discordancias de plural y género en frases nominales compuestas con adjetivos coordinados o modificadores en español."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        const PLURAL_DET_MASC: &[&str] = &["los", "unos", "estos", "esos", "aquellos", "nuestros"];
        const PLURAL_DET_FEM: &[&str] = &["las", "unas", "estas", "esas", "aquellas", "nuestras"];

        const SINGULAR_DET_MASC: &[&str] = &["el", "un", "este", "ese", "aquel", "nuestro"];
        const SINGULAR_DET_FEM: &[&str] = &["la", "una", "esta", "esa", "aquella", "nuestra"];

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            // Patrón: Determinante Plural + Adjetivo Singular + Sustantivo Plural
            // Ejemplo: "las rápida soluciones" -> "las rápidas soluciones"
            for window in word_indices.windows(3) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];
                let third_tok = &chunk[window[2]];

                let det_str = document
                    .get_span_content_str(&first_tok.span)
                    .to_lowercase();
                let adj_str = document
                    .get_span_content_str(&second_tok.span)
                    .to_lowercase();
                let noun_str = document
                    .get_span_content_str(&third_tok.span)
                    .to_lowercase();

                let is_plural_det = PLURAL_DET_MASC.contains(&det_str.as_str())
                    || PLURAL_DET_FEM.contains(&det_str.as_str());

                let noun_is_plural = noun_str.ends_with('s') || noun_str.ends_with("es");
                let adj_is_singular = !adj_str.ends_with('s') && adj_str.len() > 3;

                // Evitar conjunciones, preposiciones o palabras clave en posición de adjetivo
                const EXCLUDED_WORDS: &[&str] = &[
                    "y", "o", "e", "u", "con", "sin", "para", "por", "de", "del", "que", "como",
                    "muy", "mas", "más",
                ];

                if is_plural_det
                    && noun_is_plural
                    && adj_is_singular
                    && !EXCLUDED_WORDS.contains(&adj_str.as_str())
                {
                    let suggested_adj = format!("{}s", adj_str);
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: format!(
                            "Discordancia de plural: Para concordar con el determinante plural '{}' y el sustantivo '{}', usa '{}'.",
                            det_str, noun_str, suggested_adj
                        ),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            suggested_adj.chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 32,
                    });
                }
            }

            // Patrón: Determinante Plural + Adjetivo 1 + 'y'/'o' + Adjetivo 2 + Sustantivo Plural
            // Validar que "las rápidas y efectivas soluciones" NO genere alerta (caso válido).
            // Si el adjetivo 1 es singular en este patrón (ej. "las rápida y efectivas soluciones"):
            for window in word_indices.windows(5) {
                let det_tok = &chunk[window[0]];
                let adj1_tok = &chunk[window[1]];
                let conj_tok = &chunk[window[2]];
                let adj2_tok = &chunk[window[3]];
                let noun_tok = &chunk[window[4]];

                let det_str = document.get_span_content_str(&det_tok.span).to_lowercase();
                let adj1_str = document.get_span_content_str(&adj1_tok.span).to_lowercase();
                let conj_str = document.get_span_content_str(&conj_tok.span).to_lowercase();
                let _adj2_str = document.get_span_content_str(&adj2_tok.span).to_lowercase();
                let noun_str = document.get_span_content_str(&noun_tok.span).to_lowercase();

                let is_plural_det = PLURAL_DET_MASC.contains(&det_str.as_str())
                    || PLURAL_DET_FEM.contains(&det_str.as_str());
                let is_coord_conj =
                    conj_str == "y" || conj_str == "e" || conj_str == "o" || conj_str == "u";
                let noun_is_plural = noun_str.ends_with('s');

                if is_plural_det && is_coord_conj && noun_is_plural && !adj1_str.ends_with('s') {
                    let suggested = format!("{}s", adj1_str);
                    lints.push(Lint {
                        span: adj1_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: format!(
                            "Discordancia de plural en adjetivos coordinados: Se sugiere '{}' para concordar con '{}'.",
                            suggested, noun_str
                        ),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            suggested.chars().collect(),
                            document.get_span_content(&adj1_tok.span),
                        )],
                        priority: 32,
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
    fn accepts_valid_compound_plural_phrase() {
        let mut linter = SpanishPluralAgreement;
        let doc = Document::new_plain_english_curated("las rápidas y efectivas soluciones");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 0);
    }

    #[test]
    fn corrects_rapida_in_compound_phrase() {
        let mut linter = SpanishPluralAgreement;
        let doc = Document::new_plain_english_curated("las rápida y efectivas soluciones");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        let mut text_chars: Vec<char> = "las rápida y efectivas soluciones".chars().collect();
        lints[0].suggestions[0].apply(lints[0].span, &mut text_chars);
        let result: String = text_chars.into_iter().collect();
        assert_eq!(result, "las rápidas y efectivas soluciones");
    }
}
