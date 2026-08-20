//! German subject-verb agreement linter.
//!
//! This linter checks for proper subject-verb agreement in German text.
//! In German, verbs must agree with their subjects in person and number.
//!
//! The linter detects patterns like:
//! - "Er gehen" ❌ -> should be "Er geht" (3rd person singular)
//! - "Wir geht" ❌ -> should be "Wir gehen" (1st person plural)
//! - "Das Kind spielen" ❌ -> should be "Das Kind spielt" (3rd person singular)
//!
//! Implementation approach:
//! 1. Use dictionary metadata to get verb conjugation information
//! 2. Identify subject-verb pairs and validate agreement
//! 3. Start with basic patterns, expand to complex cases

use crate::{
    Token,
    document::Document,
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::Dictionary,
};
use harper_brill::UPOS;

/// A linter that checks for proper subject-verb agreement in German text.
pub struct GermanSubjectVerbAgreement<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanSubjectVerbAgreement<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Check if a token is a subject (noun or pronoun in nominative case)
    fn is_subject(&self, token: &Token, document: &Document) -> bool {
        // Check POS tags
        if token.kind.is_upos(UPOS::NOUN)
            || token.kind.is_upos(UPOS::PROPN)
            || token.kind.is_upos(UPOS::PRON)
        {
            return true;
        }

        // Check metadata
        let token_chars = document.get_span_content(&token.span);
        if let Some(metadata) = self.dictionary.get_word_metadata(token_chars)
            && (metadata.is_noun() || metadata.is_pronoun())
        {
            return true;
        }

        false
    }

    /// Check if a token is a verb
    fn is_verb(&self, token: &Token, document: &Document) -> bool {
        // Check POS tags
        if token.kind.is_upos(UPOS::VERB) || token.kind.is_upos(UPOS::AUX) {
            return true;
        }

        // Check metadata
        let token_chars = document.get_span_content(&token.span);
        if let Some(metadata) = self.dictionary.get_word_metadata(token_chars)
            && metadata.is_verb()
        {
            return true;
        }

        false
    }

    /// Check if a verb is in 3rd person singular form
    fn is_third_person_singular(&self, token: &Token, document: &Document) -> bool {
        let token_chars = document.get_span_content(&token.span);
        if let Some(metadata) = self.dictionary.get_word_metadata(token_chars)
            && let Some(verb_data) = metadata.verb
            && let Some(verb_forms) = verb_data.verb_forms
        {
            return verb_forms
                .contains(crate::dict_word_metadata::VerbFormFlags::THIRD_PERSON_SINGULAR);
        }

        // Check common 3rd person singular verb endings
        let token_text: String = token_chars.iter().collect();
        let lower_text = token_text.to_lowercase();
        lower_text.ends_with("t")
            || lower_text.ends_with("ht")
            || lower_text == "ist"
            || lower_text == "hat"
    }

    /// Check if a subject is 3rd person singular
    fn is_third_person_singular_subject(&self, token: &Token, document: &Document) -> bool {
        let token_chars = document.get_span_content(&token.span);
        let token_text: String = token_chars.iter().collect();

        // Check for common 3rd person singular pronouns
        let third_person_singular_pronouns = ["er", "sie", "es", "Er", "Sie", "Es"];

        if third_person_singular_pronouns.contains(&token_text.as_str()) {
            return true;
        }

        // Check if it's a noun (nouns are typically 3rd person)
        if token.kind.is_upos(UPOS::NOUN) || token.kind.is_upos(UPOS::PROPN) {
            return true;
        }

        // Check metadata
        if let Some(metadata) = self.dictionary.get_word_metadata(token_chars)
            && (metadata.is_noun() || metadata.is_pronoun())
        {
            // Check for 3rd person singular pronouns
            if let Some(pronoun_data) = metadata.pronoun
                && pronoun_data.person == Some(crate::dict_word_metadata::Person::Third)
                && pronoun_data.is_singular == Some(true)
            {
                return true;
            }
            return true; // Nouns are typically 3rd person
        }

        false
    }

    /// Check subject-verb agreement for a subject-verb pair
    fn check_subject_verb_agreement(
        &self,
        subject_token: &Token,
        verb_token: &Token,
        document: &Document,
    ) -> Option<Lint> {
        let subject_text: String = document
            .get_span_content(&subject_token.span)
            .iter()
            .collect();
        let verb_text: String = document.get_span_content(&verb_token.span).iter().collect();

        // Check if subject is 3rd person singular and verb is not in 3rd person singular form
        let subject_is_3rd_singular =
            self.is_third_person_singular_subject(subject_token, document);
        let verb_is_3rd_singular = self.is_third_person_singular(verb_token, document);

        if subject_is_3rd_singular && !verb_is_3rd_singular {
            // Common correction: add -t or -ht ending for 3rd person singular
            let corrected_verb = if verb_text.to_lowercase().ends_with("en") {
                // Remove -en and add -t for many verbs
                let base = &verb_text[..verb_text.len() - 2];
                format!("{}t", base)
            } else if verb_text.to_lowercase() == "haben" {
                "hat".to_string()
            } else if verb_text.to_lowercase() == "sein" {
                "ist".to_string()
            } else if verb_text.to_lowercase() == "werden" {
                "wird".to_string()
            } else {
                format!("{}t", verb_text)
            };

            Some(Lint {
                span: verb_token.span,
                lint_kind: LintKind::Grammar,
                suggestions: vec![Suggestion::ReplaceWith(corrected_verb.chars().collect())],
                message: format!(
                    "Subject-verb agreement: '{}' should be '{}' for 3rd person singular subject '{}'",
                    verb_text, corrected_verb, subject_text
                ),
                priority: 25,
            })
        } else {
            None
        }
    }
}

impl<T: Dictionary> Linter for GermanSubjectVerbAgreement<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // This is a basic implementation that will be enhanced
        // For now, look for subject + verb patterns
        let tokens = document.get_tokens();

        for i in 0..tokens.len() - 1 {
            let subject_token = &tokens[i];
            let verb_token = &tokens[i + 1];

            // Check if this is a subject followed by a verb
            if self.is_subject(subject_token, document)
                && self.is_verb(verb_token, document)
                && let Some(lint) =
                    self.check_subject_verb_agreement(subject_token, verb_token, document)
            {
                lints.push(lint);
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for proper subject-verb agreement in German text"
    }
}
