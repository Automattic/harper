use super::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, TokenStringExt};

/// Regla de gramática para detectar confusiones frecuentes en verbos en español:
/// - "tubo" / "tubieron" (tub-) cuando se refiere al verbo haber/tener (ej. "tubieron" -> "tuvieron")
/// - "abia" / "ubieras" / "ubieron" (faltante de H en haber)
#[derive(Default)]
pub struct SpanishHomophones;

impl Linter for SpanishHomophones {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for token in chunk.iter_words() {
                let word = document.get_span_content_str(&token.span);
                let lower = word.to_lowercase();

                // 1. Confusión tubieron/tubo (verbo tener vs tubo de cañería)
                if lower == "tubieron" || lower == "tubo" {
                    let correccion = if lower == "tubieron" {
                        "tuvieron"
                    } else {
                        "tuvo"
                    };
                    lints.push(Lint {
                        span: token.span,
                        message: format!("Si te refieres al verbo 'tener', se escribe con 'v' ('{}'). 'Tubo' con 'b' es un objeto físico.", correccion),
                        suggestions: vec![Suggestion::ReplaceWith(correccion.chars().collect())],
                        priority: 31,
                        lint_kind: LintKind::Miscellaneous,
                    });
                }

                // 2. Omisión de 'H' en formas del verbo haber (abia, ubiera, ubieron, etc.)
                if lower == "abia"
                    || lower == "abias"
                    || lower == "abiamos"
                    || lower == "abian"
                    || lower == "ubiera"
                    || lower == "ubieras"
                    || lower == "ubieron"
                {
                    let correccion = format!("h{}", lower);
                    lints.push(Lint {
                        span: token.span,
                        message: format!(
                            "Las formas del verbo 'haber' llevan 'h' inicial. Debería ser '{}'.",
                            correccion
                        ),
                        suggestions: vec![Suggestion::ReplaceWith(correccion.chars().collect())],
                        priority: 31,
                        lint_kind: LintKind::Miscellaneous,
                    });
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Detecta confusiones de homófonos comunes en español (B/V, H faltante en verbos)."
    }
}
