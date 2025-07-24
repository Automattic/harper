use crate::{
    Span, Token,
    expr::{Expr, FirstMatchOf},
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

/// Matches reflexive pronouns with configurable strictness.
///
/// By default, only matches standard English reflexive pronouns. Use `with_common_errors()` to include
/// frequently encountered non-standard forms like "hisself" or "theirself".
pub struct ReflexivePronoun {
    include_common_errors: bool,
}

impl Default for ReflexivePronoun {
    fn default() -> Self {
        Self::standard()
    }
}

impl ReflexivePronoun {
    /// Creates a matcher for standard English reflexive pronouns.
    ///
    /// Matches only the correct forms: "myself", "yourself", "himself", "herself", "itself",
    /// "ourselves", "yourselves", and "themselves".
    pub fn standard() -> Self {
        Self {
            include_common_errors: false,
        }
    }

    /// Creates a matcher that includes non-standard but commonly used reflexive pronouns.
    ///
    /// In addition to standard forms, matches common errors like "hisself", "theirself",
    /// and other non-standard forms that are frequently seen in user-generated content.
    pub fn with_common_errors() -> Self {
        Self {
            include_common_errors: true,
        }
    }
}

impl Expr for ReflexivePronoun {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        let good_pronouns = |token: &Token, _: &[char]| token.kind.is_reflexive_pronoun();
        let mut expr = FirstMatchOf::new(vec![Box::new(good_pronouns)]);
        if self.include_common_errors {
            expr.add(WordSet::new(BAD_REFLEXIVE_PRONOUNS));
        }
        expr.run(cursor, tokens, source)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Document, TokenKind,
        expr::{ExprExt, ReflexivePronoun, reflexive_pronoun::BAD_REFLEXIVE_PRONOUNS},
    };

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

    #[test]
    fn ensure_standard_ctor_includes_myself() {
        let doc =
            Document::new_plain_english_curated("If you want something done, do it yourself.");
        let rp = ReflexivePronoun::standard();
        let matches = rp.iter_matches_in_doc(&doc);
        assert_eq!(matches.count(), 1);
    }

    #[test]
    fn ensure_default_ctor_includes_myself() {
        let doc = Document::new_plain_english_curated(
            "I wanted a reflexive pronoun module, so I wrote one myself.",
        );
        let rp = ReflexivePronoun::default();
        let matches = rp.iter_matches_in_doc(&doc);
        assert_eq!(matches.count(), 1);
    }

    #[test]
    fn ensure_with_common_errors_includes_hisself() {
        let doc = Document::new_plain_english_curated("He teached hisself English.");
        let rp = ReflexivePronoun::with_common_errors();
        let matches = rp.iter_matches_in_doc(&doc);
        assert_eq!(matches.count(), 1);
    }

    #[test]
    fn ensure_standard_ctor_excludes_hisself() {
        let doc = Document::new_plain_english_curated("Was he pleased with hisself?");
        let rp = ReflexivePronoun::standard();
        let matches = rp.iter_matches_in_doc(&doc);
        assert_eq!(matches.count(), 0);
    }

    #[test]
    fn ensure_default_ctor_excludes_theirself() {
        let doc = Document::new_plain_english_curated("They look at theirself in the mirror.");
        let rp = ReflexivePronoun::default();
        let matches = rp.iter_matches_in_doc(&doc);
        assert_eq!(matches.count(), 0);
    }
}
