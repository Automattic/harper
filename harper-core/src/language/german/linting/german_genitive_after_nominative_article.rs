use std::sync::Arc;

use crate::{
    Span, Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::spell::curated_german_dictionary,
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::{Dictionary, FstDictionary},
};

/// Articles that never introduce a genitive, paired with the genitive article
/// that would make the same noun correct.
const ARTICLES: &[(&str, &str)] = &[("das", "des"), ("ein", "eines"), ("kein", "keines")];

/// Endings of nouns that are always feminine, and so never take a genitive `-s`.
const FEMININE_SUFFIXES: &[&str] = &["ung", "heit", "keit", "schaft", "ität", "ion", "ik", "ur"];

/// Words that end like a genitive and are nominatives in their own right: a
/// first name (`Johannes`) and a Latin loan (`Regens`). The dictionary lists
/// both as ordinary nouns, so nothing but the word itself tells them apart.
const NOMINATIVES_IN_S: &[&str] = &["Johannes", "Regens"];

/// Shortest noun the genitive ending is stripped from. It keeps `Ass` from
/// being read as the genitive of `As`, and `Gas` of `Ga`.
const MIN_STEM_LENGTH: usize = 4;

/// Catches a genitive ending on a noun that follows a nominative article:
/// *"Wenn **das Programms** nicht startet"* — the noun has to be *Programm*,
/// or the article *des*.
///
/// The dictionary carries case for no noun, and gender for few, so the rule
/// cannot ask whether a word is genitive. It asks the reverse question: is this
/// word a noun plus `-s` or `-es`? Behind `das`, `ein` and `kein` such a word
/// can only be a slip, because those articles never introduce a genitive.
///
/// Three things look like the pattern and are not:
///
/// * *"ein **Gutes**"* — a nominalized adjective. The dictionary reads `Gutes`
///   as an adjective form too, and a real genitive has no adjective reading.
/// * *"das **Ass**"* — the stem `As` is too short to be the noun.
/// * *"das **Volks-**wagen"* — a hyphen means the word is a compound element.
pub struct GermanGenitiveAfterNominativeArticle {
    dictionary: Arc<FstDictionary>,
}

impl Default for GermanGenitiveAfterNominativeArticle {
    fn default() -> Self {
        Self::new()
    }
}

impl GermanGenitiveAfterNominativeArticle {
    /// The stems are looked up in the base dictionary, not in a compound-aware
    /// one. A compound-aware dictionary accepts `Jagdhau` as *Jagd* + *Hau*, and
    /// then reads every `Jagdhaus` as the genitive of a word that does not exist.
    pub fn new() -> Self {
        Self {
            dictionary: curated_german_dictionary(),
        }
    }

    /// The genitive form of `word`, if `word` is one of the articles.
    fn genitive_article(word: &str) -> Option<&'static str> {
        let lower = word.to_lowercase();
        ARTICLES
            .iter()
            .find(|(article, _)| *article == lower)
            .map(|(_, genitive)| *genitive)
    }

    /// Could `stem` + `ending` be a genitive? A feminine noun has no genitive
    /// ending, and `-s` after a sibilant is not one either: `Genuss` is not the
    /// genitive of `Genus`, and `Regens` is a word of its own.
    fn can_take_genitive_ending(ending: &str, stem: &str) -> bool {
        if stem.chars().count() < MIN_STEM_LENGTH {
            return false;
        }
        if FEMININE_SUFFIXES
            .iter()
            .any(|suffix| stem.ends_with(suffix))
        {
            return false;
        }
        // A stem on a vowel takes `-s` as a plural (`Autos`, `Andreas` from a
        // first name), not as a genitive.
        if stem.ends_with(['a', 'e', 'i', 'o', 'u', 'ä', 'ö', 'ü']) {
            return false;
        }
        ending == "es" || !stem.ends_with(['s', 'ß', 'x', 'z'])
    }

    /// The noun `word` would be the genitive of.
    fn nominative_stem(&self, word: &str) -> Option<String> {
        if !word.chars().next().is_some_and(char::is_uppercase) {
            return None;
        }

        if NOMINATIVES_IN_S.contains(&word) {
            return None;
        }

        // A nominalized adjective (`Gutes`): the dictionary reads the word as an
        // adjective form too, and a real genitive has no such reading. Word ids
        // ignore case, so this looks the word up as written.
        let chars: Vec<char> = word.chars().collect();
        let Some(word_metadata) = self.dictionary.get_word_metadata(&chars) else {
            // Not a German word at all: `Queens`, a borrowed name.
            return None;
        };
        if !word_metadata.is_noun() || word_metadata.is_adjective() {
            return None;
        }

        ["es", "s"]
            .iter()
            .filter_map(|ending| word.strip_suffix(ending).map(|stem| (*ending, stem)))
            .filter(|(ending, stem)| Self::can_take_genitive_ending(ending, stem))
            .map(|(_, stem)| stem)
            .find(|stem| {
                let chars: Vec<char> = stem.chars().collect();
                self.dictionary
                    .get_word_metadata(&chars)
                    .is_some_and(|metadata| metadata.is_noun())
            })
            .map(str::to_string)
    }
}

fn with_case_of(template: &str, word: &str) -> String {
    if template.chars().next().is_some_and(char::is_uppercase) {
        let mut chars = word.chars();
        chars
            .next()
            .map(|first| first.to_uppercase().chain(chars).collect())
            .unwrap_or_default()
    } else {
        word.to_string()
    }
}

impl Linter for GermanGenitiveAfterNominativeArticle {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence.iter().collect();

            for window in tokens.windows(3) {
                let [article, space, noun] = window else {
                    continue;
                };

                if !matches!(article.kind, TokenKind::Word(_))
                    || !space.kind.is_whitespace()
                    || !matches!(noun.kind, TokenKind::Word(_))
                {
                    continue;
                }

                let article_text: String =
                    document.get_span_content(&article.span).iter().collect();
                let Some(genitive) = Self::genitive_article(&article_text) else {
                    continue;
                };

                // `Volks-wagen`: the word is the first half of a compound.
                let follows_hyphen = document
                    .get_full_content()
                    .get(noun.span.end)
                    .is_some_and(|c| *c == '-');
                if follows_hyphen {
                    continue;
                }

                let noun_text: String = document.get_span_content(&noun.span).iter().collect();
                let Some(stem) = self.nominative_stem(&noun_text) else {
                    continue;
                };

                let span = Span::new(article.span.start, noun.span.end);
                let genitive_article = with_case_of(&article_text, genitive);

                lints.push(Lint {
                    span,
                    lint_kind: LintKind::Grammar,
                    suggestions: vec![
                        Suggestion::ReplaceWith(format!("{article_text} {stem}").chars().collect()),
                        Suggestion::ReplaceWith(
                            format!("{genitive_article} {noun_text}").chars().collect(),
                        ),
                    ],
                    message: format!(
                        "Auf »{article_text}« folgt kein Genitiv. Entweder steht das Nomen im \
                         Nominativ (»{article_text} {stem}«) oder der Artikel im Genitiv \
                         (»{genitive_article} {noun_text}«)."
                    ),
                    priority: 31,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Findet ein Nomen mit Genitivendung hinter »das«, »ein« oder »kein«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanGenitiveAfterNominativeArticle;
    use crate::document::Document;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::{Lint, Linter, Suggestion};

    fn lint(text: &str) -> Vec<Lint> {
        let dictionary = combined_german_dictionary();
        let document = Document::new_markdown_default(text, &dictionary);
        GermanGenitiveAfterNominativeArticle::new().lint(&document)
    }

    fn fixes(text: &str) -> Vec<String> {
        lint(text)
            .iter()
            .flat_map(|lint| &lint.suggestions)
            .map(|suggestion| match suggestion {
                Suggestion::ReplaceWith(chars) => chars.iter().collect(),
                _ => String::new(),
            })
            .collect()
    }

    #[test]
    fn catches_genitive_after_das() {
        assert_eq!(
            fixes("Wenn das Programms nicht startet, prüfen Sie die Installation."),
            ["das Programm", "des Programms"]
        );
    }

    #[test]
    fn catches_the_es_ending() {
        assert_eq!(
            fixes("Ich sehe das Hauses nicht."),
            ["das Haus", "des Hauses"]
        );
    }

    #[test]
    fn catches_genitive_after_ein_and_kein() {
        assert_eq!(fixes("Ein Kindes schläft."), ["Ein Kind", "Eines Kindes"]);
        assert_eq!(
            fixes("Wir haben kein Programms."),
            ["kein Programm", "keines Programms"]
        );
    }

    #[test]
    fn keeps_the_case_of_the_article() {
        assert_eq!(
            fixes("Das Programms läuft."),
            ["Das Programm", "Des Programms"]
        );
    }

    #[test]
    fn accepts_the_nominative() {
        assert!(lint("Wenn das Programm nicht startet, prüfen Sie die Installation.").is_empty());
        assert!(lint("Das Haus ist groß und ein Kind spielt.").is_empty());
    }

    #[test]
    fn accepts_a_real_genitive() {
        assert!(lint("Der Start des Programms dauert lange.").is_empty());
        assert!(lint("Wegen eines Kindes kam er später.").is_empty());
    }

    #[test]
    fn leaves_nominalized_adjectives_alone() {
        assert!(lint("Das ist ein Gutes.").is_empty());
        assert!(lint("Wir wollen kein Neues.").is_empty());
    }

    #[test]
    fn leaves_words_whose_stem_is_too_short() {
        assert!(lint("Das Ass sticht.").is_empty());
        assert!(lint("Das Gas strömt aus.").is_empty());
    }

    #[test]
    fn leaves_the_first_half_of_a_compound_alone() {
        assert!(lint("Das Programms-Fenster öffnet sich.").is_empty());
    }

    #[test]
    fn leaves_nominatives_that_end_in_s_alone() {
        for text in [
            "Er ist ein Johannes.",
            "Das Regens der Anapher ist die Präposition.",
            "Das Andreas ist ein altenglisches Gedicht.",
            "Das Queens spielt heute.",
            "Das Genuss ist groß.",
        ] {
            assert!(lint(text).is_empty(), "{text}");
        }
    }

    #[test]
    fn leaves_feminine_nouns_alone() {
        assert!(lint("Das ist eine Frage.").is_empty());
    }

    #[test]
    fn leaves_lower_case_words_alone() {
        assert!(lint("Ich glaube, das kindes ist egal.").is_empty());
    }

    #[test]
    fn ignores_the_article_at_the_end() {
        assert!(lint("Ich nehme das").is_empty());
    }
}
