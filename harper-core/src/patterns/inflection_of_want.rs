use super::SingleTokenPattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches inflections of the verb "want" with configurable strictness.
///
/// By default, only matches standard English inflections. Use `with_common_errors()` to include
/// frequently encountered incorrect forms like "want's".
pub struct InflectionOfWant {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

// These are the standard inflections of the verb "want":
const GOOD_FORMS: &[&str] = &[
    "want",    // infinitive, present, dictionary form, citation form, lemma
    "wants",   // 3rd person singular present
    "wanted",  // past tense and past participle
    "wanting", // continuous, present participle, gerund, progressive
];

// These are common but incorrect forms that might be encountered in user input.
const BAD_FORMS: &[&str] = &["want's"];

impl Default for InflectionOfWant {
    fn default() -> Self {
        Self::standard()
    }
}

impl InflectionOfWant {
    /// Creates a matcher for standard English inflections of "want".
    ///
    /// Matches only the correct forms: "want", "wants", "wanted", "wanting".
    pub fn standard() -> Self {
        Self {
            inner: WordSet::new(GOOD_FORMS),
        }
    }

    /// Creates a matcher that includes common misspellings and errors.
    ///
    /// In addition to standard forms, matches common errors like "want's" which
    /// are frequently seen in user-generated content.
    pub fn with_common_errors() -> Self {
        Self {
            inner: WordSet::new(&[GOOD_FORMS, BAD_FORMS].concat()),
        }
    }
}

impl SingleTokenPattern for InflectionOfWant {
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
    fn ensure_wants_is_present() {
        let doc = Document::new_markdown_default_curated("He wants his mommy.");
        let matches = InflectionOfWant::standard().find_all_matches_in_doc(&doc);

        assert_eq!(matches.to_strings(&doc), vec!["wants"]);
    }

    #[test]
    fn ensure_wants_apostrophe_is_absent() {
        let doc = Document::new_markdown_default_curated("He want's his mommy.");
        let matches = InflectionOfWant::standard().find_all_matches_in_doc(&doc);

        assert_eq!((&matches[..]).to_strings(&doc), vec![] as Vec<String>);
    }

    #[test]
    fn ensure_wants_apostrophe_is_present() {
        let doc = Document::new_markdown_default_curated("He want's his mommy.");
        let matches = InflectionOfWant::with_common_errors().find_all_matches_in_doc(&doc);

        assert_eq!(matches.to_strings(&doc), vec!["want's"]);
    }
}
