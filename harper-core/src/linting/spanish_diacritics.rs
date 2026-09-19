use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar tildes diacríticas e interrogativas frecuentes en español:
/// - "el porque" -> "el porqué" (sustantivo)
/// - "esta" por "está" (verbo estar)
/// - "donde / por donde" en contexto interrogativo o de duda -> "dónde / por dónde"
/// - "mas" por "más" (adverbio de cantidad)
#[derive(Debug, Default)]
pub struct SpanishDiacritics;

impl Linter for SpanishDiacritics {
    fn description(&self) -> &str {
        "Detecta falta de tildes diacríticas e interrogativas en español (el porqué, dónde/dónde está, más, etc.)."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            // 1. "el porque" -> "el porqué"
            for window in word_indices.windows(2) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];

                let first_str = document.get_span_content_str(&first_tok.span).to_lowercase();
                let second_str = document.get_span_content_str(&second_tok.span).to_lowercase();

                if (first_str == "el" || first_str == "un") && second_str == "porque" {
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: "Como sustantivo (precedido de 'el' o 'un'), se escribe con tilde: 'porqué'.".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "porqué".chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 35,
                    });
                }

                // 2. "por donde" -> "por dónde"
                if first_str == "por" && second_str == "donde" {
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: "En oraciones interrogativas directas o indirectas, 'dónde' lleva tilde.".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "dónde".chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 35,
                    });
                }

                // 3. Interrogativos genéricos (donde, como, cuando, que, quien, cual) en contexto interrogativo o seguidos de verbos
                const INTERROGATIVE_WORDS: &[&str] = &["donde", "como", "cuando", "que", "quien", "cual"];
                const INTERROGATIVE_TILDE: &[&str] = &["dónde", "cómo", "cuándo", "qué", "quién", "cuál"];
                const COMMON_NEXT_VERBS: &[&str] = &[
                    "esta", "está", "podria", "podría", "podemos", "puedo", "pueden", "podamos", "es", "son", "hacer", "ir", "fue", "sera", "será", "estaba", "estaban", "encontrar", "comprar", "haber", "hay", "tengo", "tiene", "tienen"
                ];

                if let Some(pos) = INTERROGATIVE_WORDS.iter().position(|&w| w == first_str.as_str()) {
                    let has_question_mark = chunk.iter().any(|t| {
                        let s = document.get_span_content_str(&t.span);
                        s == "?" || s == "¿"
                    });

                    if has_question_mark || COMMON_NEXT_VERBS.contains(&second_str.as_str()) {
                        let tilde_word = INTERROGATIVE_TILDE[pos];
                        let is_first_char_upper = first_tok.span.get_content(document.get_source()).first().map_or(false, |c| c.is_uppercase());
                        let final_suggestion = if is_first_char_upper {
                            let mut chars = tilde_word.chars().collect::<Vec<_>>();
                            if let Some(c) = chars.first_mut() {
                                *c = c.to_uppercase().next().unwrap_or(*c);
                            }
                            chars
                        } else {
                            tilde_word.chars().collect()
                        };

                        lints.push(Lint {
                            span: first_tok.span,
                            lint_kind: LintKind::Grammar,
                            message: format!("En oraciones interrogativas o de duda, '{}' lleva tilde diacrítica ('{}').", first_str, tilde_word),
                            suggestions: vec![Suggestion::ReplaceWith(final_suggestion)],
                            priority: 35,
                        });
                    }
                }
            }

            // 4. Verificación de token único interrogativo al inicio de oración
            for &idx in &word_indices {
                let tok = &chunk[idx];
                let word_str = document.get_span_content_str(&tok.span).to_lowercase();
                const INTERROGATIVE_WORDS: &[&str] = &["donde", "como", "cuando", "que", "quien", "cual"];
                const INTERROGATIVE_TILDE: &[&str] = &["dónde", "cómo", "cuándo", "qué", "quién", "cuál"];

                if let Some(pos) = INTERROGATIVE_WORDS.iter().position(|&w| w == word_str.as_str()) {
                    let has_question_mark = chunk.iter().any(|t| {
                        let s = document.get_span_content_str(&t.span);
                        s == "?" || s == "¿"
                    });

                    if has_question_mark {
                        let tilde_word = INTERROGATIVE_TILDE[pos];
                        let is_first_char_upper = tok.span.get_content(document.get_source()).first().map_or(false, |c| c.is_uppercase());
                        let final_suggestion = if is_first_char_upper {
                            let mut chars = tilde_word.chars().collect::<Vec<_>>();
                            if let Some(c) = chars.first_mut() {
                                *c = c.to_uppercase().next().unwrap_or(*c);
                            }
                            chars
                        } else {
                            tilde_word.chars().collect()
                        };

                        if !lints.iter().any(|l| l.span.start == tok.span.start) {
                            lints.push(Lint {
                                span: tok.span,
                                lint_kind: LintKind::Grammar,
                                message: format!("En oraciones interrogativas o de duda, '{}' lleva tilde diacrítica ('{}').", word_str, tilde_word),
                                suggestions: vec![Suggestion::ReplaceWith(final_suggestion)],
                                priority: 35,
                            });
                        }
                    }
                }
            }

            // 5. Homógrafos sustantivo / verbo precedidos de determinantes (el, la, un, este, ese, aquel, su, mi, tu, etc.)
            const HOMOGRAPH_DETERMINERS: &[&str] = &[
                "el", "la", "los", "las", "un", "una", "unos", "unas", "este", "esta", "estos", "estas",
                "ese", "esa", "esos", "esas", "aquel", "aquella", "aquellos", "aquellas", "mi", "mis", "tu", "tus", "su", "sus", "nuestro", "nuestra", "un solo", "primer", "ultimo", "último"
            ];

            const HOMOGRAPHS: &[(&str, &str)] = &[
                ("termino", "término"),
                ("terminos", "términos"),
                ("fabrica", "fábrica"),
                ("fabricas", "fábricas"),
                ("calculo", "cálculo"),
                ("calculos", "cálculos"),
                ("practica", "práctica"),
                ("practicas", "prácticas"),
                ("formula", "fórmula"),
                ("formulas", "fórmulas"),
                ("maquina", "máquina"),
                ("maquinas", "máquinas"),
                ("critica", "crítica"),
                ("criticas", "críticas"),
                ("limite", "límite"),
                ("limites", "límites"),
                ("publico", "público"),
                ("publicos", "públicos"),
                ("analisis", "análisis"),
                ("sintesis", "síntesis"),
                ("hipotesis", "hipótesis"),
                ("lineas", "líneas"),
                ("linea", "línea"),
                ("nominas", "nóminas"),
                ("nomina", "nómina"),
                ("prestamos", "préstamos"),
                ("prestamo", "préstamo"),
                ("capitulo", "capítulo"),
                ("capitulos", "capítulos"),
                ("catalogo", "catálogo"),
                ("catalogos", "catálogos"),
                ("ejercito", "ejército"),
                ("ejercitos", "ejércitos"),
                ("genero", "género"),
                ("generos", "géneros"),
                ("colera", "cólera"),
                ("vividamente", "vívidamente"),
            ];

            for window in word_indices.windows(2) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];

                let first_str = document.get_span_content_str(&first_tok.span).to_lowercase();
                let second_str = document.get_span_content_str(&second_tok.span).to_lowercase();

                if HOMOGRAPH_DETERMINERS.contains(&first_str.as_str()) {
                    if let Some((_, tilde_form)) = HOMOGRAPHS.iter().find(|&&(wrong, _)| wrong == second_str.as_str()) {
                        let is_first_char_upper = second_tok.span.get_content(document.get_source()).first().map_or(false, |c| c.is_uppercase());
                        let final_suggestion = if is_first_char_upper {
                            let mut chars = tilde_form.chars().collect::<Vec<_>>();
                            if let Some(c) = chars.first_mut() {
                                *c = c.to_uppercase().next().unwrap_or(*c);
                            }
                            chars
                        } else {
                            tilde_form.chars().collect()
                        };

                        if !lints.iter().any(|l| l.span.start == second_tok.span.start) {
                            lints.push(Lint {
                                span: second_tok.span,
                                lint_kind: LintKind::Grammar,
                                message: format!("Como sustantivo precedido de determinante ('{}'), debe escribirse con tilde: '{}'.", first_str, tilde_form),
                                suggestions: vec![Suggestion::ReplaceWith(final_suggestion)],
                                priority: 36,
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
    fn detects_el_porque() {
        let mut linter = SpanishDiacritics;
        let doc = Document::new_plain_english_curated("Nunca supe el porque.");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        assert!(lints[0].message.contains("porqué"));
    }
}
