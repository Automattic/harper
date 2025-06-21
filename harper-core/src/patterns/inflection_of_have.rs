use super::SingleTokenPattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches inflections of the verb "have".
///
/// Currently only matches standard English inflections: "have", "has", "had", "having".
pub struct InflectionOfHave {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

// These are the standard inflections of the verb "have":
const FORMS: &[&str] = &[
    "have",   // infinitive, present, dictionary form, citation form, lemma
    "has",    // 3rd person singular present
    "had",    // past tense and past participle
    "having", // continuous, present participle, gerund, progressive
];

impl Default for InflectionOfHave {
    fn default() -> Self {
        Self::standard()
    }
}

impl InflectionOfHave {
    /// Creates a matcher for standard English inflections of "have".
    ///
    /// Matches: "have", "has", "had", "having"
    pub fn standard() -> Self {
        Self {
            inner: WordSet::new(FORMS),
        }
    }
}

impl SingleTokenPattern for InflectionOfHave {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        self.inner.matches_token(token, source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::patterns::{DocPattern, SpanVecExt};

    #[test]
    fn ensure_has_is_present() {
        let doc = Document::new_markdown_default_curated("He has a dog.");
        let matches = InflectionOfHave::standard().find_all_matches_in_doc(&doc);

        assert_eq!(matches.to_strings(&doc), vec!["has"]);
    }

    #[test]
    fn ensure_have_forms_are_present() {
        let doc = Document::new_markdown_default_curated(
            "I have, you have, he has, we had, they are having",
        );
        let matches = InflectionOfHave::standard().find_all_matches_in_doc(&doc);
        let matched = matches.to_strings(&doc);

        assert_eq!(matched, vec!["have", "have", "has", "had", "having"]);
    }
}
