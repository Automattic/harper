//! Metaphone phonetic algorithm for English spelling suggestions.
//! Based on the original Metaphone algorithm by Lawrence Philips.

/// Returns the Metaphone code for a word (maximum 6 characters).
/// Returns an array of 6 chars (padded with spaces for non-used positions).
/// Returns an array of spaces if the word has no English consonants.
pub fn metaphone(word: &[char]) -> [char; 6] {
    if word.is_empty() {
        return [' '; 6];
    }

    let word: Vec<char> = word.iter().map(|c| c.to_ascii_uppercase()).collect();
    let mut result = [' '; 6];
    let mut pos = 0usize;
    let mut result_pos = 0usize;

    // Skip initial letters that have special handling
    if word.len() >= 2 {
        match (word[0], word[1]) {
            ('K', 'N') => {
                result[result_pos] = 'N';
                result_pos += 1;
                pos += 2;
            }
            ('A', 'E') if word.len() == 2 => {
                result[0] = 'A';
                result[1] = ' ';
                return result;
            }
            ('P', 'N') | ('P', 'S') | ('P', 'H') | ('A', 'E') | ('A', 'I') | ('A', 'O') | ('G', 'N') | ('G', 'H') => {
                if word[0] == 'A' || word[0] == 'G' {
                    pos += 1;
                }
            }
            _ => {}
        }
    }

    // Main encoding loop
    while pos < word.len() && result_pos < 6 {
        let c = word[pos];

        match c {
            'A' | 'E' | 'I' | 'O' | 'U' => {
                if pos == 0 {
                    result[result_pos] = c;
                    result_pos += 1;
                }
                pos += 1;
            }
            'B' => {
                // Drop if at end and preceded by M
                if pos + 1 >= word.len() && pos > 0 && word[pos - 1] == 'M' {
                    pos += 1;
                    continue;
                }
                result[result_pos] = 'P';
                result_pos += 1;
                pos += 1;
            }
            'C' => {
                // Skip if preceded by S
                if pos > 0 && word[pos - 1] == 'S' {
                    pos += 1;
                    continue;
                }
                if pos + 1 < word.len() {
                    match word[pos + 1] {
                        'H' => {
                            result[result_pos] = if pos + 2 < word.len() && word[pos + 2] == 'R' {
                                'K'
                            } else {
                                'X'
                            };
                            result_pos += 1;
                            pos += 2;
                        }
                        'I' | 'E' | 'Y' => {
                            result[result_pos] = 'S';
                            result_pos += 1;
                            pos += 2;
                        }
                        _ => {
                            result[result_pos] = 'K';
                            result_pos += 1;
                            pos += 1;
                        }
                    }
                } else {
                    result[result_pos] = 'K';
                    result_pos += 1;
                    pos += 1;
                }
            }
            'D' => {
                if pos + 1 < word.len() && matches!(word[pos + 1], 'G' | 'J') {
                    result[result_pos] = 'J';
                    result_pos += 1;
                    pos += 2;
                } else {
                    result[result_pos] = 'T';
                    result_pos += 1;
                    pos += 1;
                }
            }
            'F' | 'J' | 'L' | 'M' | 'N' | 'R' => {
                result[result_pos] = c;
                result_pos += 1;
                pos += 1;
            }
            'G' => {
                // Drop if at end
                if pos + 1 >= word.len() {
                    pos += 1;
                    continue;
                }
                match word[pos + 1] {
                    'H' => {
                        // Drop H if followed by vowel or at end
                        if pos + 2 < word.len() && is_vowel(word[pos + 2]) {
                            result[result_pos] = 'F';
                            result_pos += 1;
                        } else if pos + 2 >= word.len() {
                            // at end, skip
                        } else {
                            result[result_pos] = 'F';
                            result_pos += 1;
                        }
                        pos += 2;
                    }
                    'N' => {
                        // Drop at end or before vowel
                        if pos + 2 >= word.len() || (pos + 2 < word.len() && is_vowel(word[pos + 2])) {
                            pos += 2;
                            continue;
                        }
                        result[result_pos] = 'K';
                        result_pos += 1;
                        pos += 1;
                    }
                    'I' | 'E' | 'Y' => {
                        result[result_pos] = 'J';
                        result_pos += 1;
                        pos += 2;
                    }
                    _ => {
                        result[result_pos] = 'K';
                        result_pos += 1;
                        pos += 1;
                    }
                }
            }
            'H' => {
                if pos == 0 || (pos > 0 && !is_vowel(word[pos - 1])) {
                    if pos + 1 < word.len() && is_vowel(word[pos + 1]) {
                        result[result_pos] = 'H';
                        result_pos += 1;
                    }
                }
                pos += 1;
            }
            'K' => {
                if pos == 0 || (pos > 0 && word[pos - 1] != 'C') {
                    result[result_pos] = 'K';
                    result_pos += 1;
                }
                pos += 1;
            }
            'P' => {
                if pos + 1 < word.len() && word[pos + 1] == 'H' {
                    result[result_pos] = 'F';
                    result_pos += 1;
                    pos += 2;
                } else {
                    result[result_pos] = 'P';
                    result_pos += 1;
                    pos += 1;
                }
            }
            'Q' => {
                result[result_pos] = 'K';
                result_pos += 1;
                pos += 1;
            }
            'S' => {
                if pos + 1 < word.len() && word[pos + 1] == 'H' {
                    result[result_pos] = 'X';
                    result_pos += 1;
                    pos += 2;
                } else if pos + 2 < word.len() && word[pos + 1] == 'I' && matches!(word[pos + 2], 'O' | 'A' | 'Y') {
                    result[result_pos] = 'X';
                    result_pos += 1;
                    pos += 3;
                } else {
                    result[result_pos] = 'S';
                    result_pos += 1;
                    pos += 1;
                }
            }
            'T' => {
                if pos + 1 < word.len() && word[pos + 1] == 'H' {
                    result[result_pos] = '0'; // Theta sound
                    result_pos += 1;
                    pos += 2;
                } else if pos + 2 < word.len() && word[pos + 1] == 'I' && matches!(word[pos + 2], 'O' | 'A' | 'Y') {
                    result[result_pos] = 'X';
                    result_pos += 1;
                    pos += 3;
                } else if pos + 1 < word.len() && word[pos + 1] == 'C' {
                    result[result_pos] = 'X';
                    result_pos += 1;
                    pos += 2;
                } else {
                    result[result_pos] = 'T';
                    result_pos += 1;
                    pos += 1;
                }
            }
            'V' => {
                result[result_pos] = 'F';
                result_pos += 1;
                pos += 1;
            }
            'W' | 'Y' => {
                if pos + 1 < word.len() && is_vowel(word[pos + 1]) {
                    result[result_pos] = c;
                    result_pos += 1;
                }
                pos += 1;
            }
            'X' => {
                result[result_pos] = 'K';
                result_pos += 1;
                if result_pos < 6 {
                    result[result_pos] = 'S';
                    result_pos += 1;
                }
                pos += 1;
            }
            'Z' => {
                result[result_pos] = 'S';
                result_pos += 1;
                pos += 1;
            }
            _ => pos += 1,
        }
    }

    result
}

/// Returns true if the character is an English vowel.
fn is_vowel(c: char) -> bool {
    matches!(c, 'A' | 'E' | 'I' | 'O' | 'U')
}

/// Calculates phonetic similarity score between two words.
/// Returns a penalty score: 0 = exact match, higher = more different.
pub fn phonetic_similarity(word1: &[char], word2: &[char]) -> i32 {
    let code1 = metaphone(word1);
    let code2 = metaphone(word2);

    let matching = code1.iter().zip(code2.iter()).filter(|(a, b)| **a != ' ' && a == b).count();
    let total = code1.iter().filter(|&&c| c != ' ').count().max(1);
    let code2_nonzero = code2.iter().filter(|&&c| c != ' ').count().max(1);

    // Prefix bonus for common starting sounds
    let prefix_len = word1.iter().zip(word2.iter()).take_while(|(a, b)| a.eq_ignore_ascii_case(b)).count();

    let similarity = (matching as f64 / total as f64 + matching as f64 / code2_nonzero as f64) / 2.0;
    let prefix_bonus = (prefix_len as f64 * 0.1).min(0.2);

    let score = 1.0 - similarity - prefix_bonus;
    (score * 20.0).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metaphone_basic() {
        // Test some known Metaphone values
        assert_eq!(metaphone(&['C', 'A', 'T']), ['K', 'T', ' ', ' ', ' ', ' ']);
        assert_eq!(metaphone(&['D', 'O', 'G']), ['T', 'K', ' ', ' ', ' ', ' ']);
    }

    #[test]
    fn test_phonetic_similarity_same() {
        // Same words should have very low score
        let score = phonetic_similarity(&['C', 'A', 'T'], &['C', 'A', 'T']);
        assert!(score <= 5, "Same words should score very low, got {}", score);
    }

    #[test]
    fn test_phonetic_similarity_phonetic() {
        // "cat" and "kat" sound the same
        let score = phonetic_similarity(&['C', 'A', 'T'], &['K', 'A', 'T']);
        assert!(score < 15, "Phonetically similar words should score low, got {}", score);
    }
}