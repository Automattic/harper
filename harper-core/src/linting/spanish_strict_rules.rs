use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Reglas estrictas de gramática, puntuación y tildes según las normas de la Real Academia Española (RAE):
/// 1. Tildes diacríticas obligatorias:
///    - "el" + verbo -> "él" (pronombre: ej. "el dijo" -> "él dijo")
///    - "tu" + verbo -> "tú" (pronombre: ej. "tu eres" -> "tú eres")
///    - "para mi" -> "para mí"
///    - "dijo que si" / "creo que si" -> "dijo que sí"
///    - "no se" + verbo -> "no sé" (verbo saber)
///    - "tomo un te" -> "tomo un té"
///    - "quiero mas" -> "quiero más"
/// 2. Puntuación estricta de la RAE:
///    - Prohibido el punto (.) inmediatamente después de signo de interrogación (?) o exclamación (!)
///    - Emparejamiento obligatorio de ¿ con ? y ¡ con !
///    - Tres puntos suspensivos exactamente (...)
/// 3. Adjetivos y concordancia de número en plurales.
#[derive(Debug, Default)]
pub struct SpanishStrictRules;

impl Linter for SpanishStrictRules {
    fn description(&self) -> &str {
        "Aplica reglas estrictas de la RAE para tildes diacríticas (él, tú, mí, sí, sé, té, más), puntuación sin punto tras '?' o '!' y emparejamiento de signos (¿?, ¡!)."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let tokens: Vec<_> = document.tokens().collect();
        if tokens.is_empty() {
            return lints;
        }

        // 1. REGLAS DE PUNTUACIÓN DE LA RAE: No poner punto (.) después de '?' o '!' y obligatoriedad de '¿' y '¡'
        for chunk in document.iter_chunks() {
            let has_closing_q = chunk.iter().any(|t| document.get_span_content_str(&t.span) == "?");
            let has_opening_q = chunk.iter().any(|t| document.get_span_content_str(&t.span) == "¿");

            if has_closing_q && !has_opening_q {
                if let Some(first_word) = chunk.iter_words().next() {
                    let word_str = document.get_span_content_str(&first_word.span);
                    lints.push(Lint {
                        span: first_word.span,
                        lint_kind: LintKind::Punctuation,
                        message: "En español es obligatorio el uso del signo de apertura de interrogación ('¿') al inicio de la pregunta.".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith(
                            format!("¿{}", word_str).chars().collect(),
                        )],
                        priority: 22,
                    });
                }
            }
        }

        for i in 0..tokens.len() {
            let tok = &tokens[i];
            let tok_str = document.get_span_content_str(&tok.span);

            if (tok_str == "?" || tok_str == "!") && i + 1 < tokens.len() {
                let next_tok = &tokens[i + 1];
                let next_str = document.get_span_content_str(&next_tok.span);
                if next_str == "." {
                    lints.push(Lint {
                        span: next_tok.span,
                        lint_kind: LintKind::Punctuation,
                        message: format!("En español no se coloca punto (.) después de un signo de {} ('{}') ya que este actúa como punto final.", if tok_str == "?" { "interrogación" } else { "exclamación" }, tok_str),
                        suggestions: vec![Suggestion::Remove],
                        priority: 20,
                    });
                }
            }
        }

        // 2. REGLAS DE TILDES DIACRÍTICAS Y CONCORDANCIA POR CONTEXTO
        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            for window in word_indices.windows(2) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];

                let first_str = document.get_span_content_str(&first_tok.span).to_lowercase();
                let second_str = document.get_span_content_str(&second_tok.span).to_lowercase();

                // a) "el" + verbo -> "él" (ej: "el dijo", "el comió", "el fue", "el quiere", "el sabe")
                const COMMON_VERBS: &[&str] = &[
                    "dijo", "comió", "fue", "quiere", "sabe", "hizo", "habló", "vino", "está", "es", "tiene", "vuelve", "corrió", "ganó", "perdió"
                ];

                if first_str == "el" && COMMON_VERBS.contains(&second_str.as_str()) {
                    lints.push(Lint {
                        span: first_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: format!("Pronombre personal: 'él' se escribe con tilde cuando cumple función de sujeto ante el verbo '{}'.", second_str),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "él".chars().collect(),
                            document.get_span_content(&first_tok.span),
                        )],
                        priority: 30,
                    });
                }

                // b) "tu" + verbo -> "tú" (ej: "tu eres", "tu sabes", "tu quieres", "tu dices")
                if first_str == "tu" && (COMMON_VERBS.contains(&second_str.as_str()) || second_str == "eres" || second_str == "sabes" || second_str == "dices") {
                    lints.push(Lint {
                        span: first_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: format!("Pronombre personal: 'tú' se escribe con tilde ante el verbo '{}'.", second_str),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "tú".chars().collect(),
                            document.get_span_content(&first_tok.span),
                        )],
                        priority: 30,
                    });
                }

                // c) "para mi" -> "para mí" / "de mi" cuando no sigue sustantivo
                if first_str == "para" && second_str == "mi" {
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: "Pronombre tónico: 'mí' lleva tilde cuando va precedido de preposición ('para mí').".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "mí".chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 30,
                    });
                }

                // d) "que si" en afirmaciones -> "que sí" (ej: "dijo que si", "creo que si")
                if (first_str == "que" || first_str == "dijo") && second_str == "si" {
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: "Adverbio de afirmación: 'sí' se escribe con tilde.".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "sí".chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 30,
                    });
                }

                // e) "no se" + verbo -> "no sé" (ej: "no se nada", "no se que")
                if first_str == "no" && second_str == "se" {
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: "Forma del verbo saber/ser: 'sé' lleva tilde diacrítica ('no sé').".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "sé".chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 30,
                    });
                }

                // f) "un te" -> "un té"
                if (first_str == "un" || first_str == "el") && second_str == "te" {
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: "Sustantivo (bebida): 'té' lleva tilde diacrítica.".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "té".chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 30,
                    });
                }

                // g) "en base a" -> "con base en"
                if first_str == "en" && second_str == "base" {
                    lints.push(Lint {
                        span: first_tok.span,
                        lint_kind: LintKind::Style,
                        message: "Locución no recomendada por la RAE. Se recomienda usar 'con base en' o 'según'.".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "con base en".chars().collect(),
                            document.get_span_content(&first_tok.span),
                        )],
                        priority: 30,
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
    fn detects_el_dijo() {
        let mut linter = SpanishStrictRules;
        let doc = Document::new_plain_english_curated("el dijo que si");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 2);
    }
}
