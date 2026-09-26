//! Word pairs the reform of 1996 allows to be written as one word.

use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Span, Token, TokenKind, TokenStringExt, document::Document};

/// What may follow a fused preposition: a genitive article, a possessive, or
/// `von`. This is the whole guard, and it is not optional — the separated
/// spelling is the *literal* one and stays correct. A ship runs *auf Grund*,
/// and question three is *in Frage 3*.
const GENITIVE: &[&str] = &[
    "des", "der", "dieses", "dieser", "jenes", "jener", "eines", "einer", "seines", "seiner",
    "ihres", "ihrer", "unseres", "unserer", "meines", "meiner", "deines", "deiner", "jedes",
    "jeder", "solcher", "beider", "aller", "von", "vom",
];

/// The verbs *in Frage* takes when it is the idiom rather than a numbered
/// question.
const FRAGE_VERBS: &[&str] = &[
    "kommen",
    "kommt",
    "kam",
    "kamen",
    "käme",
    "gekommen",
    "stellen",
    "stellt",
    "stellte",
    "stellten",
    "gestellt",
    "stehen",
    "steht",
    "stand",
    "standen",
    "gestanden",
    "ziehen",
    "zieht",
    "zog",
    "gezogen",
];

/// The verbs *zu Grunde* takes.
const GRUNDE_VERBS: &[&str] = &[
    "gehen",
    "geht",
    "ging",
    "gegangen",
    "liegen",
    "liegt",
    "lag",
    "gelegen",
    "legen",
    "legt",
    "legte",
    "gelegt",
    "richten",
    "richtet",
    "richtete",
    "gerichtet",
];

/// Which words may follow the pair for it to be the fused reading.
enum Follows {
    Genitive,
    OneOf(&'static [&'static str]),
}

/// A separated spelling, the word it fuses into, and what has to follow.
struct Fusion {
    first: &'static str,
    second: &'static str,
    fused: &'static str,
    follows: Follows,
}

const FUSIONS: &[Fusion] = &[
    Fusion {
        first: "in",
        second: "frage",
        fused: "infrage",
        follows: Follows::OneOf(FRAGE_VERBS),
    },
    Fusion {
        first: "zu",
        second: "grunde",
        fused: "zugrunde",
        follows: Follows::OneOf(GRUNDE_VERBS),
    },
    Fusion {
        first: "mit",
        second: "hilfe",
        fused: "mithilfe",
        follows: Follows::Genitive,
    },
    Fusion {
        first: "auf",
        second: "grund",
        fused: "aufgrund",
        follows: Follows::Genitive,
    },
    Fusion {
        first: "an",
        second: "stelle",
        fused: "anstelle",
        follows: Follows::Genitive,
    },
    Fusion {
        first: "an",
        second: "hand",
        fused: "anhand",
        follows: Follows::Genitive,
    },
    Fusion {
        first: "zu",
        second: "gunsten",
        fused: "zugunsten",
        follows: Follows::Genitive,
    },
    Fusion {
        first: "zu",
        second: "lasten",
        fused: "zulasten",
        follows: Follows::Genitive,
    },
];

/// Recommends the fused spelling of *in Frage*, *mit Hilfe*, *auf Grund* and
/// the rest.
///
/// Both spellings have been correct since 1996 and Duden recommends the fused
/// one, so this is a style rule rather than a grammar rule. It is also the
/// class LanguageTool reports most often on German prose that Harper had
/// nothing for.
///
/// The guard is what makes it usable. Each of these pairs has a literal reading
/// in which the separated spelling is the only correct one — *das Schiff lief
/// auf Grund*, *die Antwort steht in Frage 3*, *er legte die Hand auf den
/// Grund* — and what tells the two apart is the word behind them: the genitive
/// or `von` that a preposition governs, or the fixed verb the idiom takes.
#[derive(Default)]
pub struct GermanRecommendedFusion;

impl GermanRecommendedFusion {
    fn lowercase_of(token: &Token, document: &Document) -> String {
        document
            .get_span_content(&token.span)
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect()
    }

    /// The fused form, capitalized the way the first word was.
    fn suggestion(fused: &str, first_was_capitalized: bool) -> Vec<char> {
        let mut chars: Vec<char> = fused.chars().collect();
        if first_was_capitalized && let Some(first) = chars.first_mut() {
            *first = first.to_uppercase().next().unwrap_or(*first);
        }
        chars
    }
}

impl Linter for GermanRecommendedFusion {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let tokens: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();

            for index in 0..tokens.len().saturating_sub(2) {
                let words: Vec<Option<String>> = (0..3)
                    .map(|offset| {
                        let token = tokens[index + offset];
                        matches!(token.kind, TokenKind::Word(_))
                            .then(|| Self::lowercase_of(token, document))
                    })
                    .collect();
                let (Some(first), Some(second), Some(third)) = (&words[0], &words[1], &words[2])
                else {
                    continue;
                };

                let Some(fusion) = FUSIONS
                    .iter()
                    .find(|fusion| fusion.first == first && fusion.second == second)
                else {
                    continue;
                };

                let licensed = match fusion.follows {
                    Follows::Genitive => GENITIVE.contains(&third.as_str()),
                    Follows::OneOf(verbs) => verbs.contains(&third.as_str()),
                };
                if !licensed {
                    continue;
                }

                // The second word is a noun here and is capitalized; in the
                // fused spelling it is not, so a lower-case second word means
                // the writer already wrote something else.
                let second_token = tokens[index + 1];
                if !document
                    .get_span_content(&second_token.span)
                    .first()
                    .is_some_and(|c| c.is_uppercase())
                {
                    continue;
                }

                let first_token = tokens[index];
                let capitalized = document
                    .get_span_content(&first_token.span)
                    .first()
                    .is_some_and(|c| c.is_uppercase());

                lints.push(Lint {
                    span: Span::new(first_token.span.start, second_token.span.end),
                    lint_kind: LintKind::Style,
                    suggestions: vec![Suggestion::ReplaceWith(Self::suggestion(
                        fusion.fused,
                        capitalized,
                    ))],
                    priority: 40,
                    message: format!(
                        "Duden empfiehlt »{}«. Beide Schreibweisen sind zulässig.",
                        Self::suggestion(fusion.fused, capitalized)
                            .iter()
                            .collect::<String>()
                    ),
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Empfiehlt die zusammengeschriebene Form von Fügungen wie »in Frage« oder »auf Grund«."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanRecommendedFusion;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn suggestion(text: &str) -> Option<String> {
        let document = Document::new(text, &PlainGerman, &combined_german_dictionary());
        let lints = GermanRecommendedFusion.lint(&document);
        lints.first().map(|lint| {
            lint.message
                .split('»')
                .nth(1)
                .unwrap_or_default()
                .split('«')
                .next()
                .unwrap_or_default()
                .to_string()
        })
    }

    #[test]
    fn a_governed_genitive_licenses_the_fusion() {
        for (text, fused) in [
            ("Sie löste es mit Hilfe der Kollegen.", "mithilfe"),
            ("Er kam auf Grund der Umstände zu spät.", "aufgrund"),
            ("Er entschied an Hand der Unterlagen.", "anhand"),
            ("Das wirkt zu Gunsten der Firma.", "zugunsten"),
            ("Es geht zu Lasten des Betroffenen.", "zulasten"),
            ("An Stelle des Vertrags galt eine Absprache.", "Anstelle"),
        ] {
            assert_eq!(suggestion(text).as_deref(), Some(fused), "on {text:?}");
        }
    }

    #[test]
    fn von_licenses_it_as_well() {
        assert_eq!(
            suggestion("Mit Hilfe von Werkzeugen ging es besser.").as_deref(),
            Some("Mithilfe")
        );
        assert_eq!(
            suggestion("Auf Grund von Regen fiel es aus.").as_deref(),
            Some("Aufgrund")
        );
    }

    #[test]
    fn the_idioms_take_a_verb_instead() {
        for (text, fused) in [
            ("Das kann in Frage kommen.", "infrage"),
            ("Er hat alles in Frage gestellt.", "infrage"),
            ("Die Annahme wird zu Grunde gelegt.", "zugrunde"),
            ("Der Plan wird zu Grunde gehen.", "zugrunde"),
        ] {
            assert_eq!(suggestion(text).as_deref(), Some(fused), "on {text:?}");
        }
    }

    /// The literal reading, where the separated spelling is the only correct
    /// one. This is what the guard is for.
    #[test]
    fn the_literal_reading_is_left_alone() {
        for text in [
            "Das Schiff lief auf Grund und sank.",
            "Die Antwort steht in Frage 3 des Fragebogens.",
            "Er legte die Hand auf den Grund des Beckens.",
            "An dieser Stelle endet der Weg.",
            "Sie hielt seine Hand.",
            "Zu Lasten und Nutzen gibt es Studien.",
        ] {
            assert_eq!(suggestion(text), None, "should stay quiet on {text:?}");
        }
    }

    /// Already fused, nothing to say.
    #[test]
    fn the_fused_spelling_is_accepted() {
        for text in [
            "Das kann infrage kommen.",
            "Sie löste es mithilfe der Kollegen.",
            "Aufgrund des Wetters fiel es aus.",
            "Es geht zulasten der Umwelt.",
        ] {
            assert_eq!(suggestion(text), None, "should stay quiet on {text:?}");
        }
    }

    /// The capital follows the first word, not the noun.
    #[test]
    fn the_suggestion_keeps_the_sentence_capital() {
        assert_eq!(
            suggestion("Auf Grund des Wetters fiel es aus.").as_deref(),
            Some("Aufgrund")
        );
        assert_eq!(
            suggestion("Es fiel auf Grund des Wetters aus.").as_deref(),
            Some("aufgrund")
        );
    }

    /// A lower-case second word is not the separated spelling of a noun; the
    /// writer wrote something else.
    #[test]
    fn a_lowercase_second_word_is_not_this() {
        assert_eq!(suggestion("Er kam auf grund der Sache."), None);
    }
}
