use crate::{
    Document, TokenStringExt,
    linting::{Lint, LintKind, Linter, Suggestion},
};
use hashbrown::HashMap;

/// Motor de reglas automáticas de LanguageTool cargadas desde los patrones de grammar.xml / replace.txt
pub struct LanguageToolRules {
    replace_map: HashMap<String, (String, String)>,
}

impl Default for LanguageToolRules {
    fn default() -> Self {
        let mut replace_map = HashMap::new();

        // 1. Cargar parejas fijas de sustitución
        const REPLACEMENTS: &[(&str, &str, &str)] = &[
            (
                "subir arriba",
                "subir",
                "Redundancia: 'subir' ya indica dirección hacia arriba.",
            ),
            (
                "bajar abajo",
                "bajar",
                "Redundancia: 'bajar' ya indica dirección hacia abajo.",
            ),
            (
                "entrar adentro",
                "entrar",
                "Redundancia: 'entrar' ya indica dirección hacia el interior.",
            ),
            (
                "salir afuera",
                "salir",
                "Redundancia: 'salir' ya indica dirección hacia el exterior.",
            ),
            (
                "lapso de tiempo",
                "lapso",
                "Redundancia: todo lapso es de tiempo.",
            ),
            (
                "persona humana",
                "persona",
                "Redundancia: toda persona es humana por definición.",
            ),
            (
                "regalo gratuito",
                "regalo",
                "Redundancia: todo regalo es gratuito.",
            ),
            (
                "volar por el aire",
                "volar",
                "Redundancia: volar implica desplazarse por el aire.",
            ),
            (
                "de acuerdo a",
                "de acuerdo con",
                "La RAE recomienda la locución 'de acuerdo con'.",
            ),
            (
                "en relacion a",
                "en relación con",
                "La RAE recomienda la locución 'en relación con'.",
            ),
            (
                "en relacion con",
                "en relación con",
                "Falta tilde en 'relación'.",
            ),
            (
                "en funcion de",
                "en función de",
                "Falta tilde en 'función'.",
            ),
            (
                "hacer click",
                "hacer clic",
                "En español la forma adaptada es 'clic' (sin 'k').",
            ),
            ("spanglish", "español", "Anglicismo."),
            (
                "sponsor",
                "patrocinador",
                "Anglicismo no adaptado. Usa 'patrocinador'.",
            ),
            ("link", "enlace", "Anglicismo. Se recomienda usar 'enlace'."),
            (
                "post",
                "publicación",
                "Anglicismo. Se recomienda usar 'publicación'.",
            ),
        ];

        for (from, to, msg) in REPLACEMENTS {
            replace_map.insert(from.to_lowercase(), (to.to_string(), msg.to_string()));
        }

        // 2. Parsear dinámicamente replace.txt descargado de LanguageTool
        if let Ok(content) = std::fs::read_to_string(
            "external_resources/languagetool/languagetool-language-modules/es/src/main/resources/org/languagetool/rules/es/replace.txt",
        ) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((from_part, to_part)) = line.split_once('=') {
                    let first_to = to_part.split('|').next().unwrap_or(to_part).trim();
                    for wrong in from_part.split('|') {
                        let wrong = wrong.trim().to_lowercase();
                        if !wrong.is_empty() && !replace_map.contains_key(&wrong) {
                            replace_map.insert(
                                wrong.clone(),
                                (
                                    first_to.to_string(),
                                    format!(
                                        "Sustitución recomendada por LanguageTool: '{}' -> '{}'.",
                                        wrong, first_to
                                    ),
                                ),
                            );
                        }
                    }
                }
            }
        }

        // 3. Parsear dinámicamente compounds.txt (palabras compuestas sin guion)
        if let Ok(content) = std::fs::read_to_string(
            "external_resources/languagetool/languagetool-language-modules/es/src/main/resources/org/languagetool/resource/es/compounds.txt",
        ) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let clean_line =
                    line.trim_matches(|c| c == '+' || c == '*' || c == '?' || c == '$');
                if clean_line.contains('-') {
                    let joined = clean_line.replace('-', "");
                    replace_map.insert(
                        clean_line.to_lowercase(),
                        (
                            joined.clone(),
                            format!("Palabra compuesta: en español se recomienda escribirla sin guion ('{}').", joined),
                        ),
                    );
                }
            }
        }

        Self { replace_map }
    }
}

impl Linter for LanguageToolRules {
    fn description(&self) -> &str {
        "Ejecuta el catálogo de reglas de sustitución, confusiones y ortotipografía importado de LanguageTool para español."
    }

    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            let word_indices: Vec<usize> = chunk.iter_word_indices().collect();

            // 1. Coincidencias de 1 palabra (de replace.txt)
            for &idx in &word_indices {
                let tok = &chunk[idx];
                let word = document.get_span_content_str(&tok.span).to_lowercase();
                if let Some((fix, msg)) = self.replace_map.get(&word) {
                    lints.push(Lint {
                        span: tok.span,
                        lint_kind: LintKind::Style,
                        message: msg.clone(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            fix.chars().collect(),
                            document.get_span_content(&tok.span),
                        )],
                        priority: 23,
                    });
                }
            }

            // 2. Coincidencias de 2 palabras
            for window in word_indices.windows(2) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];

                let phrase = format!(
                    "{} {}",
                    document
                        .get_span_content_str(&first_tok.span)
                        .to_lowercase(),
                    document
                        .get_span_content_str(&second_tok.span)
                        .to_lowercase()
                );

                if let Some((fix, msg)) = self.replace_map.get(&phrase) {
                    let double_span = crate::Span::new(first_tok.span.start, second_tok.span.end);
                    lints.push(Lint {
                        span: double_span,
                        lint_kind: LintKind::Style,
                        message: msg.clone(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            fix.chars().collect(),
                            document.get_span_content(&double_span),
                        )],
                        priority: 23,
                    });
                }
            }

            // 3. Coincidencias de 3 palabras
            for window in word_indices.windows(3) {
                let first_tok = &chunk[window[0]];
                let second_tok = &chunk[window[1]];
                let third_tok = &chunk[window[2]];

                let phrase = format!(
                    "{} {} {}",
                    document
                        .get_span_content_str(&first_tok.span)
                        .to_lowercase(),
                    document
                        .get_span_content_str(&second_tok.span)
                        .to_lowercase(),
                    document
                        .get_span_content_str(&third_tok.span)
                        .to_lowercase()
                );

                if let Some((fix, msg)) = self.replace_map.get(&phrase) {
                    let triple_span = crate::Span::new(first_tok.span.start, third_tok.span.end);
                    lints.push(Lint {
                        span: triple_span,
                        lint_kind: LintKind::Style,
                        message: msg.clone(),
                        suggestions: vec![Suggestion::replace_with_match_case(
                            fix.chars().collect(),
                            document.get_span_content(&triple_span),
                        )],
                        priority: 23,
                    });
                }
            }
        }

        lints
    }
}
