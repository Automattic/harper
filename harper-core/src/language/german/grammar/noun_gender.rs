//! What the dictionary says about a noun's gender, and how far to trust it.
//!
//! Two linters ask the same question of the same head noun. `GermanDeterminerGender`
//! wants the one gender that *die Hund* contradicts, and the subject-verb
//! rule wants to know whether *die Kinder* can be a feminine singular at all.
//! Both read the base dictionary, never the compound-aware one, for the reason
//! the preposition rule gives: a decomposition invents readings.

use std::sync::Arc;

use crate::{
    Token, TokenKind,
    document::Document,
    language::german::grammar::noun_phrase::Phrase,
    language::german::spell::curated_german_dictionary,
    language::morphology::{Gender, GenderSet, MorphologyExt, NumberSet},
    spell::{Dictionary, FstDictionary},
};

/// Endings of a noun whose plural may be spelled like its singular (*der
/// Lehrer*, *die Lehrer*; *das Mädchen*, *die Mädchen*; *der Kuchen*). The
/// dictionary records some of these as singular only, so the spelling decides
/// unless the plural is known to carry an umlaut.
const SAME_AS_PLURAL: &[&str] = &["en", "er", "el"];

pub struct NounGender {
    dictionary: Arc<FstDictionary>,
}

impl Default for NounGender {
    fn default() -> Self {
        Self::new()
    }
}

impl NounGender {
    pub fn new() -> Self {
        Self {
            dictionary: curated_german_dictionary(),
        }
    }

    pub fn dictionary(&self) -> &FstDictionary {
        &self.dictionary
    }

    /// Every gender the dictionary records for `head` as a noun. Empty for a
    /// word that is not a noun or carries no gender.
    pub fn genders(&self, head: &str) -> GenderSet {
        let chars: Vec<char> = head.chars().collect();
        self.dictionary
            .get_word_metadata(&chars)
            .filter(|metadata| metadata.is_noun())
            .map(|metadata| metadata.noun_agreement().gender)
            .unwrap_or(GenderSet::empty())
    }

    /// Is `head` recorded as a plural and nothing else, with no gender — a
    /// plurale tantum like *Eltern*, *Leute*, *Ferien*?
    pub fn is_plural_only(&self, head: &str) -> bool {
        let chars: Vec<char> = head.chars().collect();
        self.dictionary
            .get_word_metadata(&chars)
            .filter(|metadata| metadata.is_noun())
            .is_some_and(|metadata| {
                let agreement = metadata.noun_agreement();
                agreement.number == NumberSet::PLURAL && agreement.gender.is_empty()
            })
    }

    /// The one gender the dictionary records for `head`, and whether the noun
    /// is surely a singular.
    ///
    /// "Surely" means a base entry marked singular whose spelling does not
    /// allow it to be a plural as well: *Hund* and *Haus* are, *Lehrer* and
    /// *Mädchen* are not, and *Schule* is not either, since the entry has both
    /// numbers. A noun in `-en`, `-er` or `-el` is surely singular only when
    /// its plural is known to be spelled with an umlaut — see
    /// [`Self::has_umlaut_plural`].
    pub fn gender_of(&self, head: &str) -> Option<(Gender, bool)> {
        let chars: Vec<char> = head.chars().collect();
        let metadata = self
            .dictionary
            .get_word_metadata(&chars)
            .filter(|metadata| metadata.is_noun())?;
        let agreement = metadata.noun_agreement();

        // `unique` is `None` for an empty set and for one with several genders.
        let gender = agreement.gender.unique()?;
        let lower = head.to_lowercase();
        // An acronym inflects for nothing: *die AGB*.
        let acronym = head.chars().all(|c| !c.is_lowercase());
        let spelled_like_a_plural = SAME_AS_PLURAL.iter().any(|ending| lower.ends_with(ending));
        let surely_singular = agreement.number == NumberSet::SINGULAR
            && !acronym
            && (!spelled_like_a_plural || self.has_umlaut_plural(head, gender));

        Some((gender, surely_singular))
    }

    /// Is the plural of `head` the umlauted form, so that `head` itself cannot
    /// be one? *Garten*/*Gärten*, *Vogel*/*Vögel*, *Vater*/*Väter*.
    ///
    /// The umlauted spelling has to be a noun the dictionary knows, and must
    /// not be recorded with a gender of its own that differs: *Küchen* is the
    /// plural of *Küche*, not of *Kuchen*, whose plural is *Kuchen*. The
    /// singular entry carrying the number is the other half of the evidence,
    /// and it is curated in `add_german_common_noun_genders.py`.
    fn has_umlaut_plural(&self, head: &str, gender: Gender) -> bool {
        let Some(umlauted) = umlaut_last_vowel(head) else {
            return false;
        };
        let chars: Vec<char> = umlauted.chars().collect();
        self.dictionary
            .get_word_metadata(&chars)
            .filter(|metadata| metadata.is_noun())
            .is_some_and(|metadata| {
                let recorded = metadata.noun_agreement().gender;
                recorded.is_empty() || recorded == GenderSet::from(gender)
            })
    }
}

/// The word with the last umlautable vowel of its stem umlauted: *Garten* →
/// *Gärten*, *Vogel* → *Vögel*, *Bauer* → *Bäuer*. `None` when there is none.
fn umlaut_last_vowel(word: &str) -> Option<String> {
    let mut chars: Vec<char> = word.chars().collect();
    // The vowel to umlaut sits in front of the ending, not in it.
    let stem_end = chars.len().saturating_sub(2);
    let at = chars[..stem_end]
        .iter()
        .rposition(|c| matches!(c, 'a' | 'o' | 'u' | 'A' | 'O' | 'U'))?;
    // In *au* the umlaut lands on the *a*: *Haus* → *Häuser*.
    let at = if at > 0 && matches!(chars[at], 'u') && matches!(chars[at - 1], 'a' | 'A') {
        at - 1
    } else {
        at
    };
    chars[at] = match chars[at] {
        'a' => 'ä',
        'o' => 'ö',
        'u' => 'ü',
        'A' => 'Ä',
        'O' => 'Ö',
        'U' => 'Ü',
        other => other,
    };
    Some(chars.into_iter().collect())
}

/// The noun that ends `phrase`, when the chunker's choice can be trusted.
///
/// The preposition rule asks for a function word or punctuation behind the
/// head, which is the right bar for a case error and the wrong one here: *die
/// Hund gesehen* ends on a participle and is exactly what the gender rule is
/// for. What has to be ruled out instead is a head that is only the first
/// capital of a longer run — *den Berliner Philharmonikern*, *die Deutsche
/// Bahn* — so a capitalized word behind it rejects the phrase, as does a
/// hyphen, which makes the head half of a compound.
pub fn trusted_head(document: &Document, words: &[&Token], phrase: &Phrase) -> Option<String> {
    let token = words[phrase.head];
    if !matches!(token.kind, TokenKind::Word(_)) {
        return None;
    }

    let content = document.get_full_content();
    let touches_hyphen = content.get(token.span.end) == Some(&'-')
        || (token.span.start > 0 && content.get(token.span.start - 1) == Some(&'-'));
    if touches_hyphen {
        return None;
    }

    let capitalized = |token: &Token| {
        matches!(token.kind, TokenKind::Word(_))
            && document
                .get_span_content(&token.span)
                .first()
                .is_some_and(|c| c.is_uppercase())
    };
    if words
        .get(phrase.head + 1)
        .is_some_and(|next| capitalized(next))
    {
        return None;
    }

    let head: String = document.get_span_content(&token.span).iter().collect();
    head.chars()
        .next()
        .is_some_and(char::is_uppercase)
        .then_some(head)
}

#[cfg(test)]
mod tests {
    use super::{NounGender, umlaut_last_vowel};
    use crate::language::morphology::Gender;

    #[test]
    fn the_last_stem_vowel_is_umlauted() {
        assert_eq!(umlaut_last_vowel("Garten").as_deref(), Some("Gärten"));
        assert_eq!(umlaut_last_vowel("Vogel").as_deref(), Some("Vögel"));
        assert_eq!(umlaut_last_vowel("Bauer").as_deref(), Some("Bäuer"));
        assert_eq!(umlaut_last_vowel("Lehrer"), None);
    }

    #[test]
    fn an_umlaut_plural_makes_the_singular_certain() {
        let nouns = NounGender::new();
        assert_eq!(nouns.gender_of("Garten"), Some((Gender::Masculine, true)));
        assert_eq!(nouns.gender_of("Vogel"), Some((Gender::Masculine, true)));
    }

    #[test]
    fn a_plural_spelled_like_the_singular_stays_open() {
        let nouns = NounGender::new();
        // *die Kuchen* is the plural; *Küchen* belongs to *Küche*.
        assert_eq!(nouns.gender_of("Kuchen"), Some((Gender::Masculine, false)));
        assert_eq!(nouns.gender_of("Lehrer"), Some((Gender::Masculine, false)));
    }
}
