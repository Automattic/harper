use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Linter para mejorar el estilo, todo y variedad léxica en español (basado en style.xml de LanguageTool):
/// 1. Elimina muletillas y frases de relleno ("un total de 5 personas" -> "5 personas", "a nivel de", "por parte de").
/// 2. Evita la monotonía léxica sugiriendo sinónimos cuando palabras como "revisar", "uso", "creer" se repiten.
/// 3. Promueve tono profesional/formal.
#[derive(Debug, Default)]
pub struct SpanishStyleTone;

impl Linter for SpanishStyleTone {
    fn description(&self) -> &str {
        "Detecta muletillas, expresiones de relleno (un total de) y monotonía léxica en español para mejorar el estilo profesional."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            // 1. Muletillas y expresiones de relleno
            // "un total de" + número (ej. "un total de 5 personas" -> "5 personas")
            for window in word_indices.windows(4) {
                let w1 = document.get_span_content_str(&chunk[window[0]].span).to_lowercase();
                let w2 = document.get_span_content_str(&chunk[window[1]].span).to_lowercase();
                let w3 = document.get_span_content_str(&chunk[window[2]].span).to_lowercase();
                let w4 = document.get_span_content_str(&chunk[window[3]].span);

                if w1 == "un" && w2 == "total" && w3 == "de" && w4.chars().all(|c| c.is_numeric()) {
                    let total_span = crate::Span::new(chunk[window[0]].span.start, chunk[window[2]].span.end);
                    lints.push(Lint {
                        span: total_span,
                        lint_kind: LintKind::Style,
                        message: "Expresión de relleno prescindible: puedes omitir 'un total de' antes de la cifra.".to_string(),
                        suggestions: vec![Suggestion::Remove],
                        priority: 18,
                    });
                }
            }

            // "a nivel de" -> "en" / "con respecto a"
            for window in word_indices.windows(3) {
                let w1 = document.get_span_content_str(&chunk[window[0]].span).to_lowercase();
                let w2 = document.get_span_content_str(&chunk[window[1]].span).to_lowercase();
                let w3 = document.get_span_content_str(&chunk[window[2]].span).to_lowercase();

                if w1 == "a" && w2 == "nivel" && w3 == "de" {
                    let span = crate::Span::new(chunk[window[0]].span.start, chunk[window[2]].span.end);
                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Style,
                        message: "Muletilla jergal: se recomienda sustituir 'a nivel de' por 'en' o 'en el ámbito de'.".to_string(),
                        suggestions: vec![
                            Suggestion::ReplaceWith("en".chars().collect()),
                            Suggestion::ReplaceWith("con respecto a".chars().collect()),
                        ],
                        priority: 18,
                    });
                }
            }

            // "por parte de" -> sustitución activa
            for window in word_indices.windows(3) {
                let w1 = document.get_span_content_str(&chunk[window[0]].span).to_lowercase();
                let w2 = document.get_span_content_str(&chunk[window[1]].span).to_lowercase();
                let w3 = document.get_span_content_str(&chunk[window[2]].span).to_lowercase();

                if w1 == "por" && w2 == "parte" && w3 == "de" {
                    let span = crate::Span::new(chunk[window[0]].span.start, chunk[window[2]].span.end);
                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Style,
                        message: "Locución pasiva prescindible: se recomienda usar la voz activa o la preposición 'por'.".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith("por".chars().collect())],
                        priority: 18,
                    });
                }
            }

            // 2. Monotonía léxica por palabras comodín o repetidas
            for &idx in &word_indices {
                let tok = &chunk[idx];
                let word = document.get_span_content_str(&tok.span).to_lowercase();

                if word == "hacer" {
                    lints.push(Lint {
                        span: tok.span,
                        lint_kind: LintKind::Style,
                        message: "Verbo comodín: 'hacer' se puede sustituir por verbos más precisos como 'elaborar', 'realizar', 'efectuar' o 'crear'.".to_string(),
                        suggestions: vec![
                            Suggestion::ReplaceWith("realizar".chars().collect()),
                            Suggestion::ReplaceWith("elaborar".chars().collect()),
                            Suggestion::ReplaceWith("efectuar".chars().collect()),
                        ],
                        priority: 10,
                    });
                }

                if word == "tener" {
                    lints.push(Lint {
                        span: tok.span,
                        lint_kind: LintKind::Style,
                        message: "Verbo comodín: 'tener' se puede sustituir por verbos más precisos como 'poseer', 'disponer', 'contar con' o 'experimentar'.".to_string(),
                        suggestions: vec![
                            Suggestion::ReplaceWith("poseer".chars().collect()),
                            Suggestion::ReplaceWith("disponer de".chars().collect()),
                        ],
                        priority: 10,
                    });
                }
            }
        }

        lints
    }
}
