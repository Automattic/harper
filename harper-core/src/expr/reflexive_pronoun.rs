use crate::{
    Span, Token,
    expr::{Expr, LongestMatchOf},
    patterns::WordSet,
};

// These are considered ungrammatical, or are at least not in `dictionary.dict` but are commonly used anyway.
// The tests below check if this changes so we can update this `Expr`
const BAD_REFLEXIVE_PRONOUNS: &[&str] = &[
    "hisself",
    "oneselves",
    "theirself",
    "theirselves",
    "themself",
];

#[derive(Default)]
pub struct ReflexivePronoun;

impl Expr for ReflexivePronoun {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span> {
        let good_pronouns = |token: &Token, _: &[char]| token.kind.is_reflexive_pronoun();
        let bad_pronouns = WordSet::new(BAD_REFLEXIVE_PRONOUNS);
        let expr = LongestMatchOf::new(vec![Box::new(good_pronouns), Box::new(bad_pronouns)]);
        expr.run(cursor, tokens, source)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Document, TokenKind, expr::reflexive_pronoun::BAD_REFLEXIVE_PRONOUNS};

    // These are considered grammatically correct, or are at least in `dictionary.dict`.
    // The tests below check if this changes so we can update this `Expr`
    const GOOD_REFLEXIVE_PRONOUNS: &[&str] = &[
        "herself",
        "himself",
        "itself",
        "myself",
        "oneself",
        "ourself",
        "ourselves",
        "themselves",
        "thyself",
        "yourself",
        "yourselves",
    ];

    fn test_pronoun(word: &str) {
        let doc = Document::new_plain_english_curated(word);
        let token = doc.tokens().next().expect("No tokens in document");

        let is_good_pron = GOOD_REFLEXIVE_PRONOUNS.contains(&word);
        let is_bad_pron = BAD_REFLEXIVE_PRONOUNS.contains(&word);

        match (is_good_pron, is_bad_pron, &token.kind) {
            (true, false, TokenKind::Word(Some(md))) => {
                assert!(md.is_pronoun());
                assert!(md.is_reflexive_pronoun());
            }
            (true, false, TokenKind::Word(None)) => {
                panic!("Widely accepted pronoun '{word}' has gone missing from the dictionary!")
            }
            (false, true, TokenKind::Word(Some(_))) => panic!(
                "Unaccepted pronoun '{word}' that's used in bad English is now in the dictionary!"
            ),
            (false, true, TokenKind::Word(None)) => {}
            (false, false, TokenKind::Word(Some(_))) => panic!(
                "non-pronoun '{word}' is made up just for testing but is now in the dictionary!"
            ),
            (false, false, TokenKind::Word(None)) => {}
            (true, true, _) => panic!("'{word}' is in both good and bad lists"),
            _ => panic!("'{word}' doesn't match any expected case"),
        }
    }

    #[test]
    fn test_good_reflexive_pronouns() {
        for word in GOOD_REFLEXIVE_PRONOUNS {
            test_pronoun(word);
        }
    }

    #[test]
    fn test_bad_reflexive_pronouns() {
        for word in BAD_REFLEXIVE_PRONOUNS {
            test_pronoun(word);
        }
    }

    // It's expected that nobody uses these words even in bad English.
    #[test]
    fn test_non_pronouns() {
        test_pronoun("myselves");
        test_pronoun("weselves");
        test_pronoun("usself");
        test_pronoun("usselves");
    }
}
