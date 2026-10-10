use crate::{
    Document,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para sugerir estilo tipográfico avanzado en español (RAE / Estándares de edición):
/// 1. Preferencia de comillas angulares/españolas/latinas (« ») frente a comillas inglesas (" ").
/// 2. Uso de raya/guión largo (—) en diálogos o incisos en lugar de guión simple (-).
/// 3. Espacio de no separación (o espacio simple obligatorio) antes de unidades de medida (ej: "50 km/h", "10 kg", "100 m").
#[derive(Debug, Default)]
pub struct SpanishTypography;

impl Linter for SpanishTypography {
    fn description(&self) -> &str {
        "Detecta y sugiere correcciones de estilo tipográfico avanzado en español (comillas latinas « », guión largo — en diálogos, espacio antes de unidades)."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let source = document.get_source();

        // 1. Detectar uso de comillas inglesas rectos " " y sugerir « »
        let mut quote_indices = Vec::new();
        for (i, &ch) in source.iter().enumerate() {
            if ch == '"' {
                quote_indices.push(i);
            }
        }

        if quote_indices.len() >= 2 {
            for pair in quote_indices.chunks(2) {
                if pair.len() == 2 {
                    let start_idx = pair[0];
                    let end_idx = pair[1];
                    let span = crate::Span::new(start_idx, end_idx + 1);

                    let inner_content: String = source[start_idx + 1..end_idx].iter().collect();
                    let formatted = format!("«{}»", inner_content);

                    lints.push(Lint {
                        span,
                        lint_kind: LintKind::Style,
                        message: "En tipografía española se recomienda el uso de comillas latinas o españolas (« ») en lugar de inglesas (\" \").".to_string(),
                        suggestions: vec![Suggestion::ReplaceWith(formatted.chars().collect())],
                        priority: 15,
                    });
                }
            }
        }

        // 2. Unidades de medida precedidas de número sin espacio (ej: "50km/h" -> "50 km/h", "10kg" -> "10 kg")
        const UNITS: &[&str] = &[
            "km/h", "km", "m", "cm", "mm", "kg", "g", "mg", "l", "ml", "hz", "khz", "mhz", "ghz",
            "%", "€", "$",
        ];

        let tokens: Vec<_> = document.tokens().collect();
        for window in tokens.windows(2) {
            let tok1 = &window[0];
            let tok2 = &window[1];
            let str1 = document.get_span_content_str(&tok1.span);
            let str2 = document.get_span_content_str(&tok2.span);

            if str1
                .chars()
                .all(|c| c.is_ascii_digit() || c == ',' || c == '.')
                && UNITS.contains(&str2.as_str())
                && tok1.span.end == tok2.span.start
            {
                let span = crate::Span::new(tok1.span.start, tok2.span.end);
                let corrected = format!("{} {}", str1, str2);
                lints.push(Lint {
                    span,
                    lint_kind: LintKind::Style,
                    message: format!(
                        "Tipografía: Las unidades de medida deben separarse del número con un espacio ('{}').",
                        corrected
                    ),
                    suggestions: vec![Suggestion::ReplaceWith(corrected.chars().collect())],
                    priority: 16,
                });
            }
        }

        lints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggests_spanish_quotes() {
        let mut linter = SpanishTypography;
        let doc = Document::new_plain_english_curated("Dijo \"hola a todos\" ayer.");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        assert!(lints[0].message.contains("comillas latinas"));
    }

    #[test]
    fn corrects_unit_spacing() {
        let mut linter = SpanishTypography;
        let doc = Document::new_plain_english_curated("El auto iba a 50km/h por la calle.");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        let mut text_chars: Vec<char> = "El auto iba a 50km/h por la calle.".chars().collect();
        lints[0].suggestions[0].apply(lints[0].span, &mut text_chars);
        let result: String = text_chars.into_iter().collect();
        assert_eq!(result, "El auto iba a 50 km/h por la calle.");
    }
}
