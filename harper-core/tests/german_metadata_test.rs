#![cfg(feature = "de")]

// Tests for the morphology metadata the German dictionary carries.

mod tests {
    use harper_core::language::german::spell::curated_german_dictionary;
    use harper_core::language::morphology::{Gender, MorphologyExt, Number};

    /// Metadata access for case information
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

        let metadata = metadata.unwrap();
        assert!(metadata.is_noun(), "'Mann' should have a noun reading");

        // `Mann/~~MhY`: the M flag carries masculine gender, Y carries plural.
        assert_eq!(
            metadata.get_noun_gender(),
            Some(Gender::Masculine),
            "the M flag should give 'Mann' masculine noun gender"
        );
        assert_eq!(metadata.get_noun_number(), Some(Number::Plural));

        // The case flags (p/u/v/w) carry no metadata yet, so case stays unset.
        assert_eq!(metadata.get_noun_case(), None);
    }

    /// Metadata access for articles
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

        let metadata = metadata.unwrap();
        assert!(
            metadata.is_determiner(),
            "'der' should have a determiner reading"
        );

        // `der/~~DzMpY` carries noun gender via M/z, but the determiner reading
        // itself has no agreement features -- no flag sets them. The two must
        // stay separate: if the noun gender leaked into the determiner reading, an
        // agreement check would compare a value against itself and never fire.
        assert_eq!(
            metadata.get_determiner_gender(),
            None,
            "noun gender must not leak into the determiner reading"
        );
        assert_eq!(metadata.get_determiner_case(), None);

        // 'der' is an article, not a noun. The entry used to carry M and z as
        // well, claiming both a masculine and a neuter noun reading, which is
        // what made GermanNounCapitalization treat function words as nouns.
        assert!(
            !metadata.is_noun(),
            "'der' is an article and must not carry a noun reading"
        );
    }

    /// Metadata access for pronouns
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

        let metadata = metadata.unwrap();
        assert!(metadata.is_pronoun(), "'er' should have a pronoun reading");

        // `er/~~Ip`: I marks the pronoun, p is a case flag that carries no
        // metadata yet. Nothing in annotations.json sets pronoun agreement, so
        // these are all None.
        assert_eq!(metadata.get_pronoun_case(), None);
        assert_eq!(metadata.get_pronoun_gender(), None);
        assert_eq!(metadata.get_pronoun_number(), None);
    }
}
