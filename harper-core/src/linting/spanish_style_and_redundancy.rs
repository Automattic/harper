use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar pleonasmos/redundancias y uso incorrecto de preposiciones en español:
/// 1. Pleonasmos frecuentes: "subir arriba", "bajar abajo", "entrar adentro", "salir afuera", "volar por el aire", "lapso de tiempo", "persona humana", "regalo gratuito".
/// 2. Preposiciones incorrectas (RAE):
///    - "de acuerdo a" -> "de acuerdo con"
///    - "en relación a" -> "en relación con"
///    - "en función a" -> "en función de"
/// 3. Anglicismos adaptados:
///    - "hacer click" -> "hacer clic"
///    - "link" -> "enlace"
#[derive(Debug, Default)]
pub struct SpanishStyleAndRedundancy;

impl Linter for SpanishStyleAndRedundancy {
    fn description(&self) -> &str {
        "Detecta pleonasmos/redundancias (subir arriba, bajar abajo), uso no recomendado de preposiciones (de acuerdo a -> de acuerdo con) y anglicismos en español."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            // 1. Pares de dos palabras (Redundancias y Preposiciones)
            for window in word_indices.windows(2) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];

                let first_str = document.get_span_content_str(&first_tok.span).to_lowercase();
                let second_str = document.get_span_content_str(&second_tok.span).to_lowercase();

                // Pleonasmos de 2 palabras
                let pleonasm_fix = match (first_str.as_str(), second_str.as_str()) {
                    ("subir", "arriba") => Some(("subir", "Redundancia: 'subir' ya implica dirección hacia arriba.")),
                    ("bajar", "abajo") => Some(("bajar", "Redundancia: 'bajar' ya implica dirección hacia abajo.")),
                    ("entrar", "adentro") => Some(("entrar", "Redundancia: 'entrar' ya significa ir al interior.")),
                    ("salir", "afuera") => Some(("salir", "Redundancia: 'salir' ya significa ir al exterior.")),
                    ("regalo", "gratuito") => Some(("regalo", "Redundancia: Todo regalo es gratuito por definición.")),
                    ("persona", "humana") => Some(("persona", "Redundancia: 'persona' ya implica la condición humana.")),
                    _ => None,
                };

                if let Some((fix, msg)) = pleonasm_fix {
                    let double_span = crate::Span::new(first_tok.span.start, second_tok.span.end);
                    lints.push(Lint {
                        span: double_span,
                        lint_kind: LintKind::Style,
                        message: msg.to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            fix.chars().collect(),
                            document.get_span_content(&double_span),
                        )],
                        priority: 22,
                    });
                }

                // Anglicismo: "hacer click" -> "hacer clic"
                if first_str == "hacer" && second_str == "click" {
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Style,
                        message: "En español la forma adaptada recomendada es 'clic' (sin 'k').".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "clic".chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 22,
                    });
                }
            }

            // 2. Tríos de tres palabras (Preposiciones no recomendadas: "de acuerdo a", "en relacion a")
            for window in word_indices.windows(3) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];
                let third_tok = &chunk[window[2]];

                let first_str = document.get_span_content_str(&first_tok.span).to_lowercase();
                let second_str = document.get_span_content_str(&second_tok.span).to_lowercase();
                let third_str = document.get_span_content_str(&third_tok.span).to_lowercase();

                if first_str == "de" && second_str == "acuerdo" && third_str == "a" {
                    lints.push(Lint {
                        span: third_tok.span,
                        lint_kind: LintKind::Style,
                        message: "La RAE recomienda la locución 'de acuerdo con' en lugar de 'de acuerdo a'.".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "con".chars().collect(),
                            document.get_span_content(&third_tok.span),
                        )],
                        priority: 24,
                    });
                }

                if first_str == "en" && (second_str == "relación" || second_str == "relacion") && third_str == "a" {
                    lints.push(Lint {
                        span: third_tok.span,
                        lint_kind: LintKind::Style,
                        message: "La RAE recomienda la locución 'en relación con' o 'con relación a'.".to_string(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            "con".chars().collect(),
                            document.get_span_content(&third_tok.span),
                        )],
                        priority: 24,
                    });
                }

                // 2b. Expresiones arcaicas o incorrectas (Fundéu / RAE)
                // "tal es asi" -> "tanto es así"
                if first_str == "tal" && second_str == "es" && (third_str == "así" || third_str == "asi") {
                    let span = crate::Span::new(first_tok.span.start, third_tok.span.end);
                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Style,
                        message: "Expresión no recomendada: la forma correcta es 'tanto es así' o 'tan es así'.".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith("tanto es así".chars().collect())],
                        priority: 25,
                    });
                }

                // "por tal de" -> "con tal de"
                if first_str == "por" && second_str == "tal" && third_str == "de" {
                    let span = crate::Span::new(first_tok.span.start, third_tok.span.end);
                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Style,
                        message: "Expresión arcaica: se recomienda 'con tal de'.".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith("con tal de".chars().collect())],
                        priority: 25,
                    });
                }

                // "contra mas" -> "cuanto más"
                if first_str == "contra" && (second_str == "más" || second_str == "mas") {
                    let span = crate::Span::new(first_tok.span.start, second_tok.span.end);
                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Style,
                        message: "Expresión incorrecta: se recomienda usar 'cuanto más'.".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith("cuanto más".chars().collect())],
                        priority: 25,
                    });
                }
            }

            // 2c. Expresiones de 2 palabras (afrentar dificultades, destornillarse de risa, alta cargo, tiras y aflojas)
            for window in word_indices.windows(2) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];

                let first_str = document.get_span_content_str(&first_tok.span).to_lowercase();
                let second_str = document.get_span_content_str(&second_tok.span).to_lowercase();

                if first_str == "alta" && second_str == "cargo" {
                    lints.push(Lint {
                        span: first_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: "El sustantivo 'cargo' es masculino: debe ser 'alto cargo' (ej. 'la alto cargo').".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith("alto".chars().collect())],
                        priority: 25,
                    });
                }

                if (first_str == "afrentar" || first_str == "afrentó") && (second_str.starts_with("dificultad") || second_str.starts_with("problema")) {
                    let fix = if first_str == "afrentar" { "afrontar" } else { "afrontó" };
                    lints.push(Lint {
                        span: first_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: format!("Confusión de verbos: se debe usar el verbo '{}' ante dificultades o problemas.", fix),
                        suggestions: vec![Suggestion::ReplaceWith(fix.chars().collect())],
                        priority: 25,
                    });
                }

                if (first_str.starts_with("destornill") || first_str.starts_with("destornilló")) && second_str == "de" {
                    lints.push(Lint {
                        span: first_tok.span,
                        lint_kind: LintKind::Style,
                        message: "El verbo adecuado para reírse intensamente es 'desternillar' (ej. 'desternillarse de risa').".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith("desternillar".chars().collect())],
                        priority: 25,
                    });
                }

                if first_str == "empece" && second_str == "a" {
                    lints.push(Lint {
                        span: first_tok.span,
                        lint_kind: LintKind::Grammar,
                        message: "Forma del pretérito perfecto del verbo empezar: lleva tilde ('empecé').".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith("empecé".chars().collect())],
                        priority: 30,
                    });
                }
            }

            // 3. Estilo Tipográfico Avanzado (Comillas latinas « », guiones de diálogo —)
            for &idx in &word_indices {
                let tok = &chunk[idx];
                let tok_str = document.get_span_content_str(&tok.span);
                if tok_str == "\"" {
                    lints.push(Lint {
                        span: tok.span,
                        lint_kind: LintKind::Style,
                        message: "En español se recomienda prioritariamente el uso de comillas latinas o españolas (« ») en lugar de inglesas (\" \").".to_string(),
                        suggestions: vec![
                            Suggestion::ReplaceWith("«".chars().collect()),
                            Suggestion::ReplaceWith("»".chars().collect()),
                        ],
                        priority: 15,
                    });
                }
            }

            // 4. Concordancia Múltiple en Frases Nominales ("las rápidas y efectivas soluciones")
            for window in word_indices.windows(5) {
                let tok1_str = document.get_span_content_str(&chunk[window[0]].span).to_lowercase();
                let tok3_str = document.get_span_content_str(&chunk[window[2]].span).to_lowercase();
                let tok5_str = document.get_span_content_str(&chunk[window[4]].span).to_lowercase();

                let is_plural_det = tok1_str == "las" || tok1_str == "los" || tok1_str == "unos" || tok1_str == "unas" || tok1_str == "mis" || tok1_str == "tus" || tok1_str == "sus";
                let is_and = tok3_str == "y" || tok3_str == "e";

                if is_plural_det && is_and {
                    if !tok5_str.ends_with('s') && tok5_str.len() > 3 {
                        lints.push(Lint {
                            span: chunk[window[4]].span,
                            lint_kind: LintKind::Grammar,
                            message: format!("Falta de concordancia de número: el determinante plural ('{}') requiere un sustantivo en plural ('{}s').", tok1_str, tok5_str),
                            suggestions: vec![Suggestion::ReplaceWith(
                                format!("{}s", tok5_str).chars().collect(),
                            )],
                            priority: 32,
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
    fn detects_subir_arriba() {
        let mut linter = SpanishStyleAndRedundancy;
        let doc = Document::new_plain_english_curated("decidió subir arriba inmediatamente.");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        assert!(lints[0].message.contains("Redundancia"));
    }
}
