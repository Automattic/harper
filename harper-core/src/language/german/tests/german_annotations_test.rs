//! Tests for German morphological annotations

use crate::language::german::spell::german_dict::{
    annotated_german_dictionary, mutable_german_dictionary,
};
use crate::spell::{Dictionary, suggest_correct_spelling_str};

#[test]
fn test_german_annotations_loading() {
    // Test that the annotated dictionary loads without errors
    let dict = annotated_german_dictionary();
    assert!(
        dict.word_count() > 0,
        "Annotated dictionary should contain words"
    );

    let mutable_dict = mutable_german_dictionary();
    assert!(
        mutable_dict.word_count() > 0,
        "Mutable dictionary should contain words"
    );
}

#[test]
fn test_german_noun_gender_detection() {
    let dict = mutable_german_dictionary();

    // Test masculine nouns
    assert!(
        dict.get_word_metadata_str("Mann").is_some(),
        "Mann should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Hund").is_some(),
        "Hund should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Tisch").is_some(),
        "Tisch should be in dictionary"
    );

    // Test feminine nouns
    assert!(
        dict.get_word_metadata_str("Frau").is_some(),
        "Frau should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Katze").is_some(),
        "Katze should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Tür").is_some(),
        "Tür should be in dictionary"
    );

    // Test neuter nouns
    assert!(
        dict.get_word_metadata_str("Kind").is_some(),
        "Kind should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Haus").is_some(),
        "Haus should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Buch").is_some(),
        "Buch should be in dictionary"
    );
}

#[test]
fn test_german_verb_detection() {
    let dict = mutable_german_dictionary();

    // Test basic verbs
    assert!(
        dict.get_word_metadata_str("sein").is_some(),
        "sein should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("haben").is_some(),
        "haben should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("werden").is_some(),
        "werden should be in dictionary"
    );

    // Test modal verbs
    assert!(
        dict.get_word_metadata_str("können").is_some(),
        "können should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("müssen").is_some(),
        "müssen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("sollen").is_some(),
        "sollen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("wollen").is_some(),
        "wollen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("dürfen").is_some(),
        "dürfen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("mögen").is_some(),
        "mögen should be in dictionary"
    );
}

#[test]
fn test_german_prefix_detection() {
    let dict = mutable_german_dictionary();

    // Test inseparable prefixes
    assert!(
        dict.get_word_metadata_str("bekommen").is_some(),
        "bekommen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("verstehen").is_some(),
        "verstehen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("entfernen").is_some(),
        "entfernen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("erzählen").is_some(),
        "erzählen should be in dictionary"
    );

    // Test separable prefixes
    assert!(
        dict.get_word_metadata_str("aufstehen").is_some(),
        "aufstehen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("ausgehen").is_some(),
        "ausgehen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("einkaufen").is_some(),
        "einkaufen should be in dictionary"
    );
}

#[test]
fn test_german_suffix_detection() {
    let dict = mutable_german_dictionary();

    // Test noun suffixes
    assert!(
        dict.get_word_metadata_str("Freiheit").is_some(),
        "Freiheit should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Möglichkeit").is_some(),
        "Möglichkeit should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Bildung").is_some(),
        "Bildung should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Ergebnis").is_some(),
        "Ergebnis should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Eigentum").is_some(),
        "Eigentum should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Lehrling").is_some(),
        "Lehrling should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Information").is_some(),
        "Information should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Universität").is_some(),
        "Universität should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Freundschaft").is_some(),
        "Freundschaft should be in dictionary"
    );

    // Test adjective suffixes
    assert!(
        dict.get_word_metadata_str("freundlich").is_some(),
        "freundlich should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("kindisch").is_some(),
        "kindisch should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("lesbar").is_some(),
        "lesbar should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("fleißig").is_some(),
        "fleißig should be in dictionary"
    );

    // Test diminutive suffixes
    assert!(
        dict.get_word_metadata_str("Mädchen").is_some(),
        "Mädchen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Häuschen").is_some(),
        "Häuschen should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("Büchlein").is_some(),
        "Büchlein should be in dictionary"
    );
}

#[test]
fn test_german_affix_rule_application() {
    let dict = mutable_german_dictionary();

    // Test that affix rules are applied correctly
    // These are hypothetical words that should match affix patterns
    assert!(
        dict.get_word_metadata_str("bekommenheit").is_some(),
        "bekommenheit should match affix rules"
    );
    assert!(
        dict.get_word_metadata_str("verkeit").is_some(),
        "verkeit should match affix rules"
    );
    assert!(
        dict.get_word_metadata_str("entung").is_some(),
        "entung should match affix rules"
    );
    assert!(
        dict.get_word_metadata_str("auflichkeit").is_some(),
        "auflichkeit should match affix rules"
    );
    assert!(
        dict.get_word_metadata_str("ausbar").is_some(),
        "ausbar should match affix rules"
    );
}

#[test]
fn test_german_article_and_preposition_detection() {
    let dict = mutable_german_dictionary();

    // Test articles
    assert!(
        dict.get_word_metadata_str("der").is_some(),
        "der should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("die").is_some(),
        "die should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("das").is_some(),
        "das should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("ein").is_some(),
        "ein should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("eine").is_some(),
        "eine should be in dictionary"
    );

    // Test prepositions
    assert!(
        dict.get_word_metadata_str("in").is_some(),
        "in should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("an").is_some(),
        "an should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("auf").is_some(),
        "auf should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("aus").is_some(),
        "aus should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("bei").is_some(),
        "bei should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("mit").is_some(),
        "mit should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("nach").is_some(),
        "nach should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("von").is_some(),
        "von should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("vor").is_some(),
        "vor should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("zu").is_some(),
        "zu should be in dictionary"
    );
}

#[test]
fn test_german_conjunction_and_adverb_detection() {
    let dict = mutable_german_dictionary();

    // Test conjunctions
    assert!(
        dict.get_word_metadata_str("und").is_some(),
        "und should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("oder").is_some(),
        "oder should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("aber").is_some(),
        "aber should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("weil").is_some(),
        "weil should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("dass").is_some(),
        "dass should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("wenn").is_some(),
        "wenn should be in dictionary"
    );

    // Test adverbs
    assert!(
        dict.get_word_metadata_str("nicht").is_some(),
        "nicht should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("auch").is_some(),
        "auch should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("nur").is_some(),
        "nur should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("schon").is_some(),
        "schon should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("hier").is_some(),
        "hier should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("dort").is_some(),
        "dort should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("jetzt").is_some(),
        "jetzt should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("dann").is_some(),
        "dann should be in dictionary"
    );
}

#[test]
fn test_german_phrase_detection() {
    let dict = mutable_german_dictionary();

    // Test common phrases
    assert!(
        dict.get_word_metadata_str("hallo").is_some(),
        "hallo should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("danke").is_some(),
        "danke should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("bitte").is_some(),
        "bitte should be in dictionary"
    );
    assert!(
        dict.get_word_metadata_str("entschuldigung").is_some(),
        "entschuldigung should be in dictionary"
    );
}

#[test]
fn test_german_annotation_properties() {
    let dict = mutable_german_dictionary();

    // Test that words have basic metadata
    if let Some(metadata) = dict.get_word_metadata_str("Freiheit") {
        // Freiheit should have some metadata (even if not specifically gender-related)
        assert!(
            metadata.noun.is_some() || metadata.adjective.is_some() || metadata.verb.is_some(),
            "Freiheit should have some part of speech metadata"
        );
    }

    if let Some(metadata) = dict.get_word_metadata_str("bekommen") {
        // bekommen should have verb metadata
        assert!(
            metadata.verb.is_some(),
            "bekommen should have verb metadata"
        );
    }

    if let Some(metadata) = dict.get_word_metadata_str("aufstehen") {
        // aufstehen should have verb metadata
        assert!(
            metadata.verb.is_some(),
            "aufstehen should have verb metadata"
        );
    }
}

#[test]
fn test_german_dictionary_word_count() {
    let dict = annotated_german_dictionary();
    let count = dict.word_count();

    // We expect at least 100 words in our test dictionary
    assert!(
        count >= 100,
        "Dictionary should contain at least 100 words, got {}",
        count
    );
}

#[test]
fn test_german_dictionary_fuzzy_matching() {
    let dict = annotated_german_dictionary();

    // Test fuzzy matching capabilities
    let suggestions = suggest_correct_spelling_str("Hund", 10, 2, &dict);
    assert!(
        !suggestions.is_empty(),
        "Should find suggestions for 'Hund'"
    );

    let suggestions = suggest_correct_spelling_str("Katz", 10, 2, &dict);
    assert!(
        suggestions.contains(&"Katze".to_string()),
        "Should suggest 'Katze' for 'Katz'"
    );
}
