use crate::{
    CharStringExt, Dialect, Lint, Token,
    expr::Expr,
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::WordSet,
};
use harper_brill::UPOS;

use Dialect::{Australian, British, Canadian, Indian};

/// Nouns and verbs that share a pronunciation but have different spellings in the listed
/// dialects. Elsewhere, the verb spelling serves for both, as `license` does in American English.
const NOUN_VERB_PAIRS: &[(&str, &str, &[Dialect])] = &[
    (
        "licence",
        "license",
        &[Australian, British, Canadian, Indian],
    ),
    (
        "licences",
        "licenses",
        &[Australian, British, Canadian, Indian],
    ),
    (
        "practice",
        "practise",
        &[Australian, British, Canadian, Indian],
    ),
    (
        "practices",
        "practises",
        &[Australian, British, Canadian, Indian],
    ),
    // Canadian English uses `tire` for both.
    ("tyre", "tire", &[Australian, British, Indian]),
    ("tyres", "tires", &[Australian, British, Indian]),
];

/// Words that come before a noun but not a verb.
const BEFORE_NOUN: &[&str] = &[
    "a", "an", "the", "this", "these", "those", "my", "your", "his", "its", "our", "their", "any",
    "no", "each", "every", "in", "into", "of", "without", "under",
];

/// Words that come before a verb but not a noun.
const BEFORE_VERB: &[&str] = &[
    "i", "we", "you", "they", "he", "she", "will", "would", "can", "could", "shall", "should",
    "may", "might", "must", "do", "does", "did", "don't", "doesn't", "didn't", "cannot", "can't",
    "won't",
];

/// Words that start the object of a verb, so `to` before the noun spelling marks an infinitive.
const OBJECT_START: &[&str] = &[
    "a", "an", "the", "this", "that", "my", "your", "his", "her", "its", "our", "their", "it",
    "them", "him", "us", "me", "you",
];

pub struct NounVerbSpelling {
    expr: WordSet,
    pairs: Vec<(&'static str, &'static str)>,
    dialect: Dialect,
}

impl NounVerbSpelling {
    pub fn new(dialect: Dialect) -> Self {
        let pairs: Vec<_> = NOUN_VERB_PAIRS
            .iter()
            .filter(|(_, _, dialects)| dialects.contains(&dialect))
            .map(|&(noun, verb, _)| (noun, verb))
            .collect();

        Self {
            expr: WordSet::new(
                pairs
                    .iter()
                    .flat_map(|&(noun, verb)| [noun, verb])
                    .collect::<Vec<_>>(),
            ),
            pairs,
            dialect,
        }
    }
}

/// The nearest word on one side of the match, if no punctuation comes first.
fn nearest_word<'a>(mut toks: impl Iterator<Item = &'a Token>) -> Option<&'a Token> {
    toks.find(|tok| !tok.kind.is_whitespace())
        .filter(|tok| tok.kind.is_word())
}

impl ExprLinter for NounVerbSpelling {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let tok = toks.first()?;
        let word = tok.get_ch(src);
        let (before, after) = ctx.unwrap_or_default();

        let is_any = |tok: Option<&Token>, list: &[&str]| {
            tok.is_some_and(|tok| tok.get_ch(src).eq_any_ignore_ascii_case_str(list))
        };
        let prev = nearest_word(before.iter().rev());
        let next = nearest_word(after.iter());

        let (replacement, part) =
            if let Some(&(noun, _)) = self.pairs.iter().find(|(_, verb)| word.eq_str(verb)) {
                // The tagger can read a verb after a noun subject as part of a compound noun,
                // as in "The firm licenses its patents", so an object after it means a verb.
                let used_as_noun = is_any(prev, BEFORE_NOUN)
                    || (tok.kind.is_upos(UPOS::NOUN)
                        && !is_any(prev, BEFORE_VERB)
                        && !is_any(prev, &["to"])
                        && !is_any(next, OBJECT_START));
                (used_as_noun.then_some(noun)?, "noun")
            } else {
                let &(_, verb) = self.pairs.iter().find(|(noun, _)| word.eq_str(noun))?;
                let used_as_verb = is_any(prev, BEFORE_VERB)
                    || (is_any(prev, &["to"]) && is_any(next, OBJECT_START))
                    // After `to`, only the object test above can tell an infinitive from
                    // "committed to practice".
                    || (tok.kind.is_upos(UPOS::VERB)
                        && !is_any(prev, BEFORE_NOUN)
                        && !is_any(prev, &["to"]));
                (used_as_verb.then_some(verb)?, "verb")
            };

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Spelling,
            suggestions: vec![Suggestion::replace_with_match_case_str(replacement, word)],
            message: format!(
                "In {} English, the {part} is spelled `{replacement}`.",
                self.dialect
            ),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Outside American English, some nouns and verbs that sound alike are spelled differently, such as the noun `licence` and the verb `license`, `practice` and `practise`, or `tyre` and `tire`."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Dialect,
        linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result},
    };

    use super::NounVerbSpelling;

    fn british() -> NounVerbSpelling {
        NounVerbSpelling::new(Dialect::British)
    }

    #[test]
    fn fix_driving_license() {
        assert_suggestion_result(
            "I have a driving license.",
            british(),
            "I have a driving licence.",
        );
    }

    #[test]
    fn fix_her_license() {
        assert_suggestion_result(
            "She renewed her license yesterday.",
            NounVerbSpelling::new(Dialect::Australian),
            "She renewed her licence yesterday.",
        );
    }

    #[test]
    fn fix_software_license() {
        assert_suggestion_result(
            "The software license is MIT.",
            british(),
            "The software licence is MIT.",
        );
    }

    #[test]
    fn fix_license_agreement() {
        assert_suggestion_result(
            "Read the license agreement first.",
            british(),
            "Read the licence agreement first.",
        );
    }

    #[test]
    fn fix_plural_licenses() {
        assert_suggestion_result(
            "Both of their licenses expired.",
            NounVerbSpelling::new(Dialect::Canadian),
            "Both of their licences expired.",
        );
    }

    #[test]
    fn fix_to_licence_the_code() {
        assert_suggestion_result(
            "They want to licence the code.",
            british(),
            "They want to license the code.",
        );
    }

    #[test]
    fn fix_will_licence() {
        assert_suggestion_result(
            "We will licence the software to them.",
            british(),
            "We will license the software to them.",
        );
    }

    #[test]
    fn fix_in_practise() {
        assert_suggestion_result(
            "In practise, it works.",
            british(),
            "In practice, it works.",
        );
    }

    #[test]
    fn fix_best_practise() {
        assert_suggestion_result(
            "Best practise is to wait.",
            british(),
            "Best practice is to wait.",
        );
    }

    #[test]
    fn fix_i_practice() {
        assert_suggestion_result(
            "I practice the piano daily.",
            NounVerbSpelling::new(Dialect::Indian),
            "I practise the piano daily.",
        );
    }

    #[test]
    fn fix_should_practice() {
        assert_suggestion_result(
            "You should practice more often.",
            british(),
            "You should practise more often.",
        );
    }

    #[test]
    fn fix_she_practices_law() {
        assert_suggestion_result(
            "She practices law in Leeds.",
            british(),
            "She practises law in Leeds.",
        );
    }

    #[test]
    fn fix_their_practises() {
        assert_suggestion_result(
            "Their practises are outdated.",
            british(),
            "Their practices are outdated.",
        );
    }

    #[test]
    fn allow_correct_british_nouns() {
        assert_no_lints(
            "She lost her licence, so good practice is to carry a copy.",
            british(),
        );
    }

    #[test]
    fn allow_correct_british_verbs() {
        assert_no_lints(
            "Doctors must be licensed to practise, and we license our code openly.",
            british(),
        );
    }

    #[test]
    fn allow_to_practice_as_noun() {
        assert_no_lints("The firm is committed to practice.", british());
    }

    #[test]
    fn allow_verb_after_noun_subject() {
        assert_no_lints("The firm licenses its patents widely.", british());
    }

    #[test]
    fn allow_watched_her_practise() {
        assert_no_lints("We watched her practise the speech.", british());
    }

    #[test]
    fn fix_new_tire() {
        assert_suggestion_result("I need a new tire.", british(), "I need a new tyre.");
    }

    #[test]
    fn fix_the_tires() {
        assert_suggestion_result(
            "The tires are flat.",
            NounVerbSpelling::new(Dialect::Australian),
            "The tyres are flat.",
        );
    }

    #[test]
    fn fix_tyre_as_verb() {
        assert_suggestion_result(
            "I tyre easily on long walks.",
            british(),
            "I tire easily on long walks.",
        );
    }

    #[test]
    fn allow_tire_as_verb() {
        assert_no_lints(
            "Long walks tire me out, and she tires quickly too.",
            british(),
        );
    }

    #[test]
    fn allow_tire_noun_in_canadian() {
        assert_no_lints(
            "I need a new tire.",
            NounVerbSpelling::new(Dialect::Canadian),
        );
    }

    #[test]
    fn allow_american_spellings_in_american() {
        assert_lint_count(
            "I practice daily, have a driving license and need a new tire.",
            NounVerbSpelling::new(Dialect::American),
            0,
        );
    }
}
