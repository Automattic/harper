//! *Vor und Nachteile*: the first half of a shortened compound needs its
//! hyphen, *Vor- und Nachteile*.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::{
        grammar::{
            noun_phrase::lowercase_of,
            prepositions::preposition_government,
            verbs::{looks_like_participle, zu_infix_parts},
        },
        spell::curated_german_dictionary,
    },
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::Dictionary,
};

/// Reports the missing hyphen (*Ergänzungsstrich*) on the first half of two
/// coordinated compounds that share their second part: *Vor und Nachteile* →
/// *Vor- und Nachteile*, *Ein und Ausgang*, *An und Verkauf*, *sich
/// ein und auszuloggen*. Duden, Amtliche Regelung § 98.
///
/// The evidence is the dictionary: the second word splits into a head and a
/// tail (*Nach* + *teile*), and the first word with that tail is a word of its
/// own (*Vorteile*). Only whole entries count, never a compound the checker
/// builds, which would accept almost any pair.
///
/// Two readings are left alone because they coordinate two complete words:
///
/// * a first word that is a noun with its own determiner — *das Haus und
///   Gartengeräte* sells a house and some tools;
/// * a lower-case second word that could open a clause of its own — *er kam
///   an und aufgeregt erzählte er* — so lower case is only read for an
///   infinitive with *zu* inside (*ein und auszuloggen*) or an adjective that
///   is no participle (*in und auswendig*).
#[derive(Default)]
pub struct GermanSuspendedHyphen;

impl GermanSuspendedHyphen {
    fn is_entry(word: &str) -> bool {
        let chars: Vec<char> = word.chars().collect();
        curated_german_dictionary().contains_word(&chars)
    }

    fn capitalized(token: &Token, document: &Document) -> bool {
        document
            .get_span_content(&token.span)
            .first()
            .is_some_and(|c| c.is_uppercase())
    }

    /// The tail of `second` that makes a word of its own behind `first`:
    /// *Vor*, *Nachteile* → *teile*.
    ///
    /// The tail has to be a word itself and at least four letters long. A
    /// suffix is not what two compounds share: *Partei* and *Regierung* end
    /// in *-ung* together with *Parteiung*, and *Rat* and *Kommission* would
    /// make *Ration*.
    fn shared_tail(first: &str, second: &str) -> Option<String> {
        let chars: Vec<char> = second.chars().collect();
        (2..chars.len().saturating_sub(3)).find_map(|split| {
            let tail: String = chars[split..].iter().collect();
            (Self::is_entry(&tail) && Self::is_entry(&format!("{first}{tail}"))).then_some(tail)
        })
    }

    /// A word spelled like a name or an acronym: *JavaScript*, *TI*.
    fn has_inner_capital(token: &Token, document: &Document) -> bool {
        document
            .get_span_content(&token.span)
            .iter()
            .skip(1)
            .any(|c| c.is_uppercase())
    }

    fn check(words: &[&Token], at: usize, document: &Document) -> Option<Lint> {
        let first = words[at];
        let conjunction = lowercase_of(words.get(at + 1)?, document);
        let second = words.get(at + 2)?;
        if !matches!(conjunction.as_str(), "und" | "oder" | "sowie")
            || !first.kind.is_word()
            || !second.kind.is_word()
        {
            return None;
        }
        let first_text = lowercase_of(first, document);
        let second_text = lowercase_of(second, document);
        if first_text.chars().count() < 2
            || second_text.chars().count() < 6
            || Self::has_inner_capital(first, document)
            || Self::has_inner_capital(second, document)
        {
            return None;
        }

        // *das Haus und Gartengeräte*: two noun phrases.
        let previous = at.checked_sub(1).map(|p| words[p]);
        if first.kind.is_noun()
            && previous.is_some_and(|p| p.kind.is_determiner() || p.kind.is_adjective())
        {
            return None;
        }

        let tail = if Self::capitalized(second, document) {
            // A compound noun pairs with a capitalized first half: *erzeugen
            // oder Bilder* is a verb and a noun.
            if !Self::capitalized(first, document) {
                return None;
            }
            Self::shared_tail(&first_text, &second_text)?
        } else if let Some((prefix, rest)) = zu_infix_parts(&second_text) {
            // *ein und auszuloggen*: both prefixes on one verb.
            if !Self::is_entry(&format!("{prefix}{rest}"))
                || !Self::is_entry(&format!("{first_text}{rest}"))
            {
                return None;
            }
            rest
        } else if second.kind.is_adjective()
            && !second.kind.is_verb()
            && !looks_like_participle(&second_text)
            // Only a preposition in front: *in und auswendig*, *ober und
            // untergärig*. An adjective or adverb is a word of its own —
            // *nichts wert oder sinnvoll*.
            && preposition_government(&first_text).is_some()
        {
            Self::shared_tail(&first_text, &second_text)?
        } else {
            return None;
        };

        let original = document.get_span_content(&first.span);
        let mut fixed = original.to_vec();
        fixed.push('-');
        let fixed_text: String = fixed.iter().collect();
        let second_original = document.get_span_content_str(&second.span);
        Some(Lint {
            span: first.span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![Suggestion::ReplaceWith(fixed)],
            message: format!(
                "Beide Wörter teilen sich »-{tail}«. Der ausgelassene Teil wird durch einen \
                 Ergänzungsstrich ersetzt: »{fixed_text} {conjunction} {second_original}«."
            ),
            priority: 31,
        })
    }
}

impl Linter for GermanSuspendedHyphen {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();
            for at in 0..words.len() {
                if matches!(words[at].kind, TokenKind::Word(_))
                    && let Some(lint) = Self::check(&words, at, document)
                {
                    lints.push(lint);
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Ergänzt den Ergänzungsstrich bei verkürzten Zusammensetzungen: »Vor- und Nachteile«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSuspendedHyphen;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn flagged(text: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanSuspendedHyphen
            .lint(&document)
            .into_iter()
            .flat_map(|lint| lint.suggestions)
            .map(|suggestion| suggestion.to_string())
            .collect()
    }

    #[test]
    fn the_first_half_needs_its_hyphen() {
        for (text, fixed) in [
            ("Die Vor und Nachteile sind klar.", "Vor-"),
            ("Die Ein und Ausgänge wurden überwacht.", "Ein-"),
            ("Auf dem Schild stand An und Verkauf.", "An-"),
            ("Im In und Ausland bekannt.", "In-"),
            ("Ich versuchte, mich ein und auszuloggen.", "ein-"),
            ("Er kennt das in und auswendig.", "in-"),
        ] {
            assert_eq!(
                flagged(text),
                [format!("Replace with: “{fixed}”")],
                "{text}"
            );
        }
    }

    #[test]
    fn two_whole_words_are_quiet() {
        for text in [
            "Die Vor- und Nachteile sind klar.",
            "Er verkauft das Haus und Gartengeräte.",
            "Er ging ein und aus.",
            "Er kam an und aufgeregt erzählte er alles.",
            "Kaffee und Kuchen stehen bereit.",
            "Eltern und Großeltern kamen.",
            "Papier und Bleistift liegen bereit.",
            "Krieg und Frieden ist ein Roman.",
            "Glück und Unglück liegen nah beieinander.",
            "Das ist ein und derselbe Mann.",
            "Mein Ein und Alles.",
            "Das Auf und Ab der Börse.",
            "Die Partei und Regierung tagten.",
            "Das ist überhaupt nichts wert oder sinnvoll.",
            "Rat und Kommission stimmten zu.",
            "Er arbeitet mit Java und JavaScript.",
            "Sie können Grafiken erzeugen oder Bilder laden.",
        ] {
            assert!(flagged(text).is_empty(), "{text}: {:?}", flagged(text));
        }
    }
}
