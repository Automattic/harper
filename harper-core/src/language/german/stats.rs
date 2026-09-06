//! German language statistics tooling.
//!
//! This module provides dictionary/annotation statistics for German,
//! used by the `harper-lang-stats` binary.

use std::fs;

use hashbrown::HashMap;

use crate::language::german::spell::{curated_german_dictionary, mutable_german_dictionary};

/// Print German language statistics.
pub fn analyze(detailed: bool) {
    println!("📊 German Language Statistics");
    println!("==================================");

    // Read curated dictionary size from file (first line contains the count)
    let dict_path = "harper-core/src/language/german/dictionary.dict";
    let dict_size = if let Ok(content) = fs::read_to_string(dict_path) {
        if let Some(first_line) = content.lines().next() {
            first_line.trim().parse::<usize>().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    // Load FST dictionary for other stats
    let _dict = curated_german_dictionary();

    // Basic statistics - show file-based count
    println!("Dictionary Size: {} words", dict_size);

    // Annotation statistics - count from actual curated dictionary
    let mutable_dict = mutable_german_dictionary();

    // Count annotated words (words with any metadata)
    let annotated_count = mutable_dict
        .iter()
        .filter(|(_, metadata)| {
            metadata.noun.is_some()
                || metadata.verb.is_some()
                || metadata.adjective.is_some()
                || metadata.adverb.is_some()
                || metadata.pronoun.is_some()
                || metadata.conjunction.is_some()
                || metadata.determiner.is_some()
                || metadata.affix.is_some()
                || metadata.preposition
                || metadata.pos_tag.is_some()
                || !metadata.dialects.is_empty()
        })
        .count();
    let total_count = mutable_dict.len();
    println!(
        "Annotated Words: {} ({:.1}%)",
        annotated_count,
        (annotated_count as f64 / total_count as f64) * 100.0
    );

    if detailed {
        let mut annotation_counts: HashMap<String, usize> = HashMap::new();

        // Count POS types
        for (_, metadata) in mutable_dict.iter() {
            if metadata.noun.is_some() {
                *annotation_counts.entry("Noun".to_string()).or_insert(0) += 1;
            }
            if metadata.verb.is_some() {
                *annotation_counts.entry("Verb".to_string()).or_insert(0) += 1;
            }
            if metadata.adjective.is_some() {
                *annotation_counts
                    .entry("Adjective".to_string())
                    .or_insert(0) += 1;
            }
            if metadata.adverb.is_some() {
                *annotation_counts.entry("Adverb".to_string()).or_insert(0) += 1;
            }
            if metadata.pronoun.is_some() {
                *annotation_counts.entry("Pronoun".to_string()).or_insert(0) += 1;
            }
            if metadata.conjunction.is_some() {
                *annotation_counts
                    .entry("Conjunction".to_string())
                    .or_insert(0) += 1;
            }
            if metadata.determiner.is_some() {
                *annotation_counts
                    .entry("Determiner".to_string())
                    .or_insert(0) += 1;
            }
            if metadata.affix.is_some() {
                *annotation_counts.entry("Affix".to_string()).or_insert(0) += 1;
            }
            if metadata.preposition {
                *annotation_counts
                    .entry("Preposition".to_string())
                    .or_insert(0) += 1;
            }
        }

        let mut sorted: Vec<_> = annotation_counts.into_iter().collect();
        sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
        println!("\nAnnotation Types:");
        for (annotation, count) in &sorted {
            println!("  {}: {} words", annotation, count);
        }
    }

    // Affix rules - count unique affix flags from the annotations file
    // The annotations.json file defines affix rules
    let annotations_path = "harper-core/src/language/german/annotations.json";
    let affix_count = if let Ok(contents) = fs::read_to_string(annotations_path) {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(affixes) = parsed.get("affixes").and_then(|v| v.as_object()) {
                affixes.len()
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };
    println!("Affix Rules: {} rules", affix_count);

    println!("==================================");
}
