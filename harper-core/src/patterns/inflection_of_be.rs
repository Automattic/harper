use super::SingleTokenPattern;
use crate::Token;
use crate::patterns::WordSet;

/// Matches inflections of the verb "be".
///
/// Matches standard English inflections: "be", "am", "is", "are", "was", "were", "been", "being".
pub struct InflectionOfBe {
    /// If using a `WordSet` proves expensive, we'll switch to something else.
    inner: WordSet,
}

// These are the standard inflections of the verb "be":
const FORMS: &[&str] = &[
    "be",    // infinitive, dictionary form, citation form, lemma
    "am",    // 1st person singular present
    "is",    // 3rd person singular present
    "are",   // 2nd person singular and all plural present
    "was",   // 1st and 3rd person singular past
    "were",  // 2nd person singular and all plural past
    "been",  // past participle
    "being", // present participle, gerund, progressive
];

impl Default for InflectionOfBe {
    fn default() -> Self {
        Self::standard()
    }
}

impl InflectionOfBe {
    /// Creates a matcher for standard English inflections of "be".
    ///
    /// Matches: "be", "am", "is", "are", "was", "were", "been", "being"
    pub fn standard() -> Self {
        Self {
            inner: WordSet::new(FORMS),
        }
    }
}

impl SingleTokenPattern for InflectionOfBe {
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
    fn ensure_common_forms_are_present() {
        let doc =
            Document::new_markdown_default_curated("I am, you are, he is, we were, they have been");
        let matches = InflectionOfBe::standard().find_all_matches_in_doc(&doc);
        let mut matched = matches.to_strings(&doc);
        matched.sort();

        assert_eq!(matched, vec!["am", "are", "been", "is", "were"]);
    }

    #[test]
    fn ensure_other_forms_are_present() {
        let doc = Document::new_markdown_default_curated("be, being, was");
        let matches = InflectionOfBe::standard().find_all_matches_in_doc(&doc);
        let mut matched = matches.to_strings(&doc);
        matched.sort();

        assert_eq!(matched, vec!["be", "being", "was"]);
    }
}
