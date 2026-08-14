use std::sync::{Arc, LazyLock};

use super::MutableDictionary;
use crate::DictWordMetadata;

pub(crate) fn turkish_fold_char(c: char) -> char {
    match c {
        'İ' => 'i',
        'I' => 'ı',
        other => other.to_lowercase().next().unwrap_or(other),
    }
}

pub fn turkish_fold_chars(word: &[char]) -> Vec<char> {
    word.iter().copied().map(turkish_fold_char).collect()
}

/// Word list harvested from the GhostEdit Zemberek extra dictionary
/// (`turkish/data/wordlist-tr.txt`). Folded with Turkish i/ı rules.
pub fn turkish_dictionary() -> Arc<MutableDictionary> {
    static DICT: LazyLock<Arc<MutableDictionary>> = LazyLock::new(|| {
        Arc::new(load_turkish_wordlist(include_str!(
            "../../../turkish/data/wordlist-tr.txt"
        )))
    });
    DICT.clone()
}

pub fn load_turkish_wordlist(text: &str) -> MutableDictionary {
    let mut dict = MutableDictionary::new();
    let meta = DictWordMetadata::default();
    dict.extend_words(text.lines().flat_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        let original: Vec<char> = trimmed.chars().collect();
        let folded = turkish_fold_chars(&original);
        if folded == original {
            vec![(original, meta.clone())]
        } else {
            vec![(original, meta.clone()), (folded, meta.clone())]
        }
    }));
    dict
}

#[cfg(test)]
mod tests {
    use super::{load_turkish_wordlist, turkish_dictionary};
    use crate::spell::Dictionary;

    #[test]
    fn bundled_list_contains_ve() {
        let dict = turkish_dictionary();
        let ve: Vec<char> = "ve".chars().collect();
        assert!(dict.contains_word(&ve));
    }

    #[test]
    fn bundled_list_contains_kelime() {
        let dict = turkish_dictionary();
        let w: Vec<char> = "kelime".chars().collect();
        assert!(dict.contains_word(&w));
    }

    #[test]
    fn loader_folds_dotted_i() {
        let dict = load_turkish_wordlist("İstanbul\n");
        let w: Vec<char> = "istanbul".chars().collect();
        assert!(dict.contains_word(&w));
    }
}
