//! *Sie hat gegangen*: the perfect of a verb of motion or change of state
//! takes *sein*.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::noun_phrase::lowercase_of,
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Past participles whose perfect is formed with *sein* and never with
/// *haben*: *ich bin gegangen*, *er ist gestorben*, *es ist passiert*.
///
/// Only verbs with no transitive use that would take *haben*. *gefahren*,
/// *geflogen* and *geritten* are left out (*Er hat das Auto gefahren*), as is
/// *gefallen* (*das hat mir gefallen*) and every reflexive (*sich umgezogen*).
const SEIN_PARTICIPLES: &[&str] = &[
    "gegangen",
    "gekommen",
    "gestorben",
    "geblieben",
    "gewesen",
    "geworden",
    "passiert",
    "geschehen",
    "gelungen",
    "gereist",
    "gewachsen",
    "gestiegen",
    "gesunken",
    "geflohen",
    "geschlüpft",
    "aufgestanden",
    "eingeschlafen",
    "aufgewacht",
    "angekommen",
    "abgereist",
    "ausgestiegen",
    "eingestiegen",
    "umgestiegen",
    "losgegangen",
    "weggegangen",
    "zurückgekommen",
    "mitgekommen",
    "hingegangen",
    "hingefallen",
    "umgefallen",
    "verschwunden",
    "erschienen",
    "entstanden",
    "aufgetaucht",
    "zusammengebrochen",
];

/// Forms of *haben* and the *sein* form that replaces each.
const HABEN_TO_SEIN: &[(&str, &str)] = &[
    ("habe", "bin"),
    ("hast", "bist"),
    ("hat", "ist"),
    ("haben", "sind"),
    ("habt", "seid"),
    ("hatte", "war"),
    ("hattest", "warst"),
    ("hatten", "waren"),
    ("hattet", "wart"),
    ("hätte", "wäre"),
    ("hättest", "wärst"),
    ("hätten", "wären"),
    ("hättet", "wärt"),
];

/// Words that show the participle is not the main verb of a perfect with the
/// *haben* in front: a modal perfect (*hat gehen müssen*), a passive
/// (*hat ... gekommen werden*), a reflexive.
const BREAKERS: &[&str] = &[
    "werden", "worden", "müssen", "können", "dürfen", "sollen", "wollen", "lassen", "sich",
];

/// Forms of *sein*. A clause that already has one is a coordination or a
/// modal construction the simple pattern does not describe: *Sie haben
/// gesungen und sind eingeschlafen*, *Er wäre dafür zu haben gewesen*.
const SEIN_FORMS: &[&str] = &[
    "bin", "bist", "ist", "sind", "seid", "war", "warst", "waren", "wart", "wäre", "wärst",
    "wären", "wärt", "sei", "seien", "gewesen",
];

/// Requires *sein* in the perfect of a verb of motion or change of state:
/// *Sie hat gegangen* → *Sie ist gegangen*, *weil er gestorben hat* → *weil er
/// gestorben ist*.
///
/// The *haben* form and the participle have to be in the same clause, with the
/// participle at its end (main clause) or directly in front of the auxiliary
/// (subordinate clause). Only participles that never take *haben* are listed.
#[derive(Default)]
pub struct GermanPerfectAuxiliary;

impl GermanPerfectAuxiliary {
    fn sein_form(word: &str) -> Option<&'static str> {
        HABEN_TO_SEIN
            .iter()
            .find(|(haben, _)| *haben == word)
            .map(|(_, sein)| *sein)
    }

    fn report(auxiliary: &Token, sein: &str, participle: &str, document: &Document) -> Lint {
        let original = document.get_span_content(&auxiliary.span);
        Lint {
            span: auxiliary.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case(
                sein.chars().collect(),
                original,
            )],
            priority: 31,
            message: format!("»{participle}« bildet das Perfekt mit »sein«, nicht mit »haben«."),
        }
    }
}

impl Linter for GermanPerfectAuxiliary {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();

            // Split into clauses at punctuation.
            let mut start = 0;
            while start < words.len() {
                let end = (start..words.len())
                    .find(|&at| matches!(words[at].kind, TokenKind::Punctuation(_)))
                    .unwrap_or(words.len());
                let clause = &words[start..end];
                start = end + 1;

                let lower: Vec<String> = clause
                    .iter()
                    .map(|token| lowercase_of(token, document))
                    .collect();
                if lower.iter().any(|word| {
                    BREAKERS.contains(&word.as_str()) || SEIN_FORMS.contains(&word.as_str())
                }) {
                    continue;
                }
                // *passiert* is also transitive — *nachdem das Gesetz das
                // Unterhaus passiert hatte* — so it is only read with no noun
                // in the clause that could be its object.
                let has_noun = clause.iter().skip(1).any(|token| {
                    document
                        .get_span_content(&token.span)
                        .first()
                        .is_some_and(|c| c.is_uppercase())
                });
                let participle_ok = |word: &str| {
                    SEIN_PARTICIPLES.contains(&word) && !(word == "passiert" && has_noun)
                };
                let Some(last) = clause.len().checked_sub(1) else {
                    continue;
                };

                // Main clause: auxiliary somewhere, participle last.
                if participle_ok(&lower[last])
                    && let Some((at, sein)) = lower[..last]
                        .iter()
                        .enumerate()
                        .filter(|(at, _)| *at == 0 || lower[at - 1] != "zu")
                        .find_map(|(at, word)| Self::sein_form(word).map(|sein| (at, sein)))
                {
                    lints.push(Self::report(clause[at], sein, &lower[last], document));
                    continue;
                }

                // Subordinate clause: participle directly before a final auxiliary.
                if last >= 1
                    && participle_ok(&lower[last - 1])
                    && let Some(sein) = Self::sein_form(&lower[last])
                {
                    lints.push(Self::report(clause[last], sein, &lower[last - 1], document));
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Verlangt »sein« im Perfekt von Verben der Bewegung und Zustandsänderung (»sie ist gegangen«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanPerfectAuxiliary;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn fixes(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanPerfectAuxiliary
            .lint(&document)
            .into_iter()
            .flat_map(|lint| lint.suggestions)
            .map(|suggestion| suggestion.to_string())
            .collect()
    }

    #[test]
    fn haben_with_a_sein_participle_is_reported() {
        assert_eq!(fixes("Sie hat gegangen."), ["Replace with: “ist”"]);
        assert_eq!(
            fixes("Ich habe gestern nach Hause gekommen."),
            ["Replace with: “bin”"]
        );
        assert_eq!(
            fixes("Wir haben lange in Berlin geblieben."),
            ["Replace with: “sind”"]
        );
        assert_eq!(fixes("Was hat passiert?"), ["Replace with: “ist”"]);
        assert_eq!(
            fixes("Hast du gut eingeschlafen?"),
            ["Replace with: “Bist”"]
        );
    }

    #[test]
    fn a_subordinate_clause_is_checked_at_its_end() {
        assert_eq!(
            fixes("Ich weiß, dass er gestorben hat."),
            ["Replace with: “ist”"]
        );
        assert_eq!(
            fixes("Weil wir zu spät angekommen haben, war alles vorbei."),
            ["Replace with: “sind”"]
        );
    }

    #[test]
    fn the_correct_auxiliary_is_quiet() {
        for text in [
            "Sie ist gegangen.",
            "Ich bin nach Hause gekommen.",
            "Er hat das Auto gefahren.",
            "Das hat mir gefallen.",
            "Er hat gehen müssen.",
            "Sie hat sich umgezogen.",
            "Ich habe ihn kommen sehen.",
            "Ich habe keine Lust, nach Hause zu gehen.",
            "Er hat einen Brief geschrieben.",
            "Wir hatten Glück, dass niemand gekommen ist.",
            "Sie haben gesungen und sind eingeschlafen.",
            "Er wäre dafür zu haben gewesen.",
            "Nachdem das Gesetz das Unterhaus passiert hatte, trat es in Kraft.",
        ] {
            assert!(fixes(text).is_empty(), "{text}");
        }
    }
}
