#![doc = include_str!("../README.md")]

use harper_core::{Document, TokenKind};
use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::env::args;
use std::fs;
use std::{io, io::Read};

posslq_macros::build_posslq_matrix!();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct SlqPair {
    left: PosslqPropertyField,
    right: PosslqPropertyField,
}

// Custom data container to track the rich co-occurrence data
#[derive(Debug, Clone, Default)]
struct SlqTallyData<'a> {
    total_count: usize,
    word_pairs: HashMap<(&'a [char], &'a [char]), usize>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    analyse_file()?;
    Ok(())
}

fn analyse_file() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HARPER COMBINATORIAL SCHEMATIC SCHEMAS ===");

    // Instantiate dynamic mock examples of all supported categories to extract fields
    // TODO: This is brittle! We're using macros and the AST to get this info from the source to avoid this!
    let mock_categories = vec![
        PosslqPropertyField::Verb(0),
        PosslqPropertyField::Noun(0),
        PosslqPropertyField::Pronoun(0),
        PosslqPropertyField::Determiner(0),
        PosslqPropertyField::Adjective(0),
        PosslqPropertyField::Adverb(0),
        PosslqPropertyField::Conjunction(0),
        PosslqPropertyField::Affix(0),
        PosslqPropertyField::Preposition(0),
        PosslqPropertyField::OutOfVocabulary(0),
    ];

    for category in mock_categories {
        println!("POS Variant Family: {}", category.variant_name());
        let schematic = category.field_schematic();

        if schematic.is_empty() {
            println!("  [No sub-properties registered]");
        } else {
            for (index, (name, ty)) in schematic.iter().enumerate() {
                // Display exactly what placeholder marker is assigned to this slot
                let marker_char = if *ty == "Option<bool>" { "T/F/-" } else { "?" };
                println!(
                    "    [{:02}] -> {:<16} | Type: {:<18} | String Slot: {}",
                    index, name, ty, marker_char
                );
            }
        }
        println!();
    }
    println!("================================================\n");

    let content = if let Some(file) = args().nth(1) {
        fs::read_to_string(file).expect("Failed to read file")
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    };
    let doc = Document::new_plain_english_curated(&content);

    let toks: Vec<_> = doc.tokens().collect();
    let src = doc.get_source();

    let mut shadow_lane: Vec<Vec<PosslqPropertyField>> = Vec::new();
    let mut single_pos_tally: HashMap<PosslqPropertyField, usize> = HashMap::new();

    let mut pair_pos_tally: HashMap<SlqPair, SlqTallyData> = HashMap::new();
    let mut pair_word_tally: HashMap<(&[char], &[char]), HashSet<SlqPair>> = HashMap::new();

    // 1. Pass One: Token Stream Tokenization & Single Value Analysis
    for tok in toks.iter() {
        let _t = tok.get_ch(src);
        let mut token_pos_matches = Vec::new();

        let packed_fields: Cow<[PosslqPropertyField]> = match &tok.kind {
            TokenKind::Word(Some(wmd)) => Cow::Owned(PosslqPropertyField::from_metadata(wmd)),
            TokenKind::Word(None) => Cow::Borrowed(&[PosslqPropertyField::OutOfVocabulary(0)]),
            _ => Cow::Borrowed(&[]),
        };
        if !packed_fields.is_empty() {
            for &field in packed_fields.iter() {
                *single_pos_tally.entry(field).or_insert(0) += 1;
            }
            token_pos_matches.extend(packed_fields.into_owned());
        }
        shadow_lane.push(token_pos_matches);
    }

    // 2. Pass Two: Run Adjacency Statistical Accumulation Machine
    let mut active_left_variants: Option<&Vec<PosslqPropertyField>> = None;
    let mut active_left_str: Option<&[char]> = None;

    for (i, tok) in toks.iter().enumerate() {
        match &tok.kind {
            TokenKind::Word(_) => {
                let current_variants = &shadow_lane[i];
                let current_str = tok.get_ch(src);

                if let (Some(left_variants), Some(left_str)) =
                    (active_left_variants, active_left_str)
                {
                    for &v1 in left_variants {
                        for &v2 in current_variants {
                            let pair = SlqPair {
                                left: v1,
                                right: v2,
                            };

                            let data = pair_pos_tally.entry(pair).or_default();
                            data.total_count += 1;

                            let word_key = (left_str, current_str);
                            *data.word_pairs.entry(word_key).or_insert(0) += 1;

                            pair_word_tally.entry(word_key).or_default().insert(pair);
                        }
                    }
                }

                active_left_variants = Some(current_variants);
                active_left_str = Some(current_str);
            }
            TokenKind::Space(_) => continue,
            _ => {
                active_left_variants = None;
                active_left_str = None;
            }
        }
    }

    // --- FREQUENCY DISTRIBUTION SUMMARY REPORTS ---
    println!("\n=== INDIVIDUAL POS FREQUENCY METRICS ===");
    let mut single_vec: Vec<(PosslqPropertyField, usize)> = single_pos_tally.into_iter().collect();
    single_vec.sort_by_key(|&(_, count)| Reverse(count));
    single_vec.retain(|&(_, count)| count > 1);

    for (field, count) in &single_vec {
        println!(
            "Seen Count: {:<4} | Bitfield State Shape: {:?}",
            count, field
        );
    }

    println!("\n=== POS SLQ FREQUENCY METRICS ===");
    let mut pair_vec: Vec<(SlqPair, SlqTallyData)> = pair_pos_tally.into_iter().collect();

    // Sort by total_count descending
    pair_vec.sort_by_key(|(_, data)| Reverse(data.total_count));

    for (slq_pair, data) in pair_vec {
        println!(
            "{}[{}] + {}[{}]: {}",
            slq_pair.left.variant_name(),
            slq_pair.left.trit_string(),
            slq_pair.right.variant_name(),
            slq_pair.right.trit_string(),
            data.total_count
        );

        type WordPair<'a> = (&'a [char], &'a [char]);
        type CountedPair<'a> = (WordPair<'a>, usize);

        let mut examples: Vec<CountedPair> = data.word_pairs.into_iter().collect();

        examples.sort_by_key(|&(_, count)| Reverse(count));

        let strict_one_pos_examples: Vec<_> = examples
            .into_iter()
            .filter(|((w1, w2), _)| {
                let structural_variations_count =
                    pair_word_tally.get(&(w1, w2)).map_or(0, |set| set.len());

                structural_variations_count == 1
            })
            .take(4)
            .collect();

        if !strict_one_pos_examples.is_empty() {
            println!("  Top Examples (Strictly 1 POS structure variant):");
            for ((w1, w2), count) in strict_one_pos_examples {
                println!(
                    "    - {}x “{}” + “{}”",
                    count,
                    w1.iter().collect::<String>(),
                    w2.iter().collect::<String>()
                );
            }
        } else {
            println!("  Top Examples: [None found with exactly 1 structural variation]");
        }
    }

    Ok(())
}
