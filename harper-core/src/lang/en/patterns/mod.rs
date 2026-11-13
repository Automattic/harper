//! Patterns that match English words.

mod implies_quantity;
mod indefinite_article;
mod inflection_of_be;
mod modal_verb;
mod prepositional_preceder;
mod relative_pronoun;

pub use implies_quantity::ImpliesQuantity;
pub use indefinite_article::IndefiniteArticle;
pub use inflection_of_be::InflectionOfBe;
pub use modal_verb::ModalVerb;
pub use prepositional_preceder::{PrepositionalPrecederPattern, prepositional_preceder};
pub use relative_pronoun::RelativePronoun;

use crate::patterns::{Pattern, SingleTokenPattern, WordSet};
