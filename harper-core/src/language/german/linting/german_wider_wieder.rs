use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{TokenStringExt, document::Document};

/// Verbs and nouns where only the prefix `wider-` ("against") is correct.
///
/// Matching is by prefix of the remainder rather than by whole word, so one
/// entry covers the whole paradigm: `sprech` catches "widersprechen",
/// "widerspricht" and "widersprechende".
const WIDER_ONLY: &[&str] = &[
    "fahr", "fuhr", "hall", "leg", "ruf", "setz", "sinn", "spieg", "sproch", "sprech", "sprich",
    "spruch", "stand", "steh", "streb", "willen",
];

/// Verbs and nouns where only the prefix `wieder-` ("again") is correct.
const WIEDER_ONLY: &[&str] = &[
    "aufbau", "aufnahm", "beleb", "entdeck", "eröffn", "erkann", "erkenn", "gutmach", "herstell",
    "hol", "sah", "seh", "sieh", "vereinig", "verwend", "wahl", "wähl",
];

/// Catches the `wider-` / `wieder-` prefix mix-up.
///
/// The two are unrelated — `wider` means "against", `wieder` means "again" —
/// but they differ by one letter and are among the most commonly confused
/// prefixes in written German. The spell checker cannot catch either direction
/// on its own: the compound splitter happily reads "wiederspiegeln" as
/// `wieder` + `spiegeln`, both of which are real words.
///
/// Only the two closed lists above are consulted, so genuinely ambiguous stems
/// ("wiederholen" the verb vs. "widerhallen") are never guessed at.
#[derive(Default)]
pub struct GermanWiderWieder;

impl GermanWiderWieder {
    /// The corrected spelling of `word`, if it carries the wrong prefix.
    fn correction(word: &[char]) -> Option<String> {
        let lower: String = word
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect::<String>();

        // Check `wieder` first: it is the longer prefix, and every `wieder`
        // also starts with `wied`, never with `wider`.
        let (right, rest, stems) = if let Some(rest) = lower.strip_prefix("wieder") {
            ("wider", rest, WIDER_ONLY)
        } else if let Some(rest) = lower.strip_prefix("wider") {
            ("wieder", rest, WIEDER_ONLY)
        } else {
            return None;
        };

        if rest.is_empty() || !stems.iter().any(|stem| rest.starts_with(stem)) {
            return None;
        }

        let mut fixed = String::with_capacity(lower.len());
        fixed.push_str(right);
        fixed.push_str(rest);

        // Preserve a leading capital: the word may open a sentence or be a
        // nominalisation ("Widerspruch", "Wiederholung").
        if word.first().is_some_and(|c| c.is_uppercase()) {
            let mut chars = fixed.chars();
            if let Some(first) = chars.next() {
                fixed = first.to_uppercase().chain(chars).collect();
            }
        }

        Some(fixed)
    }
}

impl Linter for GermanWiderWieder {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let chars = document.get_span_content(&word.span);
            let Some(fixed) = Self::correction(chars) else {
                continue;
            };

            let original: String = chars.iter().collect();
            lints.push(Lint {
                span: word.span,
                lint_kind: LintKind::Spelling,
                suggestions: vec![Suggestion::ReplaceWith(fixed.chars().collect())],
                message: format!(
                    "»{original}« verwechselt die Vorsilben: »wider« heißt »gegen«, \
                     »wieder« heißt »noch einmal«. Gemeint ist »{fixed}«."
                ),
                priority: 19,
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "Unterscheidet die Vorsilben »wider« (gegen) und »wieder« (noch einmal)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanWiderWieder;

    fn correction(word: &str) -> Option<String> {
        let chars: Vec<char> = word.chars().collect();
        GermanWiderWieder::correction(&chars)
    }

    #[test]
    fn corrects_wieder_to_wider() {
        for (wrong, right) in [
            ("wiederspiegeln", "widerspiegeln"),
            ("wiederspiegelt", "widerspiegelt"),
            ("wiedersprechen", "widersprechen"),
            ("wiederspricht", "widerspricht"),
            ("Wiederspruch", "Widerspruch"),
            ("wiederstehen", "widerstehen"),
            ("Wiederstand", "Widerstand"),
            ("wiederrufen", "widerrufen"),
            ("wiederlegen", "widerlegen"),
        ] {
            assert_eq!(correction(wrong).as_deref(), Some(right), "{wrong}");
        }
    }

    #[test]
    fn corrects_wider_to_wieder() {
        for (wrong, right) in [
            ("widerholen", "wiederholen"),
            ("Widerholung", "Wiederholung"),
            ("widersehen", "wiedersehen"),
            ("Widersehen", "Wiedersehen"),
            ("widerherstellen", "wiederherstellen"),
            ("Widervereinigung", "Wiedervereinigung"),
            ("widerbeleben", "wiederbeleben"),
            ("Widerwahl", "Wiederwahl"),
        ] {
            assert_eq!(correction(wrong).as_deref(), Some(right), "{wrong}");
        }
    }

    /// Both prefixes are real words with real paradigms; only the closed stem
    /// lists may be corrected.
    #[test]
    fn leaves_correct_and_ambiguous_words_alone() {
        for word in [
            "wider",
            "wieder",
            "wiederum",
            "widerlich",
            "widerwärtig",
            "Widerspruch",
            "widersprechen",
            "widerspiegeln",
            "Widerstand",
            "wiederholen",
            "Wiederholung",
            "Wiedersehen",
            "wiederkommen",
            "Wiederaufbau",
            "Widerrede",
            "Kleider",
            "Glieder",
        ] {
            assert_eq!(correction(word), None, "{word} should be left alone");
        }
    }
}
