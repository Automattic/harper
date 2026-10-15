use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{TokenStringExt, document::Document};

/// Adjectives that already denote an absolute, ungradable property, so a
/// superlative of them is a contradiction.
///
/// Deliberately short. Several adjectives that are absolute in principle have
/// an established idiomatic superlative ("in vollster Zufriedenheit",
/// "das Äußerste"), and grading those would be a false positive; only forms
/// that no reference grammar accepts are listed.
const ABSOLUTE_ADJECTIVES: &[&str] = &[
    "einzig",
    "einzigartig",
    "ideal",
    "maximal",
    "minimal",
    "optimal",
];

/// The endings an attributive adjective can carry after the superlative `-st`.
/// The empty string covers the predicative form ("am einzigsten").
const DECLENSION_ENDINGS: &[&str] = &["en", "em", "er", "es", "e", ""];

/// Flags superlatives built on adjectives that cannot be graded.
///
/// "einzigste" is the most common instance by a wide margin: "einzig" means
/// "one and only", so there is nothing for a superlative to compare against.
/// The spell checker cannot catch these, because the superlative is formed
/// productively by the affix rules and so is a perfectly well-formed word.
#[derive(Default)]
pub struct GermanAbsoluteSuperlative;

impl GermanAbsoluteSuperlative {
    /// The positive form of `word`, if `word` is the superlative of an
    /// ungradable adjective.
    fn correction(word: &[char]) -> Option<String> {
        let lower: String = word
            .iter()
            .flat_map(|c| c.to_lowercase())
            .collect::<String>();

        // Longest ending first, so "einzigsten" is read as "einzig" + "st" +
        // "en" rather than "einzigst" + "en".
        for ending in DECLENSION_ENDINGS {
            let Some(stem_with_st) = lower.strip_suffix(ending) else {
                continue;
            };
            let Some(stem) = stem_with_st.strip_suffix("st") else {
                continue;
            };
            if !ABSOLUTE_ADJECTIVES.contains(&stem) {
                continue;
            }

            let mut fixed = String::with_capacity(stem.len() + ending.len());
            fixed.push_str(stem);
            fixed.push_str(ending);

            if word.first().is_some_and(|c| c.is_uppercase()) {
                let mut chars = fixed.chars();
                if let Some(first) = chars.next() {
                    fixed = first.to_uppercase().chain(chars).collect();
                }
            }

            return Some(fixed);
        }

        None
    }
}

impl Linter for GermanAbsoluteSuperlative {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let chars = document.get_span_content(&word.span);
            let Some(fixed) = Self::correction(chars) else {
                continue;
            };

            let original: String = chars.iter().collect();
            lints.push(Lint {
                span: word.span,
                lint_kind: LintKind::Grammar,
                suggestions: vec![Suggestion::ReplaceWith(fixed.chars().collect())],
                message: format!(
                    "»{original}« steigert ein nicht steigerbares Adjektiv. \
                     Gemeint ist »{fixed}«."
                ),
                priority: 24,
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "Findet Superlative von Adjektiven, die sich nicht steigern lassen (»einzigste«)."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanAbsoluteSuperlative;

    fn correction(word: &str) -> Option<String> {
        let chars: Vec<char> = word.chars().collect();
        GermanAbsoluteSuperlative::correction(&chars)
    }

    #[test]
    fn corrects_declined_superlatives() {
        for (wrong, right) in [
            ("einzigste", "einzige"),
            ("einzigsten", "einzigen"),
            ("einzigster", "einziger"),
            ("einzigstes", "einziges"),
            ("einzigstem", "einzigem"),
            ("Einzigste", "Einzige"),
            ("optimalste", "optimale"),
            ("idealsten", "idealen"),
            ("einzigartigste", "einzigartige"),
        ] {
            assert_eq!(correction(wrong).as_deref(), Some(right), "{wrong}");
        }
    }

    #[test]
    fn leaves_the_positive_and_gradable_adjectives_alone() {
        for word in [
            "einzig",
            "einzige",
            "einzigen",
            "einzigartig",
            "optimal",
            "schönste",
            "größten",
            "besten",
            "meiste",
            "vollste",
            "Fenster",
            "Osten",
        ] {
            assert_eq!(correction(word), None, "{word} should be left alone");
        }
    }
}
