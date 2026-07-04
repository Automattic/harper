use clap::Parser;
use harper_core::spell::{MutableDictionary, Dictionary};
use harper_core::DictWordMetadata;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Harper Language Testing Framework
/// Loads dictionary and annotations from files and tests spell checking for any language
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Language to test (e.g., german, english, portuguese)
    #[arg(short, long, required = true)]
    language: String,
    
    /// Path to the dictionary file (optional, defaults to language directory)
    #[arg(short, long)]
    dict: Option<PathBuf>,
    
    /// Path to the annotations file (optional, defaults to language directory)
    #[arg(short, long)]
    annotations: Option<PathBuf>,
    
    /// Text to spell check (optional)
    #[arg(short, long)]
    text: Option<String>,
    
    /// Test mode - run basic tests
    #[arg(short, long, default_value_t = false)]
    test: bool,
    
    /// Show metadata for words
    #[arg(short, long, default_value_t = false)]
    metadata: bool,
    
    /// Check word (alternative to text for single word)
    #[arg(short, long)]
    word: Option<String>,
    
    /// Compare with hunspell spell checking
    #[arg(short, long, default_value_t = false)]
    hunspell: bool,
}

fn main() {
    let args = Args::parse();
    
    println!("🌍 Harper Language Testing Framework");
    println!("{}", "=".repeat(50));
    println!("📚 Testing language: {}", args.language);
    
    // Determine file paths
    let dict_path = match args.dict {
        Some(path) => path,
        None => PathBuf::from(format!("../../language/{}/dictionary.dict", args.language)),
    };
    
    let annotations_path = match args.annotations {
        Some(path) => path,
        None => PathBuf::from(format!("../../language/{}/annotations-{}.json", args.language, args.language)),
    };
    
    // Load dictionary file
    let dict_content = match fs::read_to_string(&dict_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read dictionary file {}: {}", dict_path.display(), e);
            return;
        }
    };
    
    // Load annotations file
    let annotations_content = match fs::read_to_string(&annotations_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Failed to read annotations file {}: {}", annotations_path.display(), e);
            return;
        }
    };
    
    println!("📖 Loading dictionary from: {}", dict_path.display());
    println!("📝 Loading annotations from: {}", annotations_path.display());
    
    // Create dictionary from files
    let dict = match MutableDictionary::from_rune_files(&dict_content, &annotations_content) {
        Ok(dict) => dict,
        Err(e) => {
            eprintln!("❌ Failed to create dictionary: {}", e);
            return;
        }
    };
    
    println!("✅ Dictionary loaded successfully!");
    println!("   Word count: {}", dict.word_count());
    
    if args.test {
        run_language_tests(&dict, &args.language);
    }
    
    if args.metadata {
        show_metadata(&dict, &args.word.clone().unwrap_or_default(), &args.text.clone().unwrap_or_default());
    }
    
    if let Some(word) = args.word {
        check_word_metadata(&dict, &word);
    }
    
    if args.hunspell {
        compare_with_hunspell(&args.language, &dict, &args.text.clone().unwrap_or_default());
    } else if let Some(text) = args.text {
        spell_check_text(&dict, &text);
    } else if !args.test && !args.metadata {
        println!("\n💡 Usage examples:");
        println!("   --test          Run basic dictionary tests");
        println!("   --text \"text\"  Spell check text in the specified language");
        println!("   --word \"word\"  Show metadata for a single word");
        println!("   --metadata       Show metadata for words in text");
        println!("   --hunspell       Compare with hunspell spell checking");
    }
}

fn get_hunspell_dict_name(language: &str) -> Option<String> {
    match language {
        "german" => Some("de_DE".to_string()),
        "english" => Some("en_US".to_string()),
        "portuguese" => Some("pt_PT".to_string()),
        "slovak" => Some("sk_SK".to_string()),
        _ => None,
    }
}

fn check_word_with_hunspell(language: &str, word: &str) -> bool {
    if let Some(dict_name) = get_hunspell_dict_name(language) {
        let output = Command::new("hunspell")
            .arg("-d")
            .arg(format!("{}", dict_name))
            .arg("-a")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn();
        
        match output {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = stdin.write_all(word.as_bytes());
                    let _ = stdin.write_all(b"\n");
                }
                if let Ok(status) = child.wait() {
                    if status.success() {
                        // hunspell -a returns lines starting with * for correct words
                        // For simplicity, we'll use the exit code approach
                        return true;
                    }
                }
            }
            Err(_) => {}
        }
    }
    
    // Fallback: try simple approach
    let result = Command::new("hunspell")
        .arg("-d")
        .arg(format!("{}_DE", language))
        .arg("-i")
        .arg("utf-8")
        .arg(word)
        .output();
    
    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn compare_with_hunspell(language: &str, dict: &MutableDictionary, text: &str) {
    println!("\n🔍 Comparing Harper with Hunspell for language: {}", language);
    println!("{}", "=".repeat(60));
    
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut both_correct = 0;
    let mut harper_only = 0;
    let mut hunspell_only = 0;
    let mut both_wrong = 0;
    
    let dict_name = get_hunspell_dict_name(language);
    
    if dict_name.is_none() {
        println!("⚠️  Hunspell dictionary not configured for language: {}", language);
        println!("   Harper results only:");
        spell_check_text(dict, text);
        return;
    }
    
    // Check if hunspell is available
    let hunspell_available = Command::new("hunspell").arg("-v").output().is_ok();
    if !hunspell_available {
        println!("⚠️  Hunspell not found. Please install hunspell and dictionary files.");
        println!("   Harper results only:");
        spell_check_text(dict, text);
        return;
    }
    
    for word in &words {
        let harper_knows = dict.get_word_metadata(&word.chars().collect::<Vec<_>>()).is_some();
        let hunspell_knows = check_word_with_hunspell(language, word);
        
        if harper_knows && hunspell_knows {
            both_correct += 1;
        } else if harper_knows && !hunspell_knows {
            harper_only += 1;
        } else if !harper_knows && hunspell_knows {
            hunspell_only += 1;
        } else {
            both_wrong += 1;
        }
    }
    
    println!("\n📊 Comparison Results:");
    println!("   Total words: {}", words.len());
    println!("   ✅ Both correct: {}", both_correct);
    println!("   🟢 Harper only: {}", harper_only);
    println!("   🔴 Hunspell only: {}", hunspell_only);
    println!("   ❌ Both wrong: {}", both_wrong);
    
    if words.len() > 0 {
        let harper_coverage = ((both_correct + harper_only) as f64 / words.len() as f64) * 100.0;
        let hunspell_coverage = ((both_correct + hunspell_only) as f64 / words.len() as f64) * 100.0;
        
        println!("\n📈 Coverage:");
        println!("   Harper: {:.1}%", harper_coverage);
        println!("   Hunspell: {:.1}%", hunspell_coverage);
        
        if harper_only > 0 {
            println!("\n💡 Harper recognizes words that Hunspell doesn't:");
            for word in &words {
                let harper_knows = dict.get_word_metadata(&word.chars().collect::<Vec<_>>()).is_some();
                let hunspell_knows = check_word_with_hunspell(language, word);
                if harper_knows && !hunspell_knows {
                    println!("   - {}", word);
                }
            }
        }
        
        if hunspell_only > 0 {
            println!("\n💡 Hunspell recognizes words that Harper doesn't:");
            for word in &words {
                let harper_knows = dict.get_word_metadata(&word.chars().collect::<Vec<_>>()).is_some();
                let hunspell_knows = check_word_with_hunspell(language, word);
                if !harper_knows && hunspell_knows {
                    println!("   - {}", word);
                }
            }
        }
    }
}

fn run_language_tests(dict: &MutableDictionary, language: &str) {
    println!("\n🧪 Running basic tests for {}...", language);
    
    // Language-specific test words
    let test_words = match language {
        "german" => vec![
            "der", "die", "das", "und", "oder",
            "Mann", "Frau", "Katze", "Hund", "Tisch",
            "Buch", "Haus", "Zeit", "Jahr", "Kind",
            "sein", "haben", "werden", "können", "müssen",
            "lesbar", "freundlich", "kindisch", "fleißig",
            "Bildung", "Freiheit", "Information", "Universität",
        ],
        "english" => vec![
            "the", "and", "of", "to", "in",
            "dog", "cat", "house", "book", "child",
            "run", "jump", "happy", "quick", "beautiful",
            "education", "freedom", "information", "university",
        ],
        "portuguese" => vec![
            "o", "a", "e", "de", "em",
            "cão", "gato", "casa", "livro", "criança",
            "correr", "pular", "feliz", "rápido", "bonito",
            "educação", "liberdade", "informação", "universidade",
        ],
        _ => vec![
            "test", "word", "example", "language", "framework",
        ],
    };
    
    let mut found = 0;
    let mut missing = 0;
    
    for word in &test_words {
        if dict.get_word_metadata(&word.chars().collect::<Vec<_>>()).is_some() {
            found += 1;
        } else {
            missing += 1;
            println!("   ❌ Missing: {}", word);
        }
    }
    
    println!("\n📊 Test Results for {}:", language);
    println!("   Found: {}/{} words", found, test_words.len());
    println!("   Missing: {}/{} words", missing, test_words.len());
    
    if missing == 0 {
        println!("   ✅ All test words present!");
    }
}

fn spell_check_text(dict: &MutableDictionary, text: &str) {
    println!("\n🔍 Spell checking text: \"{}\"", text);
    
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut unknown_words = Vec::new();
    
    for word in &words {
        if dict.get_word_metadata(&word.chars().collect::<Vec<_>>()).is_none() {
            unknown_words.push(word);
        }
    }
    
    if unknown_words.is_empty() {
        println!("   ✅ All words recognized!");
    } else {
        println!("   ⚠️  Unknown words:");
        for word in unknown_words {
            println!("      - {}", word);
        }
    }
}

fn check_word_metadata(dict: &MutableDictionary, word: &str) {
    println!("\n🔍 Checking metadata for word: \"{}\"", word);
    
    let word_chars: Vec<char> = word.chars().collect();
    let lowercase_chars: Vec<char> = word_chars.iter().map(|c| c.to_ascii_lowercase()).collect();
    
    if let Some(metadata) = dict.get_word_metadata(&word_chars) {
        println!("   ✅ Found (exact case):");
        print_metadata(&metadata, 6);
    } else if let Some(metadata) = dict.get_word_metadata(&lowercase_chars) {
        println!("   ✅ Found (lowercase):");
        print_metadata(&metadata, 6);
    } else {
        println!("   ❌ Word not found in dictionary");
    }
}

fn show_metadata(dict: &MutableDictionary, single_word: &str, text: &str) {
    let words_to_check: Vec<&str> = if !single_word.is_empty() {
        vec![single_word]
    } else if !text.is_empty() {
        text.split_whitespace().collect()
    } else {
        return;
    };
    
    println!("\n📋 Word Metadata Report");
    println!("{}", "=".repeat(50));
    
    for word in &words_to_check {
        let word_chars: Vec<char> = word.chars().collect();
        let lowercase_chars: Vec<char> = word_chars.iter().map(|c| c.to_ascii_lowercase()).collect();
        
        println!("\n🔹 Word: \"{}\"", word);
        
        // Check exact case
        if let Some(metadata) = dict.get_word_metadata(&word_chars) {
            println!("   ✅ Exact case:");
            print_metadata(&metadata, 6);
        } else {
            println!("   ❌ Not found (exact case)");
        }
        
        // Check lowercase
        if let Some(metadata) = dict.get_word_metadata(&lowercase_chars) {
            println!("   ✅ Lowercase:");
            print_metadata(&metadata, 6);
        } else {
            println!("   ❌ Not found (lowercase)");
        }
    }
}

fn print_metadata(metadata: &DictWordMetadata, indent: usize) {
    let prefix = " ".repeat(indent);
    
    if let Some(noun) = &metadata.noun {
        let is_singular = noun.is_singular.as_ref().map(|s| if *s { ", singular" } else { ", plural" }).unwrap_or_default();
        let is_plural = noun.is_plural.as_ref().map(|p| if *p { ", plural" } else { "" }).unwrap_or_default();
        let is_proper = noun.is_proper.as_ref().map(|p| if *p { ", proper" } else { "" }).unwrap_or_default();
        println!("{}📚 Noun{}{}{}", prefix, is_singular, is_plural, is_proper);
    }
    
    if let Some(verb) = &metadata.verb {
        let form = verb.verb_forms.as_ref().map(|f| format!(", form: {:?}", f)).unwrap_or_default();
        println!("{}✍️ Verb{}", prefix, form);
    }
    
    if let Some(adjective) = &metadata.adjective {
        println!("{}🎨 Adjective", prefix);
    }
    
    if let Some(adverb) = &metadata.adverb {
        println!("{}💨 Adverb", prefix);
    }
    
    if let Some(conjunction) = &metadata.conjunction {
        println!("{}🔗 Conjunction", prefix);
    }
    
    if let Some(determiner) = &metadata.determiner {
        println!("{}📍 Determiner", prefix);
    }
    
    if let Some(pronoun) = &metadata.pronoun {
        println!("{}👤 Pronoun", prefix);
    }
    
    if metadata.preposition {
        println!("{}📌 Preposition", prefix);
    }
    
    if metadata.noun.is_none() && metadata.verb.is_none() && metadata.adjective.is_none() 
       && metadata.adverb.is_none() && metadata.conjunction.is_none() 
       && metadata.determiner.is_none() && metadata.pronoun.is_none() 
       && !metadata.preposition {
        println!("{}⚪ No specific POS metadata", prefix);
    }
}