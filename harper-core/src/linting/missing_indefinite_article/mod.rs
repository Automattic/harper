mod approximate_quantities;
mod bare_uses;
mod count_senses;
mod phrase;
mod prepositional_objects;
mod sentence_subjects;
mod such_phrases;
mod verb_complements;

use crate::{
    Dialect, Document,
    linting::{Lint, LintGroup, Linter},
};
use approximate_quantities::ApproximateQuantities;
use prepositional_objects::PrepositionalObjects;
use sentence_subjects::SentenceSubjects;
use such_phrases::SuchPhrases;
use verb_complements::VerbComplements;

/// Combines context-specific article checks behind one public rule setting.
/// Children are private, explicitly enabled, and share the selected dialect.
pub struct MissingIndefiniteArticle {
    group: LintGroup,
}

impl MissingIndefiniteArticle {
    /// Builds the private group without inheriting curated public-rule defaults.
    pub fn new(dialect: Dialect) -> Self {
        let mut group = LintGroup::empty();
        group.add("VerbComplements", VerbComplements(dialect));
        group.add("PrepositionalObjects", PrepositionalObjects(dialect));
        group.add("SentenceSubjects", SentenceSubjects(dialect));
        group.add("SuchPhrases", SuchPhrases(dialect));
        group.add("ApproximateQuantities", ApproximateQuantities(dialect));
        group.set_all_rules_to(Some(true));
        Self { group }
    }
}

impl Linter for MissingIndefiniteArticle {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = self.group.lint(document);
        lints.sort_by_key(|lint| (lint.span.start, lint.span.end));
        // Empty insertion spans do not overlap geometrically. Remove identical
        // insertions explicitly rather than relying on overlap removal alone.
        lints.dedup_by(|a, b| a.span == b.span && a.suggestions == b.suggestions);
        // A child can see a participial modifier as the governor of another
        // candidate. Keep the earlier insertion for the complete phrase only.
        let tokens = document.get_tokens();
        let mut covered_until = 0;
        lints.retain(|lint| {
            if lint.span.start < covered_until {
                return false;
            }
            covered_until = tokens
                .binary_search_by_key(&lint.span.start, |token| token.span.start)
                .ok()
                .and_then(|index| phrase::phrase(tokens, index))
                .map_or(lint.span.end, |phrase| phrase.head.span.end);
            true
        });
        lints
    }
    fn description(&self) -> &'static str {
        "Suggests missing `a` or `an` before singular count noun phrases in subjects, verb complements, and prepositional objects. Skips mass nouns, plurals, proper nouns, and common bare roles."
    }
}

#[cfg(test)]
mod corpus_regressions;
#[cfg(test)]
mod tests;
