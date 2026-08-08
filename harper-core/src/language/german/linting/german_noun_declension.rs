//! German noun declension linter.
//!
//! This linter checks for proper noun declension in German text.
//! In German, nouns, adjectives, and determiners must be properly declined based on case and number.
//!
//! The linter detects patterns like:
//! - "der gute Mann" ✓ (correct nominative)
//! - "den guten Mann" ✓ (correct accusative)
//! - "dem guten Mann" ✓ (correct dative)
//! - "des guten Mannes" ✓ (correct genitive)
//!
//! Implementation approach:
//! 1. Use dictionary metadata to get noun gender, number, and case information
//! 2. Validate noun forms based on their grammatical context
//! 3. Start with basic patterns, expand to complex cases

use crate::{
    Token,
    document::Document,
    linting::{Lint, LintKind, Linter, Suggestion},
    spell::Dictionary,
};
use harper_brill::UPOS;

/// A linter that checks for proper noun declension in German text.
pub struct GermanNounDeclension<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanNounDeclension<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    /// Check if a noun has gender metadata
    fn get_noun_gender(
        &self,
        token: &Token,
        document: &Document,
    ) -> Option<crate::dict_word_metadata::Gender> {
        let token_chars = document.get_span_content(&token.span);
        self.dictionary
            .get_word_metadata(token_chars)
            .and_then(|metadata| metadata.get_noun_gender())
    }

    /// Check if a noun has number metadata
    fn get_noun_number(
        &self,
        token: &Token,
        document: &Document,
    ) -> Option<crate::dict_word_metadata::Number> {
        let token_chars = document.get_span_content(&token.span);
        self.dictionary
            .get_word_metadata(token_chars)
            .and_then(|metadata| metadata.get_noun_number())
    }

    /// Check if a determiner has gender metadata
    fn get_determiner_gender(
        &self,
        token: &Token,
        document: &Document,
    ) -> Option<crate::dict_word_metadata::Gender> {
        let token_chars = document.get_span_content(&token.span);
        self.dictionary
            .get_word_metadata(token_chars)
            .and_then(|metadata| metadata.get_determiner_gender())
    }

    /// Check if a determiner has case metadata
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

    /// Check article-noun gender agreement
    fn check_article_noun_gender_agreement(
        &self,
        article_token: &Token,
        noun_token: &Token,
        document: &Document,
    ) -> Option<Lint> {
        let article_text: String = document
            .get_span_content(&article_token.span)
            .iter()
            .collect();
        let noun_text: String = document.get_span_content(&noun_token.span).iter().collect();

        // Get gender information
        let article_gender = self.get_determiner_gender(article_token, document);
        let noun_gender = self.get_noun_gender(noun_token, document);

        // If we have gender metadata for both, check agreement
        if let (Some(art_gender), Some(noun_gender)) = (article_gender, noun_gender) {
            if art_gender != noun_gender {
                let expected_article = match (noun_gender, noun_text.chars().next().unwrap_or(' '))
                {
                    (crate::dict_word_metadata::Gender::Masculine, c) if c.is_uppercase() => "Der",
                    (crate::dict_word_metadata::Gender::Masculine, _) => "der",
                    (crate::dict_word_metadata::Gender::Feminine, c) if c.is_uppercase() => "Die",
                    (crate::dict_word_metadata::Gender::Feminine, _) => "die",
                    (crate::dict_word_metadata::Gender::Neuter, c) if c.is_uppercase() => "Das",
                    (crate::dict_word_metadata::Gender::Neuter, _) => "das",
                };

                Some(Lint {
                    span: article_token.span,
                    lint_kind: LintKind::Grammar,
                    suggestions: vec![Suggestion::ReplaceWith(expected_article.chars().collect())],
                    message: format!(
                        "Article-noun gender agreement: '{}' should be '{}' for {} noun '{}'",
                        article_text,
                        expected_article,
                        match noun_gender {
                            crate::dict_word_metadata::Gender::Masculine => "masculine",
                            crate::dict_word_metadata::Gender::Feminine => "feminine",
                            crate::dict_word_metadata::Gender::Neuter => "neuter",
                        },
                        noun_text
                    ),
                    priority: 25,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<T: Dictionary> Linter for GermanNounDeclension<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        // This is a basic implementation that will be enhanced
        // For now, look for article + noun patterns
        let tokens = document.get_tokens();

        for i in 0..tokens.len() - 1 {
            let article_token = &tokens[i];
            let noun_token = &tokens[i + 1];

            // Check if this is a determiner followed by a noun
            if (article_token.kind.is_upos(UPOS::DET) || article_token.kind.is_upos(UPOS::PRON))
                && (noun_token.kind.is_upos(UPOS::NOUN) || noun_token.kind.is_upos(UPOS::PROPN))
            {
                if let Some(lint) =
                    self.check_article_noun_gender_agreement(article_token, noun_token, document)
                {
                    lints.push(lint);
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for proper noun declension in German text"
    }
}
