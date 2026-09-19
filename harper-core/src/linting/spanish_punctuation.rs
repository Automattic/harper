use crate::{
    Document,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar errores de puntuación y espaciado en español (RAE):
/// 1. Puntuación duplicada o consecutiva (,, o ;; o ..)
/// 2. Espacio indebido antes de coma/punto y coma/punto (ej: "hola ,")
/// 3. Falta de espacio después de coma/punto y coma (ej: "hola,mundo")
#[derive(Debug, Default)]
pub struct SpanishPunctuation;

impl Linter for SpanishPunctuation {
    fn description(&self) -> &str {
        "Detecta errores de espaciado y uso en signos de puntuación en español (comas duplicadas ',,', espacio antes de signo, falta de espacio posterior)."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        let vec_tokens: Vec<_> = document.tokens().collect();
        if vec_tokens.is_empty() {
            return lints;
        }

        for i in 0..vec_tokens.len() {
            let tok = &vec_tokens[i];
            let tok_str = document.get_span_content_str(&tok.span);

            // 1. Puntuación duplicada o adyacente (ej: ",," o ";;" o ",;" o ";," o ",:")
            let is_punc = |s: &str| s == "," || s == ";" || s == "." || s == ":";
            if tok_str == ",,"
                || tok_str == ";;"
                || tok_str == ".."
                || tok_str.contains(",,")
                || tok_str.contains(";;")
                || tok_str == ",;"
                || tok_str == ";,"
                || tok_str == ",:"
            {
                let single = if tok_str.contains(',') {
                    ","
                } else if tok_str.contains(';') {
                    ";"
                } else if tok_str.contains(':') {
                    ":"
                } else {
                    "."
                };
                lints.push(Lint {
                    span: tok.span,
                    lint_kind: LintKind::Punctuation,
                    message: format!("Signos de puntuación adyacentes incompatibles ('{}'). Conserva únicamente uno de los signos ('{}').", tok_str, single),
                    suggestions: vec![Suggestion::ReplaceWith(single.chars().collect())],
                    priority: 25,
                });
            } else if is_punc(&tok_str) && i + 1 < vec_tokens.len() {
                let next_tok = &vec_tokens[i + 1];
                let next_str = document.get_span_content_str(&next_tok.span);
                if is_punc(&next_str) {
                    let double_span = crate::Span::new(tok.span.start, next_tok.span.end);
                    lints.push(Lint {
                        span: double_span,
                        lint_kind: LintKind::Punctuation,
                        message: format!("Puntuación adyacente o redundante '{}{}'. Conserva únicamente el primer signo ('{}').", tok_str, next_str, tok_str),
                        suggestions: vec![Suggestion::ReplaceWith(tok_str.chars().collect())],
                        priority: 25,
                    });
                }
            }

            // 2. Espacio indebido antes de signo de puntuación (ej: "palabra ," o "con :faltas")
            if is_punc(&tok_str) && i > 0 {
                let prev_tok = &vec_tokens[i - 1];
                if prev_tok.kind.is_whitespace() {
                    lints.push(Lint {
                        span: prev_tok.span,
                        lint_kind: LintKind::Punctuation,
                        message: format!("No debe haber espacio antes del signo '{}'.", tok_str),
                        suggestions: vec![Suggestion::Remove],
                        priority: 26,
                    });
                }
            }

            // 3. Falta de espacio después de signo (ej: "hola,mundo" o "con :faltas")
            if is_punc(&tok_str) && i + 1 < vec_tokens.len() {
                let next_tok = &vec_tokens[i + 1];
                if !next_tok.kind.is_whitespace()
                    && !next_tok.kind.is_newline()
                    && next_tok.kind.is_word()
                {
                    lints.push(Lint {
                        span: tok.span,
                        lint_kind: LintKind::Punctuation,
                        message: format!(
                            "Se debe incluir un espacio después del signo '{}'.",
                            tok_str
                        ),
                        suggestions: vec![Suggestion::ReplaceWith(
                            format!("{} ", tok_str).chars().collect(),
                        )],
                        priority: 27,
                    });
                }
            }
        }

        lints
    }
}
