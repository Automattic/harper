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
    linting::{Lint, LintKind, Linter, Suggestion},
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
        match preposition.to_lowercase().as_str() {
            // Accusative prepositions
            "durch" | "für" | "gegen" | "ohne" | "um" | "wider" => {
                Some(crate::dict_word_metadata::Case::Accusative)
            }
            // Dative prepositions
            "aus" | "außer" | "bei" | "mit" | "nach" | "seit" | "von" | "zu" => {
                Some(crate::dict_word_metadata::Case::Dative)
            }
            // Genitive prepositions
            "abseits" | "an Platz" | "an Stelle" | "an Statt" | "auf Grund" | "dank"
            | "durcheinander" | "halber" | "innerhalb" | "kraft" | "längs" | "mangels"
            | "trotz" | "während" | "wegen" => Some(crate::dict_word_metadata::Case::Genitive),
            // Two-way prepositions (depend on verb/position)
            "an" | "auf" | "hinter" | "in" | "neben" | "über" | "unter" | "vor" | "zwischen" => {
                None
            }
            _ => None,
        }
    }

    /// Analyze preposition + noun case usage
    fn check_preposition_case(
        &self,
        preposition_token: &Token,
        noun_token: &Token,
        document: &Document,
    ) -> Option<Lint> {
        let preposition_text: String = document
            .get_span_content(&preposition_token.span)
            .iter()
            .collect();
        let noun_text: String = document.get_span_content(&noun_token.span).iter().collect();

        // Check if this is a preposition
        if !preposition_token.kind.is_upos(UPOS::ADP) {
            return None;
        }

        // Get the required case for this preposition
        let required_case = match self.get_preposition_case_requirement(&preposition_text) {
            Some(case) => case,
            None => return None, // Skip two-way prepositions for now
        };

        // Get the actual case of the noun
        let noun_case = self.get_noun_case(noun_token, document)?;

        // Check for case mismatch
        if noun_case != required_case {
            let suggestion = match required_case {
                crate::dict_word_metadata::Case::Accusative => {
                    format!("{} (Accusative)", noun_text)
                }
                crate::dict_word_metadata::Case::Dative => format!("{} (Dative)", noun_text),
                crate::dict_word_metadata::Case::Genitive => format!("{} (Genitive)", noun_text),
                crate::dict_word_metadata::Case::Nominative => {
                    format!("{} (Nominative)", noun_text)
                }
            };

            Some(Lint {
                span: noun_token.span,
                lint_kind: LintKind::Grammar,
                suggestions: vec![Suggestion::ReplaceWith(suggestion.chars().collect())],
                message: format!(
                    "Possible case error: '{}' after '{}' may need to be in the {:?}",
                    noun_text, preposition_text, required_case
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
            let noun_token = &tokens[i + 1];

            // Check if this is a preposition followed by a noun
            if preposition_token.kind.is_upos(UPOS::ADP)
                && (noun_token.kind.is_upos(UPOS::NOUN) || noun_token.kind.is_upos(UPOS::PROPN))
            {
                if let Some(lint) =
                    self.check_preposition_case(preposition_token, noun_token, document)
                {
                    lints.push(lint);
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for proper case usage in German text"
    }
}
