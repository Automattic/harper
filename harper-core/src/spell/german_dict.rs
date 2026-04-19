//! Minimal German dictionary for MVP testing
//!
//! This provides a basic German dictionary with common words and their grammatical metadata.
//! For production use, this should be replaced with a comprehensive dictionary converted
//! from Hunspell sources.

use crate::spell::Dictionary;
use crate::{DictWordMetadata, NounData, VerbData};

/// A minimal German dictionary for testing and MVP demonstration
pub struct GermanDictionary {
    words: std::collections::HashMap<Vec<char>, DictWordMetadata>,
}

impl GermanDictionary {
    /// Create a new German dictionary with common words
    pub fn new() -> Self {
        let mut words = std::collections::HashMap::new();

        // Common German articles and determiners
        let articles = [
            "der", "die", "das", // definite articles
            "ein", "eine", "einen", "einem", "einen", "einer", // indefinite articles
            "mein", "dein", "sein", "ihr", "unser", "euer", "ihr", // possessive articles
            "dieser", "jener", "jeder", "mancher", "welcher", // demonstrative
            "alle", "keine", "viele", "wenige", "einige", "mehrere", // quantifiers
        ];

        // Common German nouns (must be capitalized in German)
        let nouns = [
            // Animals
            "Hund", "Katze", "Vogel", "Fisch", "Pferd", "Kuh", "Schaf", "Huhn",
            "Maus", "Elefant", "Löwe", "Tiger", "Bär", "Hirsch", "Fuchs",
            // People and family
            "Mann", "Frau", "Kind", "Junge", "Mädchen", "Eltern", "Vater", "Mutter",
            "Bruder", "Schwester", "Freund", "Freundin",
            // Time and seasons
            "Tag", "Nacht", "Woche", "Monat", "Jahr", "Stunde", "Minute", "Sekunde",
            "Frühling", "Sommer", "Herbst", "Winter", "Montag", "Dienstag", "Mittwoch",
            "Donnerstag", "Freitag", "Samstag", "Sonntag", "Januar", "Februar", "März",
            // Abstract nouns with common suffixes
            "Freiheit", "Schönheit", "Wahrheit", "Klarheit", "Warmheit", "Kälte",
            "Möglichkeit", "Gesundheit", "Zeit", "Zeitlichkeit",
            "Wirtschaft", "Gesellschaft", "Gemeinschaft", "Wissenschaft", "Kunst",
            "Freude", "Erfüllung", "Bildung", "Meinung", "Vorstellung", "Unterhaltung",
            "Musik", "Politik", "Kritik", "Philosophie", "Biologie", "Theologie",
            "Qualität", "Universität", "Fabrik", "Bank", "Buchhandlung",
            // Objects and places
            "Haus", "Wohnung", "Zimmer", "Tür", "Fenster", "Tisch", "Stuhl", "Bett",
            "Auto", "Fahrrad", "Straße", "Stadt", "Land", "Welt", "Universum",
            "Buch", "Stift", "Papier", "Computer", "Telefon", "Internet", "Welt",
            "Garten", "Park", "Wald", "Berg", "Fluss", "See", "Meer", "Ozean",
            "Schule", "Universität", "Büro", "Geschäft", "Restaurant", "Krankenhaus",
            // Food and drink
            "Brot", "Wasser", "Milch", "Kaffee", "Tee", "Bier", "Wein", "Kuchen",
            "Apfel", "Banane", "Kartoffel", "Tomate", "Salat", "Fleisch", "Fisch",
            // Concepts
            "Liebe", "Hass", "Sorge", "Glück", "Erfolg", "Problem", "Lösung",
            "Frage", "Antwort", "Idee", "Gedanke", "Geist", "Seele", "Herz",
            "Anfang", "Ende", "Mitte", "Seite", "Mitte",
        ];

        // Common German verbs
        let verbs = [
            "sein", "haben", "werden", // auxiliary verbs
            "gehen", "kommen", "laufen", "rennen", "springen", // movement
            "sagen", "sprechen", "reden", "erzählen", "fragen", "antworten", // communication
            "sehen", "hören", "fühlen", "riechen", "schmecken", // senses
            "denken", "glauben", "wissen", "verstehen", "lernen", // mental
            "essen", "trinken", "schlafen", "wachen", // basic needs
            "arbeiten", "spielen", "lesen", "schreiben", "machen", "tun", // activities
            "lieben", "mögen", "hassen", "fürchten", // emotions
            "öffnen", "schließen", "beginnen", "enden", // changes
            "kaufen", "verkaufen", "geben", "nehmen", // transactions
        ];

        // Add articles
        for article in &articles {
            let chars: Vec<char> = article.chars().collect();
            words.insert(chars, DictWordMetadata {
                determiner: Some(crate::DeterminerData::default()),
                ..Default::default()
            });
        }

        // Add nouns (with capitalization - German requires all nouns to be capitalized)
        for noun in &nouns {
            let chars: Vec<char> = noun.chars().collect();
            words.insert(chars.clone(), DictWordMetadata {
                noun: Some(NounData::default()),
                orth_info: crate::dict_word_metadata_orthography::OrthFlags::TITLECASE,
                ..Default::default()
            });

            // Also add lowercase version for spell checking
            let lowercase: Vec<char> = noun.to_lowercase().chars().collect();
            words.insert(lowercase, DictWordMetadata {
                noun: Some(NounData::default()),
                ..Default::default()
            });
        }

        // Add verbs
        for verb in &verbs {
            let chars: Vec<char> = verb.chars().collect();
            words.insert(chars, DictWordMetadata {
                verb: Some(VerbData::default()),
                ..Default::default()
            });
        }

        // Add common prepositions
        let prepositions = [
            "in", "auf", "unter", "über", "neben", "hinter", "vor", "zwischen",
            "bei", "nach", "von", "zu", "mit", "ohne", "seit", "während",
            "durch", "für", "gegen", "aus", "bei", "nach",
        ];

        for prep in &prepositions {
            let chars: Vec<char> = prep.chars().collect();
            words.insert(chars, DictWordMetadata {
                preposition: true,
                ..Default::default()
            });
        }

        // Add common pronouns
        let pronouns = [
            "ich", "du", "er", "sie", "es", "wir", "ihr", "sie", "Sie",
            "mein", "dein", "sein", "ihr", "unser", "euer",
            "dieser", "jener", "derselbe",
            "wer", "was", "wann", "wo", "wie", "warum", "weshalb", "wieso",
        ];

        for pronoun in &pronouns {
            let chars: Vec<char> = pronoun.chars().collect();
            words.insert(chars, DictWordMetadata {
                pronoun: Some(crate::PronounData::default()),
                ..Default::default()
            });
        }

        // Add common conjunctions
        let conjunctions = [
            "und", "oder", "aber", "denn", "weil", "wenn", "ob", "obwohl",
            "falls", "sobald", "während", "seit", "bis", "ehe", "bevor",
        ];

        for conj in &conjunctions {
            let chars: Vec<char> = conj.chars().collect();
            words.insert(chars, DictWordMetadata {
                conjunction: Some(crate::ConjunctionData::default()),
                ..Default::default()
            });
        }

        // Add common adverbs
        let adverbs = [
            "ja", "nein", "vielleicht", "sicher", "bestimmt", "wahrscheinlich",
            "hier", "dort", "da", "überall", "nirgendwo", "irgendwo",
            "jetzt", "damals", "später", "früher", "sofort", "gleich",
            "sehr", "ziemlich", "ganz", "fast", "kaum", "überhaupt",
            "auch", "nur", "noch", "schon", "immer", "nie", "oft", "manchmal",
        ];

        for adverb in &adverbs {
            let chars: Vec<char> = adverb.chars().collect();
            words.insert(chars, DictWordMetadata {
                adverb: Some(crate::AdverbData::default()),
                ..Default::default()
            });
        }

        Self { words }
    }

    /// Check if a word contains a common German noun suffix
    pub fn has_noun_suffix(&self, word: &[char]) -> bool {
        let suffixes: &[&[char]] = &[
            &['h', 'e', 'i', 't'],     // -heit
            &['k', 'e', 'i', 't'],     // -keit
            &['u', 'n', 'g'],          // -ung
            &['s', 'c', 'h', 'a', 'f', 't'], // -schaft
            &['i', 'k'],               // -ik
            &['i', 'o', 'n'],          // -ion
            &['t', 'ä', 't'],          // -tät
        ];

        for suffix in suffixes {
            if word.len() > suffix.len() {
                let word_suffix = &word[word.len() - suffix.len()..];
                if word_suffix == *suffix {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for GermanDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Dictionary for GermanDictionary {
    fn get_word_metadata(&self, word: &[char]) -> Option<std::borrow::Cow<'_, DictWordMetadata>> {
        self.words.get(word).map(|metadata| std::borrow::Cow::Borrowed(metadata))
    }

    fn contains_word(&self, word: &[char]) -> bool {
        self.words.contains_key(word)
    }

    fn contains_word_str(&self, word: &str) -> bool {
        let chars: Vec<char> = word.chars().collect();
        self.contains_word(&chars)
    }

    fn contains_exact_word(&self, word: &[char]) -> bool {
        self.contains_word(word)
    }

    fn contains_exact_word_str(&self, word: &str) -> bool {
        self.contains_word_str(word)
    }

    fn fuzzy_match(
        &'_ self,
        word: &[char],
        _max_distance: u8,
        _max_results: usize,
    ) -> Vec<super::FuzzyMatchResult<'_>> {
        // For MVP, return empty
        Vec::new()
    }

    fn fuzzy_match_str(
        &'_ self,
        word: &str,
        max_distance: u8,
        max_results: usize,
    ) -> Vec<super::FuzzyMatchResult<'_>> {
        let chars: Vec<char> = word.chars().collect();
        self.fuzzy_match(&chars, max_distance, max_results)
    }

    fn get_correct_capitalization_of(&self, word: &[char]) -> Option<&'_ [char]> {
        // For MVP, return None
        // A proper implementation would check if word should be capitalized (German nouns)
        None
    }

    fn get_word_metadata_str(&self, word: &str) -> Option<std::borrow::Cow<'_, DictWordMetadata>> {
        let chars: Vec<char> = word.chars().collect();
        self.get_word_metadata(&chars)
    }

    fn words_iter(&self) -> Box<dyn Iterator<Item = &'_ [char]> + Send + '_> {
        Box::new(self.words.keys().map(|v| v.as_slice()))
    }

    fn word_count(&self) -> usize {
        self.words.len()
    }

    fn get_word_from_id(&self, _id: &super::WordId) -> Option<&[char]> {
        None
    }

    fn find_words_with_prefix(&self, _prefix: &[char]) -> Vec<std::borrow::Cow<'_, [char]>> {
        Vec::new()
    }

    fn find_words_with_common_prefix(&self, _word: &[char]) -> Vec<std::borrow::Cow<'_, [char]>> {
        Vec::new()
    }
}