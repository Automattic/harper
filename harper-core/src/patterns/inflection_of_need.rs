use super::SingleTokenPattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches inflections of the verb "need" with configurable strictness.
///
/// By default, only matches standard English inflections. Use `with_common_errors()` to include
/// frequently encountered incorrect forms like "need's".
pub struct InflectionOfNeed {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

// These are the standard inflections of the verb "need":
const GOOD_FORMS: &[&str] = &[
    "need",    // infinitive, present, dictionary form, citation form, lemma
    "needs",   // 3rd person singular present
    "needed",  // past tense and past participle
    "needing", // continuous, present participle, gerund, progressive
];

// These are common but incorrect forms that might be encountered in user input.
const BAD_FORMS: &[&str] = &["need's"];

impl Default for InflectionOfNeed {
    fn default() -> Self {
        Self::standard()
    }
}

impl InflectionOfNeed {
    /// Creates a matcher for standard English inflections of "need".
    ///
    /// Matches only the correct forms: "need", "needs", "needed", "needing".
    pub fn standard() -> Self {
        Self {
            inner: WordSet::new(GOOD_FORMS),
        }
    }

    /// Creates a matcher that includes common misspellings and errors.
    ///
    /// In addition to standard forms, matches common errors like "need's" which
    /// are frequently seen in user-generated content.
    pub fn with_common_errors() -> Self {
        Self {
            inner: WordSet::new(&[GOOD_FORMS, BAD_FORMS].concat()),
        }
    }
}

impl SingleTokenPattern for InflectionOfNeed {
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
    fn ensure_needs_is_present() {
        let doc = Document::new_markdown_default_curated("He needs his coffee.");
        let matches = InflectionOfNeed::standard().find_all_matches_in_doc(&doc);

        assert_eq!(matches.to_strings(&doc), vec!["needs"]);
    }

    #[test]
    fn ensure_needs_apostrophe_is_absent() {
        let doc = Document::new_markdown_default_curated("He need's his coffee.");
        let matches = InflectionOfNeed::standard().find_all_matches_in_doc(&doc);

        assert_eq!((&matches[..]).to_strings(&doc), vec![] as Vec<String>);
    }

    #[test]
    fn ensure_needs_apostrophe_is_present() {
        let doc = Document::new_markdown_default_curated("He need's his coffee.");
        let matches = InflectionOfNeed::with_common_errors().find_all_matches_in_doc(&doc);

        assert_eq!(matches.to_strings(&doc), vec!["need's"]);
    }
}
