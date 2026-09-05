use hashbrown::HashMap;

use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::spell::Dictionary;
use crate::{CharStringExt, TokenStringExt, document::Document};

const MIN_COMPOUND_PART_LEN: usize = 3;
const MAX_COMPOUND_PARTS: usize = 5;
const EMPTY_INTERFIX: &[char] = &[];
const S_INTERFIX: &[char] = &['s'];
const N_INTERFIX: &[char] = &['n'];
const EN_INTERFIX: &[char] = &['e', 'n'];
const ER_INTERFIX: &[char] = &['e', 'r'];
const ES_INTERFIX: &[char] = &['e', 's'];
const GERMAN_COMPOUND_INTERFIXES: &[&[char]] = &[
    EMPTY_INTERFIX,
    S_INTERFIX,
    N_INTERFIX,
    EN_INTERFIX,
    ER_INTERFIX,
    ES_INTERFIX,
];

/// Common German orthographic alternations, taken from the `REP`/`MAP`
/// replacement tables of the igerman98/de_DE hunspell dictionary.
///
/// These capture regular spelling correspondences that a plain edit distance
/// counts as one or two arbitrary edits but that a writer treats as a single,
/// very plausible swap: `ae`/`ä`, `ss`/`ß`, `i`/`ie`, silent `h` after vowels,
/// `ee`/`e`, and `f`/`ph`. Consonant-devoicing pairs from hunspell such as
/// `d`/`t` and `ch`/`k` are deliberately omitted: they overmatch at arbitrary
/// positions (a doubled-letter typo like `Hundd` must fix to `Hund`, not to a
/// rare `d`→`t` variant such as `Hundt`).
const GERMAN_ALTERNATIONS: &[(&[char], &[char])] = &[
    (&['ä'], &['a', 'e']),
    (&['ö'], &['o', 'e']),
    (&['ü'], &['u', 'e']),
    (&['ß'], &['s', 's']),
    (&['i'], &['i', 'e']),
    (&['e'], &['e', 'e']),
    (&['e'], &['e', 'h']),
    (&['o'], &['o', 'h']),
    (&['a'], &['a', 'h']),
    (&['f'], &['p', 'h']),
    (&['t'], &['t', 'h']),
    (&['r'], &['r', 'h']),
];

/// The kind of error that would turn a misspelled word into a candidate
/// suggestion.
///
/// Harper's shared suggestion scorer is tuned for English. German suggestions
/// are ranked the way hunspell ranks them: by the *kind* of mistake the writer
/// most likely made, in hunspell's own heuristic order. Orthographic
/// alternations (`geschriben` → `geschrieben`, `Baeume` → `Bäume`), doubled
/// letters (`Worrt` → `Wort`) and transpositions (`flasch` → `falsch`,
/// `Wrot` → `Wort`) are far more plausible than a wholesale letter
/// substitution (`Worrt` → `Wirrt`), so candidates reachable by one of those
/// errors outrank candidates that require a substitution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum GermanEditType {
    /// A German orthographic alternation such as `ss` ↔ `ß` or `i` ↔ `ie`.
    Alternation,
    /// Two adjacent characters were swapped.
    Transposition,
    /// The misspelled word contains one extra character.
    Deletion,
    /// The misspelled word is missing one character.
    Insertion,
    /// A single character was replaced with a different one.
    Substitution,
    /// Some other edit (or several edits).
    Other,
}

impl GermanEditType {
    /// Hunspell emits its simple heuristics in a fixed order, so a candidate
    /// produced by an earlier (more plausible) heuristic always outranks one
    /// produced by a later one. We mirror that by mapping each kind of edit to
    /// a coarse "bucket"; the bucket dominates finer-grained similarity.
    fn bucket(self) -> i32 {
        match self {
            GermanEditType::Alternation => 0,
            GermanEditType::Transposition => 1,
            GermanEditType::Deletion => 2,
            GermanEditType::Insertion => 3,
            GermanEditType::Substitution => 4,
            GermanEditType::Other => 5,
        }
    }
}

/// Compare two characters, ignoring case.
fn eq_ignore_case(a: char, b: char) -> bool {
    let mut a_lower = a.to_lowercase();
    let mut b_lower = b.to_lowercase();
    loop {
        match (a_lower.next(), b_lower.next()) {
            (None, None) => return true,
            (Some(a), Some(b)) if a == b => continue,
            _ => return false,
        }
    }
}

/// Case-insensitive equality of two character slices.
fn slices_eq_ignore_case(a: &[char], b: &[char]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| eq_ignore_case(*x, *y))
}

/// Returns true when one word differs from the other by a single German
/// orthographic alternation (a `GERMAN_ALTERNATIONS` pair applied once).
fn is_german_alternation(a: &[char], b: &[char]) -> bool {
    GERMAN_ALTERNATIONS
        .iter()
        .any(|(from, to)| replaces_to_equal(a, from, to, b) || replaces_to_equal(b, from, to, a))
}

/// Returns true when replacing a single occurrence of `from` in `source` with
/// `to` produces `target`.
fn replaces_to_equal(source: &[char], from: &[char], to: &[char], target: &[char]) -> bool {
    if source.len() < from.len() || source.len() - from.len() + to.len() != target.len() {
        return false;
    }

    let last_start = source.len() - from.len();
    for start in 0..=last_start {
        if slices_eq_ignore_case(&source[start..start + from.len()], from)
            && slices_eq_ignore_case(&source[..start], &target[..start])
            && slices_eq_ignore_case(to, &target[start..start + to.len()])
            && slices_eq_ignore_case(&source[start + from.len()..], &target[start + to.len()..])
        {
            return true;
        }
    }

    false
}

/// Returns true when `longer` equals `shorter` with exactly one character
/// removed.
fn is_single_deletion(longer: &[char], shorter: &[char]) -> bool {
    if longer.len() != shorter.len() + 1 {
        return false;
    }

    let mut i = 0; // index into `longer`
    let mut j = 0; // index into `shorter`
    let mut skipped = false;

    while i < longer.len() {
        if j < shorter.len() && eq_ignore_case(longer[i], shorter[j]) {
            i += 1;
            j += 1;
        } else if !skipped {
            skipped = true;
            i += 1;
        } else {
            return false;
        }
    }

    skipped && j == shorter.len()
}

/// Classify the edit that most simply transforms `misspelled` into `candidate`.
fn classify_german_edit(misspelled: &[char], candidate: &[char]) -> GermanEditType {
    use GermanEditType::{Deletion, Insertion, Other, Substitution, Transposition};

    // Orthographic alternations are checked first: they are the single most
    // plausible class of German spelling fix.
    if is_german_alternation(misspelled, candidate) {
        return GermanEditType::Alternation;
    }

    if misspelled.len() == candidate.len() {
        // Same length: exactly one differing position is a substitution, two
        // adjacent swapped positions are a transposition.
        let mut diff_positions = [0usize; 2];
        let mut diff_count = 0usize;

        for (i, (a, b)) in misspelled.iter().zip(candidate.iter()).enumerate() {
            if !eq_ignore_case(*a, *b) {
                if diff_count == 2 {
                    return Other;
                }
                diff_positions[diff_count] = i;
                diff_count += 1;
            }
        }

        return match diff_count {
            0 => Other, // exact match, filtered out upstream
            1 => Substitution,
            2 => {
                let (first, second) = (diff_positions[0], diff_positions[1]);
                if second == first + 1
                    && eq_ignore_case(misspelled[first], candidate[second])
                    && eq_ignore_case(misspelled[second], candidate[first])
                {
                    Transposition
                } else {
                    Other
                }
            }
            _ => unreachable!(),
        };
    }

    if misspelled.len() == candidate.len() + 1 {
        return if is_single_deletion(misspelled, candidate) {
            Deletion
        } else {
            Other
        };
    }

    if misspelled.len() + 1 == candidate.len() {
        return if is_single_deletion(candidate, misspelled) {
            Insertion
        } else {
            Other
        };
    }

    Other
}

/// Score a candidate suggestion for a German misspelling.
///
/// Lower is better. The error *kind* dominates (mirroring hunspell's heuristic
/// ordering); edit distance and character-level similarity then refine the
/// ranking within each kind.
fn score_german_candidate(misspelled: &[char], candidate: &[char], edit_distance: u8) -> i32 {
    let kind = classify_german_edit(misspelled, candidate);
    let mut score = kind.bucket() * 1000 + i32::from(edit_distance) * 10;

    // Similarity bonuses: a shared prefix and matching character positions.
    let misspelled_lower = misspelled.to_lower();
    let candidate_lower = candidate.to_lower();

    let common_len = misspelled_lower.len().min(candidate_lower.len());
    let mut prefix = 0;
    while prefix < common_len && misspelled_lower[prefix] == candidate_lower[prefix] {
        prefix += 1;
    }

    let common_positions = misspelled_lower
        .iter()
        .zip(candidate_lower.iter())
        .filter(|(a, b)| a == b)
        .count();

    score -= i32::try_from(prefix).unwrap_or(i32::MAX) * 2;
    score -= i32::try_from(common_positions).unwrap_or(i32::MAX);

    score
}

/// A spell checker for German text with compound word handling.
pub struct GermanSpellCheck<T>
where
    T: Dictionary,
{
    dictionary: T,
}

impl<T: Dictionary> GermanSpellCheck<T> {
    pub fn new(dictionary: T) -> Self {
        Self { dictionary }
    }

    fn strip_compound_interfix<'a>(
        &self,
        remainder: &'a [char],
        interfix: &[char],
    ) -> Option<&'a [char]> {
        remainder.strip_prefix(interfix)
    }

    fn is_valid_compound_segment(
        &self,
        word: &[char],
        depth: usize,
        memo: &mut HashMap<Vec<char>, bool>,
    ) -> bool {
        if word.len() < MIN_COMPOUND_PART_LEN {
            return false;
        }

        if depth >= MAX_COMPOUND_PARTS {
            return false;
        }

        if depth > 0 && self.dictionary.contains_word(word) {
            return true;
        }

        if let Some(cached) = memo.get(word) {
            return *cached;
        }

        let mut valid = false;

        for split_pos in MIN_COMPOUND_PART_LEN..=word.len() - MIN_COMPOUND_PART_LEN {
            let first_part = &word[..split_pos];

            if !self.dictionary.contains_word(first_part) {
                continue;
            }

            let remainder = &word[split_pos..];

            for interfix in GERMAN_COMPOUND_INTERFIXES {
                let Some(next_part) = self.strip_compound_interfix(remainder, interfix) else {
                    continue;
                };

                if next_part.len() < MIN_COMPOUND_PART_LEN {
                    continue;
                }

                // In German, compound noun parts are capitalized. Try both the original
                // and capitalized versions of the next part.
                let mut capitalized_next_part = next_part.to_vec();
                if let Some(first_char) = capitalized_next_part.first_mut() {
                    *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
                }

                if self.dictionary.contains_word(next_part)
                    || self.dictionary.contains_word(&capitalized_next_part)
                    || self.is_valid_compound_segment(next_part, depth + 1, memo)
                    || self.is_valid_compound_segment(&capitalized_next_part, depth + 1, memo)
                {
                    valid = true;
                    break;
                }
            }

            if valid {
                break;
            }
        }

        memo.insert(word.to_vec(), valid);
        valid
    }

    /// Check if a word is a valid German compound.
    /// German freely combines nouns and often inserts linking morphemes such as
    /// `s`, `n`, `en`, `er`, `e`, or `es` between parts.
    fn try_compound_word_check(&self, word: &[char]) -> bool {
        if word.len() < MIN_COMPOUND_PART_LEN * 2 {
            return false;
        }

        let mut memo = HashMap::new();
        self.is_valid_compound_segment(word, 0, &mut memo)
    }

    /// Get spelling suggestions for a word.
    ///
    /// Candidates come from the (FST-backed) dictionary's fuzzy match, which
    /// keeps lookups fast even over a large word list. They are then ranked
    /// with [`score_german_candidate`], which prefers the error kinds writers
    /// actually make (transpositions, doubled letters) over arbitrary
    /// substitutions — the same strategy hunspell uses for German.
    fn get_suggestions(&self, word: &[char]) -> Vec<Vec<char>> {
        // Pull a generous candidate set, then rank it ourselves. The default
        // Harper ranking is tuned for English and ties are broken
        // alphabetically, which produces poor results for German.
        let mut scored: Vec<(i32, &[char])> = self
            .dictionary
            .fuzzy_match(word, 2, 100)
            .into_iter()
            .map(|result| {
                (
                    score_german_candidate(word, result.word, result.edit_distance),
                    result.word,
                )
            })
            .collect();

        scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));

        let mut suggestions: Vec<Vec<char>> = scored
            .into_iter()
            .map(|(_, candidate)| candidate.to_vec())
            .collect();

        // Preserve the input's capitalization. German nouns and sentence-initial
        // words are capitalized, so a capitalized misspelling ("Worrt") implies a
        // capitalized correction ("Wort").
        if word.first().is_some_and(|c| c.is_uppercase()) {
            for suggestion in suggestions.iter_mut() {
                let has_internal_caps = suggestion.iter().skip(1).any(|c| c.is_uppercase());
                if !has_internal_caps && let Some(first) = suggestion.first_mut() {
                    *first = first.to_uppercase().next().unwrap();
                }
            }
        }

        // Also try a simple capitalization fix (common German error).
        if suggestions.is_empty() && word.len() > 1 {
            let mut capitalized = word.to_vec();
            if let Some(first_char) = capitalized.first_mut() {
                *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
            }
            if self.dictionary.contains_word(&capitalized) {
                suggestions.push(capitalized);
            }
        }

        suggestions.truncate(5);
        suggestions
    }
}

impl<T: Dictionary> Linter for GermanSpellCheck<T> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for paragraph in document.iter_paragraphs() {
            for sentence in paragraph.iter_sentences() {
                for word in sentence.iter_words() {
                    let word_chars = document.get_span_content(&word.span);

                    // Skip words in dictionary
                    if self.dictionary.contains_word(word_chars) {
                        continue;
                    }

                    // Try compound word splitting
                    if self.try_compound_word_check(word_chars) {
                        continue;
                    }

                    // Get spelling suggestions
                    let suggestions = self.get_suggestions(word_chars);
                    let word_str: String = word_chars.iter().collect();

                    let message = if !suggestions.is_empty() {
                        let suggestions_str: Vec<String> = suggestions
                            .iter()
                            .map(|s| s.iter().collect::<String>())
                            .collect();
                        format!(
                            "Possible spelling error: \"{}\". Did you mean: {}?",
                            word_str,
                            suggestions_str.join(", ")
                        )
                    } else {
                        format!("Unknown word: \"{}\".", word_str)
                    };

                    lints.push(Lint {
                        span: word.span,
                        lint_kind: LintKind::Spelling,
                        suggestions: suggestions
                            .into_iter()
                            .map(Suggestion::ReplaceWith)
                            .collect(),
                        priority: 20,
                        message,
                    });
                }
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Checks for spelling errors in German text"
    }
}

#[cfg(test)]
mod tests {
    use super::GermanSpellCheck;
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::combined_german_dictionary;
    use crate::linting::Linter;
    use crate::spell::Dictionary;

    fn lint_text(text: &str) -> Vec<String> {
        use crate::language::german::linting::new_curated_german;
        use crate::language::german::spell::combined_german_dictionary;
        let dict = combined_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::Standard, dict.clone());
        let document = Document::new(text, &PlainGerman, &dict);

        linter
            .lint(&document)
            .into_iter()
            .map(|lint| lint.message)
            .collect()
    }

    fn recognizes_compound(word: &str) -> bool {
        let dict = combined_german_dictionary();
        let spellcheck = GermanSpellCheck::new(dict);
        let chars: Vec<char> = word.chars().collect();

        spellcheck.try_compound_word_check(&chars)
    }

    #[test]
    fn recognizes_recursive_compounds() {
        for word in [
            "Gartenhaus",
            "Arbeitsstelle",
            "Frühstücksspeck",
            "Straßenrand",
            "Festplattenspeicher",
        ] {
            assert!(
                recognizes_compound(word),
                "{word} should be treated as a valid compound"
            );
        }
    }

    #[test]
    fn does_not_accept_misspelled_compounds() {
        for word in ["Festplattenspeicer", "Arbeitsplaz", "Straßenrant"] {
            assert!(
                !recognizes_compound(word),
                "{word} should not be treated as a valid compound"
            );
        }
    }

    #[test]
    fn recognizes_simple_compounds() {
        for word in ["Gartenhaus", "Arbeitsstelle", "Straßenrand"] {
            assert!(
                recognizes_compound(word),
                "{word} should be treated as a valid compound"
            );
        }
    }

    #[test]
    fn lint_allows_festplattenspeicher() {
        let messages = lint_text("Der Festplattenspeicher ist fast voll.");

        assert!(
            messages
                .iter()
                .all(|message| !message.contains("Festplattenspeicher")),
            "Festplattenspeicher should not be flagged: {messages:?}"
        );
    }

    #[test]
    fn lint_flags_misspelled_storage_compounds() {
        let messages = lint_text("Der Festplattenspeicer ist fast voll.");

        assert!(
            messages
                .iter()
                .any(|message| message.contains("Festplattenspeicer")),
            "Misspelled compound should still be flagged: {messages:?}"
        );
    }

    #[test]
    fn lint_allows_common_technical_compounds() {
        let messages = lint_text(
            "Die Systemvoraussetzungen sind dokumentiert. \
             Das Betriebssystem nutzt eine Konfigurationsdatei im Texteditor zur Fehlerbehebung. \
             Ihre Unterschrift und die Unterschriften sind vorhanden.",
        );

        for word in [
            "Systemvoraussetzungen",
            "Betriebssystem",
            "Konfigurationsdatei",
            "Texteditor",
            "Fehlerbehebung",
            "Unterschrift",
            "Unterschriften",
        ] {
            assert!(
                messages.iter().all(|message| !message.contains(word)),
                "{word} should not be flagged: {messages:?}"
            );
        }
    }

    #[test]
    fn curated_german_uses_german_spellcheck_instead_of_generic_spellcheck() {
        use crate::language::german::linting::new_curated_german;
        use crate::language::german::spell::curated_german_dictionary;
        let linter = new_curated_german(GermanDialect::Standard, curated_german_dictionary());

        assert!(linter.config.is_rule_enabled("GermanSpellCheck"));
        assert!(!linter.config.is_rule_enabled("SpellCheck"));
    }

    fn suggestions_for(word: &str) -> Vec<String> {
        let dict = combined_german_dictionary();
        let spellcheck = GermanSpellCheck::new(dict);
        let chars: Vec<char> = word.chars().collect();
        spellcheck
            .get_suggestions(&chars)
            .into_iter()
            .map(|s| s.into_iter().collect())
            .collect()
    }

    /// These expectations mirror hunspell's de_DE suggestion ordering: the
    /// intended word is a deletion, transposition, or orthographic alternation
    /// away from the typo, and those error kinds must outrank plain letter
    /// substitutions.
    #[test]
    fn suggests_intended_word_first() {
        for (typo, intended) in [
            ("Worrt", "Wort"),             // doubled letter (deletion)
            ("Hundd", "Hund"),             // doubled letter (deletion)
            ("Gartehn", "Garten"),         // stray letter (deletion)
            ("Sprechenn", "Sprechen"),     // doubled letter (deletion)
            ("flasch", "falsch"),          // transposition
            ("Wrot", "Wort"),              // transposition
            ("geschriben", "geschrieben"), // i -> ie alternation
        ] {
            let suggestions = suggestions_for(typo);
            assert_eq!(
                suggestions.first().map(String::as_str),
                Some(intended),
                "expected '{intended}' as the top suggestion for '{typo}', got {suggestions:?}"
            );
        }
    }

    #[test]
    fn suggested_words_are_valid_dictionary_words() {
        let dict = combined_german_dictionary();
        let spellcheck = GermanSpellCheck::new(dict.clone());

        for typo in [
            "Worrt",
            "flasch",
            "geschriben",
            "Gartehn",
            "Hundd",
            "Sprechenn",
        ] {
            let chars: Vec<char> = typo.chars().collect();
            let suggestions = spellcheck.get_suggestions(&chars);

            assert!(!suggestions.is_empty(), "expected suggestions for '{typo}'");
            for suggestion in &suggestions {
                assert!(
                    dict.contains_word(suggestion),
                    "suggestion '{}' for '{typo}' is not a valid German word",
                    suggestion.iter().collect::<String>()
                );
            }
        }
    }

    #[test]
    fn preserves_capitalization_of_capitalized_typo() {
        let suggestions = suggestions_for("Worrt");
        assert_eq!(
            suggestions.first().map(String::as_str),
            Some("Wort"),
            "got {suggestions:?}"
        );
    }
}
