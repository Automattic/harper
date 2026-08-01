use std::fs;
use std::io::{self, BufRead};
use std::time::Instant;

use harper_core::spell::{MutableDictionary, Dictionary};
use harper_core::spell::rune::AttributeList;


/// Load expanded dictionary from gzip file
fn load_expanded_dictionary(path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::BufReader;
    use flate2::read::GzDecoder;

    let file = File::open(path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let reader = io::BufReader::new(decoder);

    let words: Vec<String> = reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?;
            let trimmed = line.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .collect();

    Ok(words)
}

/// Filter words for testing (remove proper nouns, abbreviations, etc.)
fn filter_test_words(words: &[String]) -> Vec<String> {
    words.iter()
        .filter(|word| {
            // Skip words that start with hyphen or uppercase
            if word.starts_with('-') || (word.chars().next().map_or(false, |c| c.is_uppercase())) {
                return false;
            }
            // Skip very long words
            if word.len() > 30 {
                return false;
            }
            // Skip very short words
            if word.len() < 3 {
                return false;
            }
            // Skip words with special characters
            let special_chars = ['/', '\\', '*', '?', '[', ']', '{', '}', '(', ')'];
            if word.chars().any(|c| special_chars.contains(&c)) {
                return false;
            }
            true
        })
        .cloned()
        .collect()
}

/// Check words with Harper dictionary (in-memory, no subprocess)
fn check_words_with_harper(
    dict: &MutableDictionary,
    words: &[String],
) -> (usize, Vec<String>) {
    let mut recognized = 0;
    let mut unknown_words = Vec::new();

    for word in words {
        let word_chars: Vec<char> = word.chars().collect();
        if dict.get_word_metadata(&word_chars).is_some() {
            recognized += 1;
        } else {
            unknown_words.push(word.clone());
        }
    }

    (recognized, unknown_words)
}

/// Capitalize first letter of string
pub trait Capitalize {
    fn capitalize(&self) -> String;
}

impl Capitalize for str {
    fn capitalize(&self) -> String {
        let mut chars = self.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().chain(chars).collect(),
        }
    }
}

/// Run coverage analysis with a pre-loaded dictionary (more efficient)
pub fn run_coverage_analysis_with_dict(
    language: &str,
    dict: &MutableDictionary,
    expanded_dict_path: &str,
    sample_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    println!("🔍 {} Coverage Analysis", language.capitalize());
    println!("{}", "=".repeat(50));

    let harper_word_count = dict.word_count();
    println!("📖 Using pre-loaded Harper dictionary...");
    println!("   ✅ Harper dictionary loaded: {} base words", harper_word_count);

    // Load expanded dictionary
    println!("📖 Loading expanded dictionary...");
    let expanded_words = load_expanded_dictionary(expanded_dict_path)?;
    println!("   Loaded {} words from expanded dictionary", expanded_words.len());

    // Filter words for testing
    let test_words = filter_test_words(&expanded_words);
    println!("   Filtered to {} words for testing", test_words.len());

    // Apply sample size
    use rand::seq::SliceRandom;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut shuffled = test_words;
    shuffled.shuffle(&mut rng);
    shuffled.truncate(sample_size.min(shuffled.len()));
    let test_words = shuffled;

    println!("   Using {} words for coverage testing", test_words.len());

    // Test words with Harper (in-memory, no subprocess overhead)
    println!("🧪 Testing words with Harper...");
    let (recognized, unknown_words) = check_words_with_harper(dict, &test_words);

    let coverage_percentage = if !test_words.is_empty() {
        (recognized as f64 / test_words.len() as f64) * 100.0
    } else {
        0.0
    };

    println!("\n📊 Coverage Results");
    println!("   Words Tested: {}", test_words.len());
    println!("   Words Recognized: {}", recognized);
    println!("   Coverage: {:.1}%", coverage_percentage);

    // Output sample of unrecognized words
    if !unknown_words.is_empty() {
        println!("\n📋 Sample of Unrecognized Words ({} total):", unknown_words.len());
        let sample_size_output = std::cmp::min(20, unknown_words.len());
        for (i, word) in unknown_words.into_iter().take(sample_size_output).enumerate() {
            println!("   {:2}. {}", i + 1, word);
        }
    }

    // Dictionary statistics
    println!("\n📚 Dictionary Statistics");
    println!("   Harper Dictionary Size: {} base words", harper_word_count);
    println!("   Expanded Dictionary Size: {} words", expanded_words.len());

    if harper_word_count > 0 && !expanded_words.is_empty() {
        let size_ratio = (harper_word_count as f64 / expanded_words.len() as f64) * 100.0;
        println!("   Size Ratio: {:.2}%", size_ratio);
    }

    // Efficiency metrics
    if harper_word_count > 0 {
        let efficiency = if recognized > 0 {
            recognized as f64 / harper_word_count as f64
        } else {
            0.0
        };
        println!("\n🎯 Efficiency Metrics");
        println!("   Base words: {}", harper_word_count);
        println!("   Words recognized: {}", recognized);
        println!("   Efficiency ratio: {:.2} words per base word", efficiency);

        println!("\n   For reference:");
        println!("   - English typically has ~1.5-2.0 words per base word");
        println!("   - German should aim for >2.5 due to compounding");
    }

    // Annotation statistics
    println!("\n🏷️  Annotation Statistics");
    println!("   Note: Using pre-loaded dictionary, annotation stats not available");

    // Recommendations
    println!("\n💡 Recommendations");
    if coverage_percentage < 30.0 {
        println!("   ⚠️  Low coverage ({:.1}%) - consider adding more root words", coverage_percentage);
    } else if coverage_percentage < 60.0 {
        println!("   🟡 Moderate coverage ({:.1}%) - focus on common word patterns and affix rules", coverage_percentage);
    } else {
        println!("   ✅ Good coverage ({:.1}%) - focus on edge cases and compound words", coverage_percentage);
    }

    if harper_word_count > 0 && !expanded_words.is_empty() {
        let target_coverage = 80.0;
        let efficiency = if recognized > 0 {
            recognized as f64 / harper_word_count as f64
        } else {
            0.0
        };
        if coverage_percentage < target_coverage && efficiency > 0.0 {
            let words_needed_approx = (expanded_words.len() as f64 * target_coverage / 100.0 - recognized as f64) / efficiency;
            println!("   🎯 To reach {}% coverage: approximately {} more base words or improved rules",
                     target_coverage, words_needed_approx as usize);
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("📈 Summary: {:.1}% coverage with {} base words", coverage_percentage, harper_word_count);
    if harper_word_count > 0 {
        let efficiency = if recognized > 0 {
            recognized as f64 / harper_word_count as f64
        } else {
            0.0
        };
        println!("   Efficiency: {:.2} words per base word", efficiency);
    }
    println!("   Time elapsed: {:.2?}", start_time.elapsed());
    println!("{}", "=".repeat(50));

    Ok(())
}
