//! The comma around a relative clause.

use crate::{
    Token, TokenKind, TokenStringExt,
    document::Document,
    language::german::grammar::determiners::determiner_readings,
    language::german::grammar::noun_gender::NounGender,
    language::morphology::{Gender, GenderSet},
    linting::{Lint, LintKind, Linter, Suggestion},
};

/// Subject pronouns that can open a relative clause right behind its pronoun:
/// *das Buch, das **ich** lese*.
const SUBJECT_PRONOUNS: &[&str] = &["ich", "du", "er", "sie", "es", "wir", "ihr", "man"];

/// Words that end the search for the clause's verb: a second clause starts.
const CLAUSE_BREAKERS: &[&str] = &[
    "und", "oder", "aber", "sondern", "denn", "doch", "sowie", "dass", "weil", "wenn", "ob", "als",
    "wie",
];

/// The longest relative clause, in words, this looks through for its verb.
const MAX_CLAUSE: usize = 10;

/// Requires the comma in front of a relative clause: *das Buch **das** ich
/// lese*, *der Mann **der** dort steht*.
///
/// German spells the relative pronoun like the article, and the comma is what
/// tells a reader which one it is — which is why a missing one is a real
/// error and also why finding it without a parser is dangerous. *Ich habe dem
/// Kind das Buch gegeben* has a noun, a `das` and no relative clause at all.
/// So the pronoun has to be followed by something an article never is:
///
/// * a **subject pronoun** — *das ich*, *die wir*, *den man*. An article is
///   followed by its noun or an adjective, never by *ich*;
/// * for *der* directly behind a masculine noun only, an **adverb or a verb** — *der
///   dort steht*, *der kommt*. *das* and *die* are left out here, because
///   both are also demonstratives that stand alone as objects: *Ich habe dem
///   Kind das gestern erklärt*.
///
/// A preposition may stand between the noun and the pronoun — *das Haus **in
/// dem** ich wohne* — and the comma then goes in front of the preposition.
///
/// Two more conditions hold the rest:
///
/// * the pronoun **agrees with the noun** in gender, read from the dictionary;
///   a noun with no recorded gender is skipped. *Er sagt der Mutter das er
///   kommt* has *das* behind a feminine noun, which is not a relative clause
///   (it is *dass*, see `DasDass.weir`);
/// * the clause **ends on a verb**, found within ten words: either the last
///   word before the punctuation, or a run of verbs whose last member is the
///   main clause's (*das ich lese **ist** spannend*). That second case also
///   gets the closing comma.
///
/// `das` behind a neuter noun stays ambiguous with `dass` — *Ich sage dem Kind
/// das er kommt* — and the message says so.
pub struct GermanRelativeClauseComma {
    nouns: NounGender,
}

impl Default for GermanRelativeClauseComma {
    fn default() -> Self {
        Self::new()
    }
}

impl GermanRelativeClauseComma {
    pub fn new() -> Self {
        Self {
            nouns: NounGender::new(),
        }
    }

    fn text(token: &Token, document: &Document) -> String {
        document.get_span_content(&token.span).iter().collect()
    }

    fn is_capitalized(token: &Token, document: &Document) -> bool {
        matches!(token.kind, TokenKind::Word(_))
            && document
                .get_span_content(&token.span)
                .first()
                .is_some_and(|c| c.is_uppercase())
    }

    /// The genders a relative pronoun can refer back to, and whether it can
    /// refer to a plural.
    fn antecedents(pronoun: &str) -> Option<(GenderSet, bool)> {
        match pronoun {
            "dessen" => Some((GenderSet::MASCULINE | GenderSet::NEUTER, false)),
            "deren" => Some((GenderSet::FEMININE, true)),
            "denen" => Some((GenderSet::empty(), true)),
            _ => {
                let readings = determiner_readings(pronoun)?;
                let mut genders = GenderSet::empty();
                let mut plural = false;
                for reading in readings {
                    match reading.gender {
                        Some(gender) => genders |= GenderSet::from(gender),
                        None => plural = true,
                    }
                }
                Some((genders, plural))
            }
        }
    }

    /// A lower-case word with a verb reading and no adjective, determiner or
    /// pronoun reading. A noun reading does not count against it: written
    /// lower case, `ist` and `lese` are not *das Ist* and *die Lese*.
    fn is_verb(token: &Token, document: &Document) -> bool {
        let TokenKind::Word(Some(metadata)) = &token.kind else {
            return false;
        };
        !Self::is_capitalized(token, document)
            && metadata.is_verb()
            && !metadata.is_adjective()
            && !metadata.is_determiner()
            && !metadata.is_pronoun()
    }

    fn is_adverb(token: &Token, document: &Document) -> bool {
        let TokenKind::Word(Some(metadata)) = &token.kind else {
            return false;
        };
        !Self::is_capitalized(token, document)
            && metadata.is_adverb()
            && !metadata.is_noun()
            && !metadata.is_adjective()
            && !metadata.is_determiner()
            && !metadata.is_verb()
    }

    /// Where the relative clause opened behind `pronoun_at` ends, and whether
    /// the main clause carries on behind it.
    ///
    /// `Some((last, true))` when a run of verbs is followed by more of the
    /// sentence: the run's last verb belongs to the main clause, so the
    /// relative clause ends one before it. `Some((last, false))` when the run
    /// is the end of the sentence or of the clause.
    fn clause_end(
        words: &[&Token],
        pronoun_at: usize,
        document: &Document,
    ) -> Option<(usize, bool)> {
        let mut at = pronoun_at + 1;
        while at < words.len() && at <= pronoun_at + MAX_CLAUSE {
            let token = words[at];
            if !matches!(token.kind, TokenKind::Word(_)) {
                return None;
            }
            let lower = Self::text(token, document).to_lowercase();
            if CLAUSE_BREAKERS.contains(&lower.as_str()) {
                return None;
            }
            if Self::is_verb(token, document) {
                let mut last = at;
                while words
                    .get(last + 1)
                    .is_some_and(|next| Self::is_verb(next, document))
                {
                    last += 1;
                }
                let sentence_goes_on = words
                    .get(last + 1)
                    .is_some_and(|next| matches!(next.kind, TokenKind::Word(_)));
                if !sentence_goes_on {
                    return Some((last, false));
                }
                // A single verb followed by more words leaves open where the
                // relative clause stops; two or more put the main verb last.
                return (last > at).then_some((last - 1, true));
            }
            at += 1;
        }
        None
    }
}

impl Linter for GermanRelativeClauseComma {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for sentence in document.iter_sentences() {
            let words: Vec<&Token> = sentence
                .iter()
                .filter(|token| !token.kind.is_whitespace())
                .collect();

            for (at, token) in words.iter().enumerate() {
                let pronoun = Self::text(token, document);
                if !matches!(token.kind, TokenKind::Word(_)) || pronoun != pronoun.to_lowercase() {
                    continue;
                }
                let Some((genders, plural)) = Self::antecedents(&pronoun) else {
                    continue;
                };
                // *das Haus **in dem** ich wohne*: German fronts the preposition
                // of a relativized phrase, and the comma goes in front of it.
                let behind_preposition = at
                    .checked_sub(1)
                    .is_some_and(|i| words[i].kind.is_preposition());
                let noun_at = if behind_preposition {
                    at.checked_sub(2)
                } else {
                    at.checked_sub(1)
                };
                let Some(noun_token) = noun_at.map(|i| words[i]) else {
                    continue;
                };
                if !Self::is_capitalized(noun_token, document) {
                    continue;
                }
                let content = document.get_full_content();
                if content.get(noun_token.span.end) == Some(&'-') {
                    continue;
                }

                let noun = Self::text(noun_token, document);
                let noun_genders = self.nouns.genders(&noun);
                if noun_genders.is_empty() {
                    continue;
                }
                let surely_singular = self
                    .nouns
                    .gender_of(&noun)
                    .is_some_and(|(_, surely_singular)| surely_singular);
                let agrees = genders.intersects(noun_genders) || (plural && !surely_singular);
                if !agrees {
                    continue;
                }

                let Some(next) = words.get(at + 1) else {
                    continue;
                };
                let next_text = Self::text(next, document);
                let opens_with_subject = SUBJECT_PRONOUNS.contains(&next_text.as_str());
                let masculine_subject = !behind_preposition
                    && pronoun == "der"
                    && noun_genders == GenderSet::from(Gender::Masculine)
                    && (Self::is_adverb(next, document) || Self::is_verb(next, document));
                if !(opens_with_subject || masculine_subject) {
                    continue;
                }

                let Some((last, main_clause_continues)) = Self::clause_end(&words, at, document)
                else {
                    continue;
                };

                let dass_hint = if pronoun == "das" {
                    " Ist kein Relativsatz gemeint, sondern ein Inhaltssatz, heißt es »dass«."
                } else {
                    ""
                };
                lints.push(Lint {
                    span: noun_token.span,
                    lint_kind: LintKind::Punctuation,
                    suggestions: vec![Suggestion::InsertAfter(vec![','])],
                    priority: 28,
                    message: format!(
                        "»{pronoun}« leitet hier einen Relativsatz ein. Davor steht ein Komma.{dass_hint}"
                    ),
                });
                if main_clause_continues {
                    lints.push(Lint {
                        span: words[last].span,
                        lint_kind: LintKind::Punctuation,
                        suggestions: vec![Suggestion::InsertAfter(vec![','])],
                        priority: 28,
                        message: "Der Relativsatz endet hier. Danach steht ein Komma.".to_string(),
                    });
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Setzt das Komma vor und nach einem Relativsatz (»das Buch, das ich lese, ist spannend«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanRelativeClauseComma;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;

    fn lint_count(text: &str) -> usize {
        let dict = combined_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        GermanRelativeClauseComma::new().lint(&document).len()
    }

    #[test]
    fn a_relative_clause_needs_its_commas() {
        assert_eq!(lint_count("Das Buch das ich lese ist spannend."), 2);
        assert_eq!(lint_count("Der Mann der dort steht ist mein Vater."), 2);
        assert_eq!(lint_count("Die Frau die ich kenne wohnt hier."), 2);
    }

    #[test]
    fn a_clause_at_the_end_needs_only_the_opening_comma() {
        assert_eq!(lint_count("Ich kenne den Mann der dort wohnt."), 1);
        assert_eq!(lint_count("Das ist das Buch das ich gelesen habe."), 1);
        assert_eq!(lint_count("Wo ist der Hund den wir gesehen haben?"), 1);
    }

    #[test]
    fn a_preposition_in_front_of_the_pronoun_takes_the_comma_with_it() {
        assert_eq!(lint_count("Das ist das Haus in dem ich wohne."), 1);
        assert_eq!(lint_count("Das ist das Haus, in dem ich wohne."), 0);
        assert_eq!(lint_count("Wir wohnen im Haus mit dem Garten."), 0);
    }

    #[test]
    fn commas_already_there_are_quiet() {
        assert_eq!(lint_count("Das Buch, das ich lese, ist spannend."), 0);
        assert_eq!(lint_count("Ich kenne den Mann, der dort wohnt."), 0);
    }

    #[test]
    fn an_article_behind_a_noun_is_not_a_relative_pronoun() {
        assert_eq!(lint_count("Ich habe dem Kind das Buch gegeben."), 0);
        assert_eq!(lint_count("Er zeigt der Frau die Stadt."), 0);
        assert_eq!(lint_count("Der Hund der Nachbarin bellt."), 0);
    }

    #[test]
    fn a_demonstrative_object_is_not_a_relative_clause() {
        assert_eq!(lint_count("Ich habe dem Kind das gestern erklärt."), 0);
        assert_eq!(lint_count("Wir haben den Kindern die schon gegeben."), 0);
    }

    #[test]
    fn a_pronoun_of_the_wrong_gender_is_not_a_relative_pronoun() {
        // *das* behind a feminine noun: this is *dass*, not a relative clause.
        assert_eq!(lint_count("Er sagt der Mutter das er kommt."), 0);
    }

    #[test]
    fn the_verb_may_stand_several_words_away() {
        assert_eq!(lint_count("Der Mann der gestern im Garten war."), 1);
        assert_eq!(lint_count("Am Tag der Arbeit ist frei."), 0);
    }
}
