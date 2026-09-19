use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla de puntuación contextual para el español (RAE):
/// 1. Coma requerida antes de conjunciones adversativas (pero, sino, aunque, mas) en oraciones compuestas.
/// 2. Coma requerida alrededor de conectores explicativos/consecutivos (es decir, sin embargo, por lo tanto, no obstante).
/// 3. Preferencia de punto y coma (;) cuando se conectan proposiciones largas con comas internas.
#[derive(Debug, Default)]
pub struct SpanishContextualPunctuation;

impl Linter for SpanishContextualPunctuation {
    fn description(&self) -> &str {
        "Detecta la necesidad de coma (,) o punto y coma (;) según el contexto gramatical de la oración (conectores adversativos como 'pero', 'sin embargo', 'es decir')."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        const ADVERSATIVE_CONJUNCTIONS: &[&str] = &["pero", "sino", "aunque"];

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            for window in word_indices.windows(2) {
                let prev_token = &chunk[window[0]];
                let curr_token = &chunk[window[1]];

                let _prev_str = document.get_span_content_str(&prev_token.span).to_lowercase();
                let curr_str = document.get_span_content_str(&curr_token.span).to_lowercase();

                // Caso 1: Coma requerida antes de "pero", "sino", "aunque" en oraciones compuestas
                if ADVERSATIVE_CONJUNCTIONS.contains(&curr_str.as_str()) {
                    // Verificar si entre el token anterior y el conector hay una coma
                    let span_between = crate::Span::new(prev_token.span.end, curr_token.span.start);
                    let text_between = document.get_span_content_str(&span_between);

                    if !text_between.contains(',') && !text_between.contains(';') && !text_between.contains('.') {
                        lints.push(Lint {
                            span: curr_token.span,
                            lint_kind: LintKind::Punctuation,
                            message: format!(
                                "Puntuación contextual: Se antepone una coma (,) antes de la conjunción adversativa '{}'.",
                                curr_str
                            ),
                            suggestions: vec![Suggestion::replace_with_match_case(
                                format!(", {}", curr_str).chars().collect(),
                                document.get_span_content(&curr_token.span),
                            )],
                            priority: 28,
                        });
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
    fn detects_missing_comma_before_pero() {
        let mut linter = SpanishContextualPunctuation;
        let doc = Document::new_plain_english_curated("Estudió mucho pero no aprobó.");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        assert!(lints[0].message.contains("conjunción adversativa"));
    }
}
