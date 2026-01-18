use std::borrow::Cow;

use itertools::Itertools;

use crate::char_ext::CharExt;
use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Dialect, Document, TokenStringExt};

#[derive(PartialEq)]
pub enum InitialSound {
    Vowel,
    Consonant,
    Either, // for SQL
}

#[derive(Debug)]
pub struct AnA {
    dialect: Dialect,
}

impl AnA {
    pub fn new(dialect: Dialect) -> Self {
        Self { dialect }
    }
}

impl Linter for AnA {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for chunk in document.iter_chunks() {
            for (first_idx, second_idx) in chunk.iter_word_indices().tuple_windows() {
                // [`TokenKind::Unlintable`] might have semantic meaning.
                if chunk[first_idx..second_idx].iter_unlintables().count() > 0
                    || chunk[first_idx + 1..second_idx]
                        .iter_word_like_indices()
                        .count()
                        > 0
                {
                    continue;
                }

                let first = &chunk[first_idx];
                let second = &chunk[second_idx];

                let chars_first = document.get_span_content(&first.span);
                let chars_second = document.get_span_content(&second.span);
                // Break the second word on hyphens for this lint.
                // Example: "An ML-based" is an acceptable noun phrase.
                let chars_second = chars_second
                    .split(|c| !c.is_alphanumeric())
                    .next()
                    .unwrap_or(chars_second);

                let is_a_an = match chars_first {
                    ['a'] => Some(true),
                    ['A'] => Some(true),
                    ['a', 'n'] => Some(false),
                    ['A', 'n'] => Some(false),
                    _ => None,
                };

                let Some(a_an) = is_a_an else {
                    continue;
                };

                let should_be_a_an = match starts_with_vowel(chars_second, self.dialect) {
                    InitialSound::Vowel => false,
                    InitialSound::Consonant => true,
                    InitialSound::Either => return lints,
                };

                if a_an != should_be_a_an {
                    let replacement = match a_an {
                        true => vec!['a', 'n'],
                        false => vec!['a'],
                    };

                    lints.push(Lint {
                        span: first.span,
                        lint_kind: LintKind::Miscellaneous,
                        suggestions: vec![Suggestion::replace_with_match_case(
                            replacement,
                            chars_first,
                        )],
                        message: "Incorrect indefinite article.".to_string(),
                        priority: 31,
                    })
                }
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "A rule that looks for incorrect indefinite articles. For example, `this is an mule` would be flagged as incorrect."
    }
}

fn to_lower_word(word: &[char]) -> Cow<'_, [char]> {
    if word.iter().any(|c| c.is_uppercase()) {
        Cow::Owned(
            word.iter()
                .flat_map(|c| c.to_lowercase())
                .collect::<Vec<_>>(),
        )
    } else {
        Cow::Borrowed(word)
    }
}

/// Checks whether a provided word begins with a vowel _sound_.
///
/// It was produced through trial and error.
/// Matches with 99.71% and 99.77% of vowels and non-vowels in the
/// Carnegie-Mellon University word -> pronunciation dataset.
fn starts_with_vowel(word: &[char], dialect: Dialect) -> InitialSound {
    let is_likely_initialism = word.iter().all(|c| !c.is_alphabetic() || c.is_uppercase());

    if is_likely_initialism && !word.is_empty() && !is_likely_acronym(word) {
        if matches!(word, ['S', 'Q', 'L']) {
            return InitialSound::Either;
        }
        return if matches!(
            word[0],
            'A' | 'E' | 'F' | 'H' | 'I' | 'L' | 'M' | 'N' | 'O' | 'R' | 'S' | 'X'
        ) {
            InitialSound::Vowel
        } else {
            InitialSound::Consonant
        };
    }

    let word = to_lower_word(word);
    let word = word.as_ref();

    if matches!(word, ['e', 'u', 'l', 'e', ..]) {
        return InitialSound::Vowel;
    }

    if matches!(
        word,
        [] | ['u', 'k', ..]
            | ['e', 'u', 'p', 'h', ..]
            | ['e', 'u', 'g' | 'l' | 'c', ..]
            | ['o', 'n', 'e']
            | ['o', 'n', 'c', 'e']
    ) {
        return InitialSound::Consonant;
    }

    if matches!(word, |['h', 'o', 'u', 'r', ..]| ['h', 'o', 'n', ..]
        | ['u', 'n', 'i', 'n' | 'm', ..]
        | ['u', 'n', 'a' | 'u', ..]
        | ['u', 'r', 'b', ..]
        | ['i', 'n', 't', ..])
    {
        return InitialSound::Vowel;
    }

    if matches!(word, ['h', 'e', 'r', 'b', ..] if dialect == Dialect::American || dialect == Dialect::Canadian)
    {
        return InitialSound::Vowel;
    }

    if matches!(word, ['u', 'n' | 's', 'i' | 'a' | 'u', ..]) {
        return InitialSound::Consonant;
    }

    if matches!(word, ['u', 'n', ..]) {
        return InitialSound::Vowel;
    }

    if matches!(word, ['u', 'r', 'g', ..]) {
        return InitialSound::Vowel;
    }

    if matches!(word, ['u', 't', 't', ..]) {
        return InitialSound::Vowel;
    }

    if matches!(
        word,
        ['u', 't' | 'r' | 'n', ..] | ['e', 'u', 'r', ..] | ['u', 'w', ..] | ['u', 's', 'e', ..]
    ) {
        return InitialSound::Consonant;
    }

    if matches!(word, ['o', 'n', 'e', 'a' | 'e' | 'i' | 'u', 'l' | 'd', ..]) {
        return InitialSound::Vowel;
    }

    if matches!(word, ['o', 'n', 'e', 'a' | 'e' | 'i' | 'u' | '-' | 's', ..]) {
        return InitialSound::Consonant;
    }

    if matches!(
        word,
        ['s', 'o', 's']
            | ['r', 'z', ..]
            | ['n', 'g', ..]
            | ['n', 'v', ..]
            | ['x']
            | ['x', 'b', 'o', 'x']
            | ['h', 'e', 'i', 'r', ..]
            | ['h', 'o', 'n', 'o', 'r', ..]
    ) {
        return InitialSound::Vowel;
    }

    if matches!(
        word,
        ['j', 'u' | 'o', 'n', ..] | ['j', 'u', 'r', 'a' | 'i' | 'o', ..]
    ) {
        return InitialSound::Consonant;
    }

    if matches!(word, ['x', '-' | '\'' | '.' | 'o' | 's', ..]) {
        return InitialSound::Vowel;
    }

    if word[0].is_vowel() {
        return InitialSound::Vowel;
    }

    InitialSound::Consonant
}

fn is_likely_acronym(word: &[char]) -> bool {
    fn word_contains_false_positive_sequence(word: &[char]) -> bool {
        let likely_false_positive_sequences = [['V', 'C']];
        for fp_sequence in likely_false_positive_sequences {
            if word
                .windows(fp_sequence.len())
                .any(|subslice| subslice == fp_sequence)
            {
                return true;
            }
        }
        false
    }

    // If the initialism is shorter than this, skip it.
    const MIN_LEN: usize = 3;

    if let Some(first_chars) = word.get(..MIN_LEN)
        // Unlikely to be an acronym if it contains non-alphabetic characters.
        && first_chars.iter().copied().all(char::is_alphabetic)
        && !word_contains_false_positive_sequence(word)
    {
        let vowel_map = first_chars
            .iter()
            .map(CharExt::is_vowel)
            .collect_array::<MIN_LEN>()
            .unwrap();
        matches!(vowel_map, [false, true, false] | [false, true, true])
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::AnA;
    use crate::Dialect;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn detects_html_as_vowel() {
        assert_lint_count("Here is a HTML document.", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn detects_llm_as_vowel() {
        assert_lint_count("Here is a LLM document.", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn detects_llm_hyphen_as_vowel() {
        assert_lint_count(
            "Here is a LLM-based system.",
            AnA::new(Dialect::American),
            1,
        );
    }

    #[test]
    fn detects_euler_as_vowel() {
        assert_lint_count("This is an Euler brick.", AnA::new(Dialect::American), 0);
        assert_lint_count(
            "The graph has an Eulerian tour.",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn capitalized_fourier() {
        assert_lint_count(
            "Then, perform a Fourier transform.",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn once_over() {
        assert_lint_count("give this a once-over.", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn issue_196() {
        assert_lint_count(
            "This is formatted as an `ext4` file system.",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn allows_lowercase_vowels() {
        assert_lint_count("not an error", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn allows_lowercase_consonants() {
        assert_lint_count("not a crash", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn disallows_lowercase_vowels() {
        assert_lint_count("not a error", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn disallows_lowercase_consonants() {
        assert_lint_count("not an crash", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn allows_uppercase_vowels() {
        assert_lint_count("not an Error", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn allows_uppercase_consonants() {
        assert_lint_count("not a Crash", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn disallows_uppercase_vowels() {
        assert_lint_count("not a Error", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn disallows_uppercase_consonants() {
        assert_lint_count("not an Crash", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn disallows_a_interface() {
        assert_lint_count(
            "A interface for an object that can perform linting actions.",
            AnA::new(Dialect::American),
            1,
        );
    }

    #[test]
    fn allow_issue_751() {
        assert_lint_count(
            "He got a 52% approval rating.",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn allow_an_mp_and_an_mp3() {
        assert_lint_count("an MP and an MP3?", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn disallow_a_mp_and_a_mp3() {
        assert_lint_count("a MP and a MP3?", AnA::new(Dialect::American), 2);
    }

    #[test]
    fn recognize_acronyms() {
        // a
        assert_lint_count("using a MAC address", AnA::new(Dialect::American), 0);
        assert_lint_count("a NASA spacecraft", AnA::new(Dialect::American), 0);
        assert_lint_count("a NAT", AnA::new(Dialect::American), 0);
        assert_lint_count("a REST API", AnA::new(Dialect::American), 0);
        assert_lint_count("a LIBERO", AnA::new(Dialect::American), 0);
        assert_lint_count("a README", AnA::new(Dialect::American), 0);
        assert_lint_count("a LAN", AnA::new(Dialect::American), 0);

        // an
        assert_lint_count("an RA message", AnA::new(Dialect::American), 0);
        assert_lint_count("an SI unit", AnA::new(Dialect::American), 0);
        assert_lint_count(
            "he is an MA of both Oxford and Cambridge",
            AnA::new(Dialect::American),
            0,
        );
        assert_lint_count(
            "in an FA Cup 6th Round match",
            AnA::new(Dialect::American),
            0,
        );
        assert_lint_count("a AM transmitter", AnA::new(Dialect::American), 1);
    }

    #[test]
    fn dont_misrecognize_as_acronym() {
        assert_lint_count("a UPD connection", AnA::new(Dialect::American), 0);
        assert_lint_count("a UPB device", AnA::new(Dialect::American), 0);
        assert_lint_count("a UPS or power device", AnA::new(Dialect::American), 0);
        assert_lint_count("a USB 2.0 port", AnA::new(Dialect::American), 0);
        assert_lint_count("an HEVC HLS stream", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn assorted_abbreviation_esque() {
        assert_lint_count("a udev rule", AnA::new(Dialect::American), 0);
        assert_lint_count("an mDNS tool", AnA::new(Dialect::American), 0);
        assert_lint_count("an RFLink device", AnA::new(Dialect::American), 0);
        assert_lint_count(
            "an FFmpeg-compatible input file",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn assorted() {
        assert_lint_count("a Honeywell alarm panel", AnA::new(Dialect::American), 0);
        assert_lint_count("a OneDrive folder", AnA::new(Dialect::American), 0);
        assert_lint_count(
            "a Ubiquiti UniFi Network application",
            AnA::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn dont_flag_an_herb_for_american() {
        assert_lint_count("an herb", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn dont_flag_a_herb_for_british() {
        assert_lint_count("a herb", AnA::new(Dialect::British), 0);
    }

    #[test]
    fn correct_an_herb_for_australian() {
        assert_suggestion_result("an herb", AnA::new(Dialect::Australian), "a herb");
    }

    #[test]
    fn correct_a_herb_for_canadian() {
        assert_suggestion_result("a herb", AnA::new(Dialect::Canadian), "an herb");
    }

    #[test]
    fn dont_flag_a_sql() {
        assert_lint_count("a SQL query", AnA::new(Dialect::American), 0);
    }

    #[test]
    fn dont_flag_an_sql() {
        assert_lint_count("an SQL query", AnA::new(Dialect::Australian), 0);
    }
}
