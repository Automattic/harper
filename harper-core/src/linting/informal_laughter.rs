pub fn is_informal_laughter(chars: &[char]) -> bool {
    if chars.len() < 2 {
        return false;
    }

    chars.iter().enumerate().all(|(idx, c)| {
        if idx % 2 == 0 {
            matches!(c, 'h' | 'H')
        } else {
            matches!(c, 'a' | 'A')
        }
    })
}

#[cfg(test)]
mod tests {
    use super::is_informal_laughter;

    fn matches(word: &str) -> bool {
        is_informal_laughter(&word.chars().collect::<Vec<_>>())
    }

    #[test]
    fn matches_alternating_laughter() {
        for word in ["ha", "hah", "haha", "hahah", "hahaha", "hahahah"] {
            assert!(matches(word), "{word} should match");
        }
    }

    #[test]
    fn matches_laughter_case_insensitively() {
        for word in ["Hah", "Hahahah", "HAHAHA", "HaHaH"] {
            assert!(matches(word), "{word} should match");
        }
    }

    #[test]
    fn does_not_match_malformed_hah_chains() {
        for word in ["hahhah", "haah", "hahh", "hha"] {
            assert!(!matches(word), "{word} should not match");
        }
    }

    #[test]
    fn does_not_match_out_of_scope_interjections() {
        for word in ["ah", "Ah", "ahah", "hehe"] {
            assert!(!matches(word), "{word} should be ignored by this matcher");
        }
    }
}
