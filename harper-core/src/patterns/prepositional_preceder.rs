use lazy_static::lazy_static;

use super::{SingleTokenPattern, WordSet};
use crate::Token;

/// Matches adjectives that routinely introduce a `… to …` prepositional phrase, such as
/// `accustomed`, `prone`, or `used`.
///
/// Several `ToTwoToo` branches need this guard so they only flag cases where `to` is meant as
/// `too`, not when it participates in idioms like `accustomed to precision`.
#[derive(Debug, Clone)]
pub struct PrepositionalPrecederPattern {
    word_set: WordSet,
}

impl Default for PrepositionalPrecederPattern {
    fn default() -> Self {
        Self {
            word_set: WordSet::new(&[
                "accustomed",
                "addicted",
                "adjacent",
                "allergic",
                "attached",
                "attuned",
                "committed",
                "connected",
                "dedicated",
                "devoted",
                "immune",
                "oblivious",
                "opposed",
                "partial",
                "prone",
                "receptive",
                "related",
                "resistant",
                "sensitive",
                "subject",
                "susceptible",
                "used",
            ]),
        }
    }
}

impl SingleTokenPattern for PrepositionalPrecederPattern {
    fn matches_token(&self, token: &Token, source: &[char]) -> bool {
        self.word_set.matches_token(token, source)
    }
}

lazy_static! {
    static ref PREPOSITIONAL_PRECEDER_PATTERN: PrepositionalPrecederPattern =
        PrepositionalPrecederPattern::default();
}

/// Shared accessor for the lazily-initialized [`PrepositionalPrecederPattern`].
pub fn prepositional_preceder() -> &'static PrepositionalPrecederPattern {
    &PREPOSITIONAL_PRECEDER_PATTERN
}
