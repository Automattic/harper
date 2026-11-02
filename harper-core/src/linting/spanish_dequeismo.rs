use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar dequeísmo ("me dijo de que") y queísmo ("estoy seguro que").
#[derive(Debug, Default)]
pub struct SpanishDequeismo;

impl Linter for SpanishDequeismo {
    fn description(&self) -> &str {
        "Detecta casos de dequeísmo (ej. 'me dijo de que' por 'me dijo que') y queísmo (ej. 'estoy seguro que' por 'estoy seguro de que') en español."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // Verbos que rigen "que" directamente (Dequeísmo cuando se pone "de que")
        const DIRECT_QUE_VERBS: &[&str] = &[
            "dijo",
            "decía",
            "afirmó",
            "pensó",
            "creo",
            "creía",
            "opinó",
            "aseguró",
            "anunció",
            "sostenía",
        ];

        // Expresiones de certeza o estado que rigen "de que" (Queísmo cuando falta "de")
        const REQUIRES_DE_QUE_EXPRESSIONS: &[&str] = &[
            "seguro",
            "segura",
            "seguros",
            "seguras",
            "convencido",
            "convencida",
            "convencidos",
            "convencidas",
            "antes",
            "después",
            "despues",
        ];

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            // Detectar Dequeísmo: VERBO + "de" + "que" (ej: "dijo de que")
            for window in word_indices.windows(3) {
                let verb_token = &chunk[window[0]];
                let de_token = &chunk[window[1]];
                let que_token = &chunk[window[2]];

                let verb_str = document
                    .get_span_content_str(&verb_token.span)
                    .to_lowercase();
                let de_str = document.get_span_content_str(&de_token.span).to_lowercase();
                let que_str = document
                    .get_span_content_str(&que_token.span)
                    .to_lowercase();

                if DIRECT_QUE_VERBS.contains(&verb_str.as_str())
                    && de_str == "de"
                    && que_str == "que"
                {
                    lints.push(Lint {
                        span: de_token.span,
                        lint_kind: LintKind::Grammar,
                        message: format!(
                            "Dequeísmo: El verbo '{}' no requiere la preposición 'de' antes de 'que'.",
                            verb_str
                        ),
                        suggestions: vec![Suggestion::Remove],
                        priority: 32,
                    });
                }
            }

            // Detectar Queísmo: ADJETIVO + "que" (sin "de") (ej: "seguro que")
            for window in word_indices.windows(2) {
                let first_token = &chunk[window[0]];
                let second_token = &chunk[window[1]];

                let first_str = document
                    .get_span_content_str(&first_token.span)
                    .to_lowercase();
                let second_str = document
                    .get_span_content_str(&second_token.span)
                    .to_lowercase();

                if REQUIRES_DE_QUE_EXPRESSIONS.contains(&first_str.as_str()) && second_str == "que"
                {
                    lints.push(Lint {
                        span: second_token.span,
                        lint_kind: LintKind::Grammar,
                        message: format!(
                            "Queísmo: La expresión '{}' requiere 'de que' en lugar de únicamente 'que'.",
                            first_str
                        ),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "de que".chars().collect(),
                            document.get_span_content(&second_token.span),
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
    fn detects_dequeismo() {
        let mut linter = SpanishDequeismo;
        let doc = Document::new_plain_english_curated("Él dijo de que vendría.");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        assert!(lints[0].message.contains("Dequeísmo"));
    }

    #[test]
    fn detects_queismo() {
        let mut linter = SpanishDequeismo;
        let doc = Document::new_plain_english_curated("Estoy seguro que funciona.");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        assert!(lints[0].message.contains("Queísmo"));
    }
}
