use clap::Parser;
use harper_core::spell::{MutableDictionary, Dictionary};
use std::fs;
use std::path::PathBuf;

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
    
    if let Some(text) = args.text {
        spell_check_text(&dict, &text);
    } else if !args.test {
        println!("\n💡 Usage examples:");
        println!("   --test          Run basic dictionary tests");
        println!("   --text \"text\"  Spell check text in the specified language");
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