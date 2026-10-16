use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar discordancias evidentes de número entre sujetos plurales/singulares y verbos en español.
/// Ejemplos: "los niños juega" -> "los niños juegan", "el perro corren" -> "el perro corre".
#[derive(Debug, Default)]
pub struct SpanishSubjectVerbAgreement;

impl Linter for SpanishSubjectVerbAgreement {
    fn description(&self) -> &str {
        "Detecta discordancias de número en español entre determinantes/sustantivos plurales y verbos en singular (ej. 'los niños juega')."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        const COMMON_PLURAL_DETERMINERS: &[&str] = &[
            "los", "las", "unos", "unas", "estos", "estas", "nuestros", "nuestras",
        ];

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            for window in word_indices.windows(3) {
                let det_token = &chunk[window[0]];
                let noun_token = &chunk[window[1]];
                let verb_token = &chunk[window[2]];

                let det_str = document
                    .get_span_content_str(&det_token.span)
                    .to_lowercase();
                let noun_str = document
                    .get_span_content_str(&noun_token.span)
                    .to_lowercase();
                let verb_str = document
                    .get_span_content_str(&verb_token.span)
                    .to_lowercase();

                // Si el determinante y sustantivo son claramente plurales (terminan en 's') pero el verbo termina en vocal (singular)
                if COMMON_PLURAL_DETERMINERS.contains(&det_str.as_str())
                    && noun_str.ends_with('s')
                    && (verb_str.ends_with('a')
                        || verb_str.ends_with('e')
                        || verb_str.ends_with('ó'))
                    && !verb_str.ends_with('n')
                    && verb_str.len() > 2
                {
                    let suggested_verb = format!("{}n", verb_str);
                    lints.push(Lint {
                        span: verb_token.span,
                        lint_kind: LintKind::Grammar,
                        message: format!(
                            "El sujeto plural '{} {}' no concuerda con el verbo en singular '{}'. Usa '{}'.",
                            det_str, noun_str, verb_str, suggested_verb
                        ),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            suggested_verb.chars().collect(),
                            document.get_span_content(&verb_token.span),
                        )],
                        priority: 33,
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
    fn corrects_los_ninos_juega() {
        let mut linter = SpanishSubjectVerbAgreement;
        let doc = Document::new_plain_english_curated("Los niños juega en el parque.");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        let mut text_chars: Vec<char> = "Los niños juega en el parque.".chars().collect();
        lints[0].suggestions[0].apply(lints[0].span, &mut text_chars);
        let result: String = text_chars.into_iter().collect();
        assert_eq!(result, "Los niños juegan en el parque.");
    }
}
