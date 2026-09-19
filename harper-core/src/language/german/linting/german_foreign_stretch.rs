//! Recognising a stretch of another language inside a German text.
//!
//! German academic prose quotes constantly, and mostly not in German: a book
//! title, a term of art, a line of Kant in the spelling of 1781, three words of
//! Latin. Every German capitalization rule then fires on it — *„to keep and
//! bear **arms**"*, *„an **introduction** to the study of speech"*, *„the
//! Cecropia-Azteca **association** in Costa Rica"* — because a lower-case
//! English noun looks exactly like a German noun that lost its capital.
//!
//! Measured on 1.44M words of German prose, one lint in six that is not a
//! spelling mistake sits inside a run of three or more English words. That is
//! the largest false-positive class the German rules have left.
//!
//! **Asking a dictionary does not work.** The obvious test — is this word
//! German? — fails on precisely the words that matter: Harper's German
//! dictionary accepts `arms` (genitive of *Arm*), `people`, `high`, `action`
//! and `drama`, either outright or through compound splitting. Of the lints
//! inside an English run, that test catches one in eight.
//!
//! What does work is a closed list of function words the other language has
//! and German does not, counted in a window. A single one proves nothing —
//! anything can appear in a title — but two inside a few tokens is a sentence
//! that has stopped being German.

use crate::{Token, TokenKind, document::Document};

/// Function words of the languages German prose quotes, chosen so that none of
/// them is also a German word.
///
/// That constraint is the whole design and it is not obvious from reading:
/// `also`, `all`, `will`, `war`, `man`, `in`, `an`, `was`, `not`, `such`,
/// `per`, `pro`, `art`, `arm`, `first`, `last`, `after`, `before`, `am`,
/// `made` and `most` are every bit as English as the entries below and every
/// one of them is German too. Check a candidate against the German dictionary
/// before adding it — `harper-cli lint -d de -o` on a frame sentence says so
/// in one command.
const FOREIGN_FUNCTION_WORDS: &[&str] = &[
    // English
    "the", "of", "and", "for", "with", "from", "their", "its", "his", "her", "they", "these",
    "those", "which", "that", "about", "into", "upon", "between", "among", "were", "been", "being",
    "there", "when", "where", "would", "could", "should", "to", "is", "as", "at", "by", "this",
    "are", "it", "have", "had", "but", "we", "you", "can", "more", "than", "may", "must", "shall",
    "our", "your", "on", "or", "what", "how", "them", "us", "him", "he", "she", "if", "who",
    "whom", "whose", "why", "very", "much", "many", "only", "some", "any", "each", "through",
    "during", "against", "because", "while", "both", "then", "other", "others", "own", "do",
    "does", "did", // French
    "la", "le", "les", "du", "de", "au", "aux", "et", "une", "dans", "sur", "pour", "avec", "sans",
    "chez", "leur", "ses", "son", "sa", "cette", "ces", "qui", "que",
    // Italian / Spanish / Portuguese
    "della", "delle", "degli", "dei", "nel", "nella", "il", "lo", "gli", "una", "col", "por",
    "para", "los", "las", "del", "el", "uma", "dos", // Latin
    "apud", "atque", "quae", "quod", "cum", "sive", "seu", "ratione", "liber", "libri",
];

/// How many tokens either side of the candidate are consulted.
const WINDOW: usize = 5;

/// The lower-case spelling of a token.
pub fn lowercase_of(token: &Token, document: &Document) -> String {
    document
        .get_span_content(&token.span)
        .iter()
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Is the token at `index` inside a stretch of another language?
///
/// German prose quotes foreign titles without translating them, and a
/// bibliography is mostly that: *"The Modes of scepticism: ancient **texts**
/// and modern interpretations"*, *"Galien et la **philosophie**"*, *"Memorie
/// della Reale Accademia **delle** Scienze"*. Several of those words are in
/// the German dictionary with a noun reading — `texts`, `model`, `period`,
/// `zone`, `roman` — so every one of them is reported as a lower-case German
/// noun.
///
/// The neighbourhood settles it. Two kinds of evidence count, within [`WINDOW`]
/// word tokens on either side:
///
/// * a function word no German sentence contains — `the`, `of`, `la`, `du`,
///   `et`, `della`. These are the strongest signal and the most common.
/// * a word the dictionary does not know at all. Latin and taxonomic names
///   carry no function words — *"Conspectus generum avium"*, *"Mellisuga
///   minima vielloti"* — and are nothing but unknown words.
///
/// At least one function word is **required**, and one more point has to come
/// from somewhere — a second function word, or an unknown word. Unknown words
/// alone are not enough and the difference is large: German Wikipedia is full
/// of proper names the dictionary does not have, and letting two of those
/// silence the rule cost a fifth of the injected lower-case nouns in
/// `just language-recall german`.
///
/// The price is the Latin and taxonomic runs, which carry no function word at
/// all — *"Conspectus generum avium"* stays flagged. A handful of those against
/// several hundred real errors is the right way round.
///
/// `is_german_determiner` is supplied by the caller rather than kept here: it
/// needs the dictionary, and only the noun rule has one.
pub fn in_foreign_stretch(
    tokens: &[&Token],
    index: usize,
    document: &Document,
    mut is_german_determiner: impl FnMut(&Token) -> bool,
) -> bool {
    // A German determiner directly in front settles it the other way: *"durch
    // die Zeitschrift Le Mercure Galant"*, *"an der University of Virginia"*
    // are German sentences that happen to name something foreign, and the
    // word after the article is a German noun.
    if index
        .checked_sub(1)
        .is_some_and(|previous| is_german_determiner(tokens[previous]))
    {
        return false;
    }

    let mut function_words = 0;
    let mut unknown_words = 0;
    let mut adjacent_function_word = false;

    let mut visit = |token: &Token, distance: usize| {
        if !matches!(token.kind, TokenKind::Word(_)) {
            return;
        }
        if FOREIGN_FUNCTION_WORDS.contains(&lowercase_of(token, document).as_str()) {
            function_words += 1;
            adjacent_function_word |= distance == 1;
        } else if token.kind.is_oov() {
            unknown_words += 1;
        }
    };

    for (offset, token) in tokens[..index].iter().rev().take(WINDOW).enumerate() {
        visit(token, offset + 1);
    }
    for (offset, token) in tokens.iter().skip(index + 1).take(WINDOW).enumerate() {
        visit(token, offset + 1);
    }

    function_words >= 2 || (adjacent_function_word && function_words + unknown_words >= 2)
}

#[cfg(test)]
mod tests {
    use super::FOREIGN_FUNCTION_WORDS;

    /// Words that are English (or French, or Latin) *and* German. Every one of
    /// them is a plausible addition to the list and every one would silence
    /// German capitalization on ordinary German sentences: `also` and `war`
    /// alone would cover a good part of any article.
    ///
    /// The dictionary cannot be asked this. It holds `with`, `from`, `that`
    /// and `about` as compound elements, and the compound splitter then
    /// accepts them as words — that permissiveness is the very thing this
    /// module works around, so a dictionary lookup would reject the entries
    /// that work and admit nothing new. The list is therefore checked against
    /// a hand-kept list of traps.
    const ALSO_GERMAN: &[&str] = &[
        "also", "all", "alle", "will", "war", "man", "in", "an", "was", "not", "such", "per",
        "pro", "art", "arm", "band", "bad", "first", "last", "after", "before", "out", "down",
        "am", "made", "make", "most", "same", "has", "be", "list", "mode", "so", "wer", "hat",
        "bald", "fast", "gift", "rat", "ohne", "bin", "sie", "wie", "sind",
    ];

    #[test]
    fn no_entry_is_also_a_german_word() {
        let trapped: Vec<&&str> = FOREIGN_FUNCTION_WORDS
            .iter()
            .filter(|word| ALSO_GERMAN.contains(word))
            .collect();

        assert!(
            trapped.is_empty(),
            "these are German words too and must not be in FOREIGN_FUNCTION_WORDS: {trapped:?}"
        );
    }

    #[test]
    fn the_list_has_no_duplicates() {
        let mut seen = FOREIGN_FUNCTION_WORDS.to_vec();
        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();
        assert_eq!(
            before,
            seen.len(),
            "FOREIGN_FUNCTION_WORDS repeats an entry"
        );
    }
}
