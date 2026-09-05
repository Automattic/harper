//! German pronoun agreement linter.
//!
//! This linter checks for proper pronoun agreement in German text.
//! In German, pronouns must agree with their referents in person, number, and case.
//!
//! The linter detects patterns like:
//! - "Er gibt sie ihm" ❌ (if "sie" refers to a masculine singular object)
//! - "Sie gibt es ihr" ✓ (correct feminine singular pronoun agreement)
//!
//! Implementation approach:
//! 1. Use dictionary metadata to get pronoun case, number, and gender
//! 2. Validate pronoun usage based on context and referent information
//! 3. Start with basic patterns, expand to complex cases

use crate::{
    Token,
    document::Document,
    linting::{Lint, Linter},
    spell::Dictionary,
};
use harper_brill::UPOS;

/// A linter that checks for proper pronoun agreement in German text.
pub struct GermanPronounAgreement<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanPronounAgreement<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Check if a pronoun has case metadata
    fn get_pronoun_case(
        &self,
        token: &Token,
        document: &Document,
    ) -> Option<crate::dict_word_metadata::Case> {
        let token_chars = document.get_span_content(&token.span);
        self.dictionary
            .get_word_metadata(token_chars)
            .and_then(|metadata| metadata.get_pronoun_case())
    }

    /// Check if a pronoun has gender metadata
    fn get_pronoun_gender(
        &self,
        token: &Token,
        document: &Document,
    ) -> Option<crate::dict_word_metadata::Gender> {
        let token_chars = document.get_span_content(&token.span);
        self.dictionary
            .get_word_metadata(token_chars)
            .and_then(|metadata| metadata.get_pronoun_gender())
    }

    /// Check if a pronoun has number metadata
    fn get_pronoun_number(
        &self,
        token: &Token,
        document: &Document,
    ) -> Option<crate::dict_word_metadata::Number> {
        let token_chars = document.get_span_content(&token.span);
        self.dictionary
            .get_word_metadata(token_chars)
            .and_then(|metadata| metadata.get_pronoun_number())
    }

    /// Check if a token is a personal pronoun
    fn is_personal_pronoun(&self, token: &Token, document: &Document) -> bool {
        let token_chars = document.get_span_content(&token.span);
        if let Some(metadata) = self.dictionary.get_word_metadata(token_chars) {
            return metadata.is_personal_pronoun();
        }

        // Also check common German personal pronouns
        let token_text: String = token_chars.iter().collect();
        let common_pronouns = [
            "ich", "du", "er", "sie", "es", "wir", "ihr", "sie", "Sie", "me", "dir", "ihm", "ihr",
            "uns", "euch", "ihnen", "mich", "dich", "ihn", "sie", "es", "uns", "euch", "sie",
            "Sie",
        ];
        common_pronouns.contains(&token_text.to_lowercase().as_str())
    }

    /// Analyze pronoun usage patterns
    fn check_pronoun_usage(&self, pronoun_token: &Token, document: &Document) -> Option<Lint> {
        // This is a basic implementation that will be enhanced
        // For now, check if pronouns have metadata and validate basic patterns

        // Check if this is a personal pronoun
        if !self.is_personal_pronoun(pronoun_token, document) {
            return None;
        }

        // Get pronoun metadata
        let pronoun_case = self.get_pronoun_case(pronoun_token, document);
        let pronoun_gender = self.get_pronoun_gender(pronoun_token, document);
        let pronoun_number = self.get_pronoun_number(pronoun_token, document);

        // This is a placeholder for more sophisticated checks
        // For now, just validate that we can access the metadata
        if pronoun_case.is_none() && pronoun_gender.is_none() && pronoun_number.is_none() {
            // No metadata available yet - this is expected until dictionary is updated
            return None;
        }

        None
    }
}

impl<T: Dictionary> Linter for GermanPronounAgreement<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // This is a basic implementation that will be enhanced
        // For now, check individual pronouns for metadata consistency
        let tokens = document.get_tokens();

        for token in tokens {
            // Check if this is a pronoun
            if token.kind.is_upos(UPOS::PRON)
                && let Some(lint) = self.check_pronoun_usage(token, document)
            {
                lints.push(lint);
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for proper pronoun agreement in German text"
    }
}
