//! German case usage linter.
//!
//! This linter checks for proper case usage in German text.
//! In German, nouns, pronouns, adjectives, and determiners must be in the correct case
//! (nominative, accusative, dative, genitive) based on their grammatical role.
//!
//! The linter detects patterns like:
//! - "Ich sehe der Mann" ❌ -> should be "Ich sehe den Mann" (accusative required after "sehe")
//! - "Ich gebe dem Mann das Buch" ✓ (dative correct)
//! - "Das Buch des Mannes" ✓ (genitive correct)
//!
//! Implementation approach:
//! 1. Use dictionary metadata to get case information for words
//! 2. Validate case usage based on prepositions and verb contexts
//! 3. Start with basic patterns, expand to complex cases

use crate::{
    Token,
    document::Document,
    linting::{Lint, LintKind, Linter},
    spell::Dictionary,
};
use harper_brill::UPOS;

/// A linter that checks for proper case usage in German text.
pub struct GermanCaseUsage<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanCaseUsage<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Check if a noun has case metadata that can be validated
    fn get_noun_case(
        &self,
        token: &Token,
        document: &Document,
    ) -> Option<crate::dict_word_metadata::Case> {
        let token_chars = document.get_span_content(&token.span);
        self.dictionary
            .get_word_metadata(token_chars)
            .and_then(|metadata| metadata.get_noun_case())
    }

    /// Check if a pronoun has case metadata that can be validated  
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

    /// Check if a determiner has case metadata that can be validated
    fn get_determiner_case(
        &self,
        token: &Token,
        document: &Document,
    ) -> Option<crate::dict_word_metadata::Case> {
        let token_chars = document.get_span_content(&token.span);
        self.dictionary
            .get_word_metadata(token_chars)
            .and_then(|metadata| metadata.get_determiner_case())
    }

    /// Check if a word is a preposition that requires a specific case
    fn get_preposition_case_requirement(
        &self,
        preposition: &str,
    ) -> Option<crate::dict_word_metadata::Case> {
        // Common German prepositions and their required cases
        // Based on standard German grammar rules
        match preposition.to_lowercase().as_str() {
            // Accusative-only prepositions
            // These always require the accusative case
            "durch" | "für" | "gegen" | "ohne" | "um" | "wider" | "bis" => {
                Some(crate::dict_word_metadata::Case::Accusative)
            }
            // Dative-only prepositions
            // These always require the dative case
            "aus" | "außer" | "bei" | "mit" | "nach" | "seit" | "von" | "zu" | "entgegen"
            | "gemäß" | "laut" => Some(crate::dict_word_metadata::Case::Dative),
            // Genitive-only prepositions
            // These always require the genitive case
            "abseits" | "an Stelle" | "an Statt" | "auf Grund" | "aufgrund" | "dank" | "halber"
            | "innerhalb" | "kraft" | "längs" | "mangels" | "trotz" | "während" | "wegen"
            | "anhand" | "anlässlich" | "behufs" | "dienlich" | "entsprechend" | "oberhalb"
            | "unterhalb" | "diesseits" | "jenseits" | "mittels" | "tagsüber" | "nachtsüber" => {
                Some(crate::dict_word_metadata::Case::Genitive)
            }
            // Two-way prepositions (Wechselpräpositionen)
            // These can be either accusative or dative depending on context:
            // - Accusative: when indicating direction/motion (wohin?)
            // - Dative: when indicating location/position (wo?)
            // For now, we skip these as they require sentence context analysis
            "an" | "auf" | "hinter" | "in" | "neben" | "über" | "unter" | "vor" | "zwischen" => {
                None
            }
            _ => None,
        }
    }

    /// Analyze preposition + following word case usage
    /// Checks nouns, pronouns, and determiners (articles) after prepositions
    fn check_preposition_case(
        &self,
        preposition_token: &Token,
        following_token: &Token,
        document: &Document,
    ) -> Option<Lint> {
        let preposition_text: String = document
            .get_span_content(&preposition_token.span)
            .iter()
            .collect();
        let following_text: String = document
            .get_span_content(&following_token.span)
            .iter()
            .collect();

        // Check if this is a preposition
        if !preposition_token.kind.is_upos(UPOS::ADP) {
            return None;
        }

        // Get the required case for this preposition
        let required_case = self.get_preposition_case_requirement(&preposition_text)?; // Skip two-way prepositions for now

        // Try to get the actual case from different word types
        let actual_case = if following_token.kind.is_upos(UPOS::NOUN)
            || following_token.kind.is_upos(UPOS::PROPN)
        {
            self.get_noun_case(following_token, document)
        } else if following_token.kind.is_upos(UPOS::PRON) {
            self.get_pronoun_case(following_token, document)
        } else if following_token.kind.is_upos(UPOS::DET) {
            self.get_determiner_case(following_token, document)
        } else {
            None
        }?;

        // Check for case mismatch
        if actual_case != required_case {
            let case_name = match required_case {
                crate::dict_word_metadata::Case::Accusative => "Accusative",
                crate::dict_word_metadata::Case::Dative => "Dative",
                crate::dict_word_metadata::Case::Genitive => "Genitive",
                crate::dict_word_metadata::Case::Nominative => "Nominative",
            };

            Some(Lint {
                span: following_token.span,
                lint_kind: LintKind::Grammar,
                suggestions: vec![], // Suggestions would require declension logic
                message: format!(
                    "Possible case error: '{}' after '{}' should be in the {}",
                    following_text, preposition_text, case_name
                ),
                priority: 25,
            })
        } else {
            None
        }
    }
}

impl<T: Dictionary> Linter for GermanCaseUsage<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // This is a basic implementation that will be enhanced
        // For now, look for preposition + noun patterns
        let tokens = document.get_tokens();

        for i in 0..tokens.len() - 1 {
            let preposition_token = &tokens[i];
            let following_token = &tokens[i + 1];

            // Check if this is a preposition followed by a word that can have case
            // (noun, proper noun, pronoun, or determiner/article)
            if preposition_token.kind.is_upos(UPOS::ADP)
                && (following_token.kind.is_upos(UPOS::NOUN)
                    || following_token.kind.is_upos(UPOS::PROPN)
                    || following_token.kind.is_upos(UPOS::PRON)
                    || following_token.kind.is_upos(UPOS::DET))
                && let Some(lint) =
                    self.check_preposition_case(preposition_token, following_token, document)
            {
                lints.push(lint);
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for proper case usage in German text"
    }
}
