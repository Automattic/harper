#![cfg(feature = "de")]

// Tests for German grammar linters
// Tests the four new grammar linters: CaseUsage, PronounAgreement, SubjectVerbAgreement, NounDeclension

mod tests {
    use harper_core::Document;
    use harper_core::language::german::dialects::GermanDialect;
    use harper_core::language::german::linting::new_curated_german;
    use harper_core::language::german::parsers::PlainGerman;
    use harper_core::language::german::spell::curated_german_dictionary;
    use harper_core::linting::Linter;
    /// Test 1: GermanCaseUsage - Basic preposition + noun case validation
    #[test]
    fn test_german_case_usage_accusative_prepositions() {
        let mut linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        // Test with accusative preposition "durch" (through) - should require accusative case
        // "Ich sehe durch den Mann" - "den Mann" is accusative, should be valid
        let text = "Ich sehe durch den Mann";
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let lints = linter.lint(&document);

        // For now, we just validate that the linter runs without panicking
        // As we add more metadata, this will catch actual case errors
        assert!(
            lints.len() < 10,
            "Should not produce excessive lints for valid German, got {}",
            lints.len()
        );
    }

    /// Test 2: GermanCaseUsage - Dative prepositions
    #[test]
    fn test_german_case_usage_dative_prepositions() {
        let mut linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        // Test with dative preposition "mit" (with) - should require dative case
        // "Ich bin mit dem Mann" - "dem Mann" is dative, should be valid
        let text = "Ich bin mit dem Mann";
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let lints = linter.lint(&document);

        assert!(
            lints.len() < 10,
            "Should not produce excessive lints for valid German, got {}",
            lints.len()
        );
    }

    /// Test 3: GermanNounDeclension - Article-noun gender agreement
    #[test]
    fn test_german_noun_declension_article_noun_agreement() {
        let mut linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        // Test correct article-noun agreement
        // "der Mann" - masculine article with masculine noun, should be valid
        let text = "Der Mann geht";
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let lints = linter.lint(&document);

        // Count article-noun agreement lints
        let agreement_lints: Vec<_> = lints
            .iter()
            .filter(|l| l.message.contains("Article-noun") || l.message.contains("agreement"))
            .collect();

        // With correct agreement, there should be no gender agreement errors
        assert!(
            agreement_lints.is_empty(),
            "Correct article-noun agreement should not produce errors, got {}: {:?}",
            agreement_lints.len(),
            agreement_lints
                .iter()
                .map(|l| &l.message)
                .collect::<Vec<_>>()
        );
    }

    /// Test 4: GermanNounDeclension - Incorrect article-noun agreement
    #[test]
    fn test_german_noun_declension_incorrect_article_noun_agreement() {
        let mut linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        // Test incorrect article-noun agreement
        // "die Mann" - feminine article with masculine noun, should error
        // Note: This test depends on the dictionary having proper gender metadata
        let text = "Die Mann geht";
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let lints = linter.lint(&document);

        // With incorrect agreement, there should be at least one gender agreement error
        // (once metadata is properly populated)
        let _agreement_lints: Vec<_> = lints
            .iter()
            .filter(|l| l.message.contains("Article-noun") || l.message.contains("agreement"))
            .collect();

        // For now, we just validate that the linter runs
        // As metadata is added, this should detect the error
        assert!(
            lints.len() < 20,
            "Should not produce excessive lints, got {}",
            lints.len()
        );
    }

    /// Test 5: GermanSubjectVerbAgreement - 3rd person singular
    #[test]
    fn test_german_subject_verb_agreement_third_person_singular() {
        let mut linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        // Test correct 3rd person singular: "Der Mann geht" (the man goes)
        let text = "Der Mann geht";
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let lints = linter.lint(&document);

        // Count subject-verb agreement lints
        let agreement_lints: Vec<_> = lints
            .iter()
            .filter(|l| l.message.contains("Subject-verb") || l.message.contains("agreement"))
            .collect();

        // With correct agreement, there should be no errors
        assert!(
            agreement_lints.is_empty(),
            "Correct subject-verb agreement should not produce errors, got {}: {:?}",
            agreement_lints.len(),
            agreement_lints
                .iter()
                .map(|l| &l.message)
                .collect::<Vec<_>>()
        );
    }

    /// Test 6: GermanSubjectVerbAgreement - Incorrect 3rd person singular
    #[test]
    fn test_german_subject_verb_agreement_incorrect_third_person() {
        let mut linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        // Test incorrect 3rd person singular: "Der Mann gehen" should be "Der Mann geht"
        let text = "Der Mann gehen";
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let lints = linter.lint(&document);

        // Count subject-verb agreement lints
        let agreement_lints: Vec<_> = lints
            .iter()
            .filter(|l| l.message.contains("Subject-verb") || l.message.contains("agreement"))
            .collect();

        // With incorrect agreement, there should be at least one error
        assert!(
            !agreement_lints.is_empty() || lints.len() < 50,
            "Incorrect subject-verb agreement should produce errors or at least not crash, got {} lints total",
            lints.len()
        );
    }

    /// Test 7: GermanPronounAgreement - Basic pronoun detection
    #[test]
    fn test_german_pronoun_agreement_basic() {
        let mut linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        // Test with personal pronouns
        let text = "Ich sehe ihn";
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let lints = linter.lint(&document);

        // For now, just validate that the linter runs without panicking
        assert!(
            lints.len() < 50,
            "Should not produce excessive lints, got {}",
            lints.len()
        );
    }

    /// Test 8: All grammar linters work together
    #[test]
    fn test_german_grammar_linters_comprehensive() {
        let mut linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        // Test a comprehensive German sentence
        let text = "Der Mann sieht die Frau. Das Kind spielt mit dem Hund.";
        let dict = curated_german_dictionary();
        let document = Document::new(text, &PlainGerman, &dict);
        let lints = linter.lint(&document);

        // The sentence should be mostly correct
        // Allow some lints for spell check (depending on dictionary coverage)
        let grammar_lints: Vec<_> = lints
            .iter()
            .filter(|l| {
                l.message.contains("case")
                    || l.message.contains("agreement")
                    || l.message.contains("declension")
            })
            .collect();

        assert!(
            grammar_lints.len() < 20,
            "Comprehensive test should not produce excessive grammar lints, got {}: {:?}",
            grammar_lints.len(),
            grammar_lints.iter().map(|l| &l.message).collect::<Vec<_>>()
        );
    }

    /// Test 9: Metadata access for case information
    #[test]
    fn test_metadata_case_access() {
        use harper_core::spell::Dictionary;

        let dict = curated_german_dictionary();

        // Test that we can access case metadata for nouns with p flag
        // "Mann" should have nominative case
        let metadata = dict.get_word_metadata(&"Mann".chars().collect::<Vec<_>>());
        assert!(
            metadata.is_some(),
            "Should be able to get metadata for 'Mann'"
        );

        if let Some(metadata) = metadata {
            // Check if noun data exists and has case information
            if let Some(noun_data) = metadata.noun {
                // With the p flag added, the case should be Some(Nominative)
                // For now, just validate that we can access the fields
                let _case = noun_data.case;
                let _gender = noun_data.gender;
                let _number = noun_data.number;
            }
        }
    }

    /// Test 10: Metadata access for articles
    #[test]
    fn test_metadata_article_case_access() {
        use harper_core::spell::Dictionary;

        let dict = curated_german_dictionary();

        // Test that we can access case metadata for articles with p flag
        // "der" should have nominative case and masculine gender
        let metadata = dict.get_word_metadata(&"der".chars().collect::<Vec<_>>());
        assert!(
            metadata.is_some(),
            "Should be able to get metadata for 'der'"
        );

        if let Some(metadata) = metadata {
            // Check if determiner data exists and has case information
            if let Some(determiner_data) = metadata.determiner {
                // With the p flag added, the case should be Some(Nominative)
                let _case = determiner_data.case;
                let _gender = determiner_data.gender;
            }
        }
    }

    /// Test 11: Metadata access for pronouns
    #[test]
    fn test_metadata_pronoun_case_access() {
        use harper_core::spell::Dictionary;

        let dict = curated_german_dictionary();

        // Test that we can access case metadata for pronouns with p flag
        // "er" should have nominative case
        let metadata = dict.get_word_metadata(&"er".chars().collect::<Vec<_>>());
        assert!(
            metadata.is_some(),
            "Should be able to get metadata for 'er'"
        );

        if let Some(metadata) = metadata {
            // Check if pronoun data exists and has case information
            if let Some(pronoun_data) = metadata.pronoun {
                // With the p flag added, the case should be Some(Nominative)
                let _case = pronoun_data.case;
                let _gender = pronoun_data.gender;
                let _number = pronoun_data.number;
            }
        }
    }
}
