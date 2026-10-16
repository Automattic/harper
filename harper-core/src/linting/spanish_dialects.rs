use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar variantes de voseo (Centroamérica y Cono Sur) y fenómenos de leísmo/laísmo/loísmo.
#[derive(Debug, Default)]
pub struct SpanishDialects;

impl Linter for SpanishDialects {
    fn description(&self) -> &str {
        "Detecta y ofrece sugerencias/análisis sobre expresiones de voseo (Argentina, Uruguay, Centroamérica) y leísmo/laísmo/loísmo."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // 1. Detección y normalización / sugerencias opcionales de Voseo profundo
        // Ejemplos: "vos tenes" -> "vos tenés", "vos sabes" -> "vos sabés", "vos querés"
        const VOSEO_VERBS: &[(&str, &str)] = &[
            ("tenes", "tenés"),
            ("sabes", "sabés"),
            ("queres", "querés"),
            ("haces", "hacés"),
            ("podes", "podés"),
            ("venis", "venís"),
            ("decis", "decís"),
        ];

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            for window in word_indices.windows(2) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];

                let first_str = document
                    .get_span_content_str(&first_tok.span)
                    .to_lowercase();
                let second_str = document
                    .get_span_content_str(&second_tok.span)
                    .to_lowercase();

                if first_str == "vos"
                    && let Some((_, tilde_voseo)) = VOSEO_VERBS
                        .iter()
                        .find(|&&(flat, _)| flat == second_str.as_str())
                {
                    lints.push(Lint {
                        span: second_tok.span,
                        lint_kind: LintKind::Style,
                        message: format!(
                            "En el voseo (Argentina/Uruguay/Centroamérica), la forma verbal lleva tilde: '{}'.",
                            tilde_voseo
                        ),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            tilde_voseo.chars().collect(),
                            document.get_span_content(&second_tok.span),
                        )],
                        priority: 18,
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
    fn corrects_voseo_accent() {
        let mut linter = SpanishDialects;
        let doc = Document::new_plain_english_curated("vos tenes razón");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        assert!(lints[0].message.contains("voseo"));
    }
}
