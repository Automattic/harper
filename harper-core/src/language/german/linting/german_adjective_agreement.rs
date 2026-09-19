//! **Not registered in the lint group.** It emits nothing — not on "Der großes
//! Haus", not on "Die guten Mann", not on "Ein alte Frau". Its six tests passed
//! because every one of them asserted that *correct* text produces no lints, or
//! checked the description string; none asserted that a wrong phrase is caught,
//! so they would have passed against a linter that does nothing at all. Which
//! is what there is. The tests below now pin that instead.
//!
//! **The cause** is one line: `analyze_sentence` walks `DET ADJ NOUN` over a
//! token slice that still contains the whitespace, so `tokens[i - 1]` is always
//! a space and never a determiner. Nothing downstream of that ever runs.
//!
//! **What it would be worth.** LanguageTool reports 90 `DE_AGREEMENT` matches
//! on the 101-article prose corpus, of which somewhat over half are genuine —
//! "die hochdeutsche Dialekte", "bei den Weichtiere", "diese
//! Anwendungsprogrammen". That is the largest class of real German errors
//! Harper cannot see, well ahead of anything the registered rules miss:
//!
//! ```bash
//! docker run -d --name lt-de -p 8010:8010 -e Java_Xmx=8g erikvl87/languagetool
//! # then POST each corpus file to /v2/check with language=de-DE and count
//! # matches whose rule id is DE_AGREEMENT
//! ```
//!
//! **But do not simply delete the whitespace bug and register it.** The two
//! heuristics underneath are wrong in opposite directions: `der` + a noun in
//! `-ung` is the genitive/dative feminine, where the adjective *should* end in
//! `-en` ("wegen der schnellen Entwicklung"), and `die` + a noun ending in `n`
//! is far more often a correct singular ("die schnelle Bahn") than a plural.
//! Agreement needs case, and the dictionary carries gender and number but not
//! case, so this is a dictionary question before it is a linter question.
//!
//! German adjective agreement linter.
//!
//! This linter checks for proper adjective declension (adjective agreement) in German text.
//! In German, adjectives must agree with the nouns they modify in case, number, and gender.
//!
//! The linter detects patterns like:
//! - "die gute Mann" ❌ -> should be "der gute Mann" (gender mismatch)
//! - "der guten Tisch" ❌ -> should be "der gute Tisch" (case mismatch)
//! - "das gute Kinder" ❌ -> should be "die guten Kinder" (number mismatch)
//!
//! Implementation approach:
//! 1. Use Brill POS tagging to identify DET/ADP + ADJ + NOUN patterns
//! 2. Validate adjective endings match noun case, number, gender using dictionary metadata
//! 3. Start with basic patterns, expand to complex cases

use crate::{
    Token, TokenStringExt,
    document::Document,
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::Dictionary,
};
use harper_brill::UPOS;

/// A linter that checks for proper adjective agreement in German text.
/// In German, adjectives must agree with the nouns they modify in case, number, and gender.
pub struct GermanAdjectiveAgreement<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanAdjectiveAgreement<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Check if a token is a determiner, pronoun, or preposition that can introduce a noun phrase
    fn is_noun_phrase_introducer(&self, token: &Token, document: &Document) -> bool {
        let token_str: String = document.get_span_content(&token.span).iter().collect();

        // Check POS tag first
        if token.kind.is_upos(UPOS::DET)
            || token.kind.is_upos(UPOS::PRON)
            || token.kind.is_upos(UPOS::ADP)
        {
            return true;
        }

        // Check metadata for determiners/pronouns
        if let Some(Some(metadata)) = token.kind.as_word()
            && (metadata.is_determiner() || metadata.is_pronoun() || metadata.preposition)
        {
            return true;
        }

        // Common determiners and pronouns in German
        let introducers = [
            "der", "die", "das", "dem", "den", "des", // definite articles
            "ein", "eine", "einem", "einen", "eines", "einer", // indefinite articles
            "mein", "dein", "sein", "ihr", "unser", "euer", // possessive determiners
            "dieser", "diese", "dieses", "diesen", "diesem", "dieser", // demonstratives
            "jeder", "jede", "jedes", "jeden", "jedem", "jeder", // universal determiners
            "welcher", "welche", "welches", "welchen", "welchem",
            "welcher", // relative determiners
            "kein", "keine", "keinem", "keinen", "keines", "keiner", // negative determiners
            "mancher", "manche", "manches", "manchen", "manchem",
            "mancher", // some determiners
        ];

        introducers.contains(&token_str.as_str())
    }

    /// Check if a token is an adjective
    fn is_adjective(&self, token: &Token, document: &Document) -> bool {
        // Check POS tag first
        if token.kind.is_upos(UPOS::ADJ) {
            return true;
        }

        // Check metadata
        if let Some(Some(metadata)) = token.kind.as_word()
            && metadata.is_adjective()
        {
            return true;
        }

        // Also check dictionary metadata for the token
        let token_chars = document.get_span_content(&token.span);
        if let Some(metadata) = self.dictionary.get_word_metadata(token_chars)
            && metadata.is_adjective()
        {
            return true;
        }

        false
    }

    /// Check if a token is a noun
    fn is_noun(&self, token: &Token, document: &Document) -> bool {
        // Check POS tag first
        if token.kind.is_upos(UPOS::NOUN) || token.kind.is_upos(UPOS::PROPN) {
            return true;
        }

        // Check metadata
        if let Some(Some(metadata)) = token.kind.as_word()
            && metadata.is_noun()
        {
            return true;
        }

        // Also check dictionary metadata for the token
        let token_chars = document.get_span_content(&token.span);
        if let Some(metadata) = self.dictionary.get_word_metadata(token_chars)
            && metadata.is_noun()
        {
            return true;
        }

        false
    }

    /// Check if an adjective + noun pattern has proper agreement
    /// This is a simplified version that will be enhanced over time
    fn check_adjective_noun_agreement(
        &self,
        determiner_token: Option<&Token>,
        adjective_token: &Token,
        noun_token: &Token,
        document: &Document,
    ) -> Option<(String, String)> {
        // Get the actual text of the tokens
        let adj_text: String = document
            .get_span_content(&adjective_token.span)
            .iter()
            .collect();
        let noun_text: String = document.get_span_content(&noun_token.span).iter().collect();
        let det_text = determiner_token.map(|t| {
            document
                .get_span_content(&t.span)
                .iter()
                .collect::<String>()
        });

        // Skip if any token is not alphabetic
        if !adj_text.chars().all(|c| c.is_alphabetic())
            || !noun_text.chars().all(|c| c.is_alphabetic())
        {
            return None;
        }

        // Skip proper nouns (they don't take adjective agreement)
        if noun_token.kind.is_upos(UPOS::PROPN) {
            return None;
        }

        // For now, implement basic pattern detection
        // This is a placeholder - the actual implementation will be more sophisticated

        // Check for common errors:
        // 1. Feminine noun with masculine article and adjective
        // Example: "der gute Frau" -> should be "die gute Frau" or "der guten Frau"

        // 2. Plural noun with singular adjective
        // Example: "die gute Kinder" -> should be "die guten Kinder"

        // These are simplified checks that will be refined
        if let Some(det_text) = det_text {
            let det_lower = det_text.to_lowercase();
            let adj_lower = adj_text.to_lowercase();
            let noun_lower = noun_text.to_lowercase();

            // Check for gender mismatch: masculine article + feminine noun
            if (det_lower == "der" || det_lower == "ein") && noun_lower.ends_with("ung") {
                // Nouns ending in -ung are typically feminine
                // Check if adjective ends with -e (correct for nominative feminine)
                if !adj_lower.ends_with("e") {
                    return Some((adj_text, format!("{}e", adj_lower)));
                }
            }

            // Check for plural mismatch: singular article + plural noun
            if det_lower == "die" && noun_lower.ends_with("n") && !noun_lower.ends_with("en") {
                // This might be a plural noun (many German plurals end in -n or -en)
                // Check if adjective should be plural
                if !adj_lower.ends_with("en") {
                    return Some((adj_text, format!("{}en", adj_lower)));
                }
            }
        }

        None
    }

    /// Analyze a sentence for adjective agreement errors
    fn analyze_sentence(&self, sentence_tokens: &[Token], document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let tokens: Vec<&Token> = sentence_tokens.iter().collect();

        // Look for patterns: [DET/ADP/PRON]? + ADJ + NOUN
        for i in 1..tokens.len() {
            let current_token = tokens[i];

            if self.is_adjective(current_token, document) {
                // Look for preceding determiner/pronoun/preposition
                let prev_token = tokens[i.saturating_sub(1)];

                if self.is_noun_phrase_introducer(prev_token, document) || i == 0
                // adjective at start of sentence (uncommon but possible)
                {
                    // Look for following noun
                    if i + 1 < tokens.len() {
                        let next_token = tokens[i + 1];

                        if self.is_noun(next_token, document) {
                            // Found DET + ADJ + NOUN pattern
                            if let Some((current_adj, suggested_adj)) = self
                                .check_adjective_noun_agreement(
                                    Some(prev_token),
                                    current_token,
                                    next_token,
                                    document,
                                )
                            {
                                let current_span = current_token.span;
                                lints.push(Lint {
                                    span: current_span,
                                    lint_kind: LintKind::Grammar,
                                    suggestions: vec![Suggestion::ReplaceWith(suggested_adj.chars().collect())],
                                    priority: 25,
                                    message: format!(
                                        "Adjective '{}' may not agree with the noun in case, number, or gender. Did you mean '{}'?",
                                        current_adj, suggested_adj
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }

        lints
    }
}

impl<T: Dictionary> Linter for GermanAdjectiveAgreement<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence_tokens in paragraph.iter_sentences() {
                let sentence_lints = self.analyze_sentence(sentence_tokens, document);
                lints.extend(sentence_lints);
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for proper adjective agreement in German text"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;

    fn create_linter() -> GermanAdjectiveAgreement<impl Dictionary> {
        GermanAdjectiveAgreement::new(combined_german_dictionary())
    }

    fn create_document(text: &str) -> Document {
        Document::new(text, &PlainGerman, &combined_german_dictionary())
    }

    /// What the linter actually does, so that the file cannot look tested
    /// again.
    ///
    /// Every sentence here is a real agreement error lifted from the prose
    /// corpus, and the linter finds none of them. When someone makes it work,
    /// this test will fail, and the sentences are then the start of its real
    /// test suite.
    #[test]
    fn finds_none_of_the_errors_it_is_named_for() {
        let mut linter = create_linter();

        for text in [
            "Die hochdeutsche Dialekte wurden von der Lautverschiebung erfasst.",
            "Bei den Weichtiere haben Schnecken und Muscheln ein Herz.",
            "Salzborn setzt den Beginn nach den Terroranschläge an.",
            "Das Betriebssystem stellt diese Anwendungsprogrammen zur Verfügung.",
            "Auslöser ist die Kritik an der moderne Architektur.",
            "Das deckt sich mit dem personalem Erzähler.",
            "Der großes Haus steht dort.",
            "Ein alte Frau kam herein.",
        ] {
            let document = create_document(text);
            assert!(
                linter.lint(&document).is_empty(),
                "this linter is not wired up and has never caught anything; \
                 if it now catches {text:?}, delete this test and write real ones"
            );
        }
    }

    /// And why: the pattern walks `DET ADJ NOUN` over a token slice that still
    /// contains the spaces, so the token before an adjective is always
    /// whitespace and never a determiner.
    #[test]
    fn the_pattern_never_sees_a_determiner() {
        let linter = create_linter();
        let document = create_document("Der gute Mann ist hier.");

        let tokens: Vec<_> = document
            .iter_paragraphs()
            .flat_map(|p| p.iter_sentences())
            .flat_map(|s| s.tokens())
            .collect();

        let adjective = tokens
            .iter()
            .position(|token| linter.is_adjective(token, &document))
            .expect("'gute' is an adjective");

        assert!(
            !linter.is_noun_phrase_introducer(tokens[adjective - 1], &document),
            "the token before the adjective is the space, not 'Der'"
        );
    }

    #[test]
    fn test_empty_text() {
        let mut linter = create_linter();
        let text = "";
        let document = create_document(text);
        let lints = linter.lint(&document);

        assert_eq!(lints.len(), 0, "Empty text should produce no lints");
    }
}
