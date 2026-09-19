use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Regla para detectar infinitivos no conjugados tras pronombres personales o en oraciones con tiempo verbal definido.
/// Ejemplo: "nosotros venir contentos" -> "nosotros vinimos / venimos contentos".
/// Ejemplo: "yo comer pan" -> "yo comí / como pan".
#[derive(Debug, Default)]
pub struct SpanishVerbConjugation;

impl Linter for SpanishVerbConjugation {
    fn description(&self) -> &str {
        "Detecta uso incorrecto de verbos en infinitivo tras pronombres personales (ej. 'nosotros venir' por 'nosotros venimos / vinimos')."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        const SUBJECT_PRONOUNS_PLURAL_1: &[&str] = &["nosotros", "nosotras"];
        const SUBJECT_PRONOUNS_PLURAL_3: &[&str] = &["ellos", "ellas", "ustedes"];
        const SUBJECT_PRONOUNS_SINGULAR_1: &[&str] = &["yo"];
        const SUBJECT_PRONOUNS_SINGULAR_2: &[&str] = &["tú", "tu", "usted"];

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            for window in word_indices.windows(2) {
                let pron_tok = &chunk[window[0]];
                let verb_tok = &chunk[window[1]];

                let pron_str = document.get_span_content_str(&pron_tok.span).to_lowercase();
                let verb_str = document.get_span_content_str(&verb_tok.span).to_lowercase();

                // Detectar verbo en infinitivo (-ar, -er, -ir)
                let is_infinitive = (verb_str.ends_with("ar") || verb_str.ends_with("er") || verb_str.ends_with("ir")) && verb_str.len() > 3;

                if is_infinitive {
                    let mut suggestions = Vec::new();

                    if SUBJECT_PRONOUNS_PLURAL_1.contains(&pron_str.as_str()) {
                        // Nosotros -> vinimos / venimos
                        let stem = &verb_str[..verb_str.len() - 2];
                        let past = match verb_str.as_str() {
                            "venir" => "vinimos".to_string(),
                            "hacer" => "hicimos".to_string(),
                            "decir" => "dijimos".to_string(),
                            "ir" | "ser" => "fuimos".to_string(),
                            "tener" => "tuvimos".to_string(),
                            "estar" => "estuvimos".to_string(),
                            "poner" => "pusimos".to_string(),
                            "poder" => "pudimos".to_string(),
                            "querer" => "quisimos".to_string(),
                            _ if verb_str.ends_with("ar") => format!("{}amos", stem),
                            _ => format!("{}imos", stem),
                        };
                        let pres = if verb_str.ends_with("ar") { format!("{}amos", stem) } else { format!("{}emos", stem) };
                        suggestions.push(Suggestion::replace_with_match_case(past.chars().collect(), document.get_span_content(&verb_tok.span)));
                        if pres != past {
                            suggestions.push(Suggestion::replace_with_match_case(pres.chars().collect(), document.get_span_content(&verb_tok.span)));
                        }
                    } else if SUBJECT_PRONOUNS_PLURAL_3.contains(&pron_str.as_str()) {
                        let stem = &verb_str[..verb_str.len() - 2];
                        let past = if verb_str.ends_with("ar") { format!("{}aron", stem) } else { format!("{}ieron", stem) };
                        suggestions.push(Suggestion::replace_with_match_case(past.chars().collect(), document.get_span_content(&verb_tok.span)));
                    } else if SUBJECT_PRONOUNS_SINGULAR_1.contains(&pron_str.as_str()) {
                        let stem = &verb_str[..verb_str.len() - 2];
                        let pres = if verb_str.ends_with("ar") { format!("{}o", stem) } else { format!("{}o", stem) };
                        suggestions.push(Suggestion::replace_with_match_case(pres.chars().collect(), document.get_span_content(&verb_tok.span)));
                    }

                    if !suggestions.is_empty() {
                        lints.push(Lint {
                            span: verb_tok.span,
                            lint_kind: LintKind::Grammar,
                            message: format!(
                                "Conjugación verbal: El verbo en infinitivo '{}' debe conjugarse según el pronombre '{}'.",
                                verb_str, pron_str
                            ),
                            suggestions,
                            priority: 36,
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
    fn detects_nosotros_venir() {
        let mut linter = SpanishVerbConjugation;
        let doc = Document::new_plain_english_curated("nosotros venir contentos");
        let lints = linter.lint(&doc);
        assert_eq!(lints.len(), 1);
        assert!(lints[0].message.contains("Conjugación verbal"));
    }
}
