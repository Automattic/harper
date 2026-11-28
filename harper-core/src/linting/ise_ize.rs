use std::sync::Arc;

use super::proper_noun_capitalization_linters::proper_nouns_expr;
use crate::{
    CharStringExt, Dialect, Document, TokenStringExt,
    expr::LongestMatchOf,
    linting::{Lint, LintKind, Linter, Suggestion},
    remove_lints_overlapping_expr,
    spell::{Dictionary, FstDictionary},
};

/// Endings that can follow `-iz`/`-is` or `-yz`/`-ys`, longest first.
const ENDINGS: &[&str] = &[
    "ationally",
    "ational",
    "ability",
    "ations",
    "ation",
    "ative",
    "able",
    "ing",
    "ers",
    "er",
    "es",
    "ed",
    "e",
];

/// Stems where `-ise` or `-ize` isn't the suffix, as in `prise`/`prize`, `size`, `reprise` and
/// `apprise`/`apprize`.
const NOT_SUFFIX_STEMS: &[&str] = &["pr", "s", "repr", "appr"];

/// Longer words are left alone, so the lookup never needs to allocate.
const MAX_WORD_LEN: usize = 48;

#[derive(Clone, Copy, PartialEq)]
enum Prefer {
    /// `-ise` and `-yse`, as in `organise` and `analyse`.
    Ise,
    /// Oxford spelling: `-ize` but `-yse`, as in `organize` and `analyse`.
    Ize,
}

/// The preferred spelling of `word`, if it differs and is in the dictionary.
fn preferred_spelling(word: &[char], prefer: Prefer, dict: &FstDictionary) -> Option<Vec<char>> {
    if word.len() > MAX_WORD_LEN {
        return None;
    }

    ENDINGS.iter().find_map(|ending| {
        // Index of the `z` or `s` that may change.
        let pos = word.len().checked_sub(ending.len() + 1)?;
        if pos < 2 || !word[pos + 1..].eq_str(ending) {
            return None;
        }

        let replacement = match (
            prefer,
            word[pos - 1].to_ascii_lowercase(),
            word[pos].to_ascii_lowercase(),
        ) {
            (Prefer::Ise, 'i' | 'y', 'z') | (Prefer::Ize, 'y', 'z') => 's',
            (Prefer::Ize, 'i', 's') => 'z',
            _ => return None,
        };

        if word[..pos - 1].eq_any_ignore_ascii_case_str(NOT_SUFFIX_STEMS) {
            return None;
        }

        let mut buf = ['\0'; MAX_WORD_LEN];
        let alt = &mut buf[..word.len()];
        alt.copy_from_slice(word);
        alt[pos] = if word[pos].is_uppercase() {
            replacement.to_ascii_uppercase()
        } else {
            replacement
        };

        dict.contains_word(alt).then(|| alt.to_vec())
    })
}

struct IseIze {
    prefer: Prefer,
    dialect: Dialect,
    dict: Arc<FstDictionary>,
    proper_nouns: LongestMatchOf,
}

impl IseIze {
    fn new(prefer: Prefer, dialect: Dialect) -> Self {
        Self {
            prefer,
            dialect,
            dict: FstDictionary::curated(),
            proper_nouns: proper_nouns_expr(),
        }
    }

    fn lint(&self, document: &Document) -> Vec<Lint> {
        // American and Canadian English use `-ize`, so there's no preference to apply.
        if !matches!(
            self.dialect,
            Dialect::British | Dialect::Australian | Dialect::Indian
        ) {
            return Vec::new();
        }

        let src = document.get_source();
        let mut lints: Vec<Lint> = document
            .iter_words()
            .filter_map(|tok| {
                let alt = preferred_spelling(tok.get_ch(src), self.prefer, &self.dict)?;
                let alt_str: String = alt.iter().collect();
                let message = match self.prefer {
                    Prefer::Ise => {
                        format!(
                            "In {} English, the usual spelling is `{alt_str}`.",
                            self.dialect
                        )
                    }
                    Prefer::Ize => format!("Oxford spelling uses `{alt_str}`."),
                };

                Some(Lint {
                    span: tok.span,
                    lint_kind: LintKind::Spelling,
                    suggestions: vec![Suggestion::ReplaceWith(alt)],
                    message,
                    ..Default::default()
                })
            })
            .collect();

        // Names keep their own spelling, such as `World Health Organization`.
        remove_lints_overlapping_expr(&self.proper_nouns, document, &mut lints);

        lints
    }
}

/// Suggests `-ise` and `-yse` spellings in British, Australian and Indian English.
pub struct PreferIse(IseIze);

impl PreferIse {
    pub fn new(dialect: Dialect) -> Self {
        Self(IseIze::new(Prefer::Ise, dialect))
    }
}

impl Linter for PreferIse {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        self.0.lint(document)
    }

    fn description(&self) -> &str {
        "In British, Australian and Indian English, suggests `-ise` and `-yse` spellings, such as `organise` and `analyse`. Has no effect in American or Canadian English."
    }
}

/// Suggests Oxford spelling, `-ize` but `-yse`, in British, Australian and Indian English.
pub struct PreferIze(IseIze);

impl PreferIze {
    pub fn new(dialect: Dialect) -> Self {
        Self(IseIze::new(Prefer::Ize, dialect))
    }
}

impl Linter for PreferIze {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        self.0.lint(document)
    }

    fn description(&self) -> &str {
        "In British, Australian and Indian English, suggests Oxford spelling: `-ize` spellings such as `organize`, but `-yse` spellings such as `analyse`. Has no effect in American or Canadian English."
    }
}

#[cfg(test)]
mod tests {
    use super::{PreferIse, PreferIze};
    use crate::{
        Dialect,
        linting::{
            LintGroup,
            tests::{assert_lint_count, assert_no_lints, assert_suggestion_result},
        },
        spell::FstDictionary,
    };

    fn ise() -> PreferIse {
        PreferIse::new(Dialect::British)
    }

    fn ize() -> PreferIze {
        PreferIze::new(Dialect::British)
    }

    #[test]
    fn ise_fixes_organize() {
        assert_suggestion_result("We organize events.", ise(), "We organise events.");
    }

    #[test]
    fn ise_keeps_capital_letter() {
        assert_suggestion_result(
            "Organization matters here.",
            ise(),
            "Organisation matters here.",
        );
    }

    #[test]
    fn ise_fixes_analyze() {
        assert_suggestion_result("We analyze the data.", ise(), "We analyse the data.");
    }

    #[test]
    fn ise_fixes_inflected_form() {
        assert_suggestion_result(
            "They are realizing it now.",
            ise(),
            "They are realising it now.",
        );
    }

    #[test]
    fn ise_fixes_long_word() {
        assert_suggestion_result(
            "Internationalization takes time.",
            ise(),
            "Internationalisation takes time.",
        );
    }

    #[test]
    fn ise_fixes_in_australian() {
        assert_suggestion_result(
            "Please capitalize the title.",
            PreferIse::new(Dialect::Australian),
            "Please capitalise the title.",
        );
    }

    #[test]
    fn ise_allows_ise_spellings() {
        assert_no_lints(
            "We organise events, analyse the data and realise the goal.",
            ise(),
        );
    }

    #[test]
    fn ise_allows_words_where_ize_is_not_a_suffix() {
        assert_no_lints(
            "Police seize the prized boat as it begins to capsize, so we resize the photo.",
            ise(),
        );
    }

    #[test]
    fn ise_allows_names() {
        assert_no_lints("The World Health Organization replied.", ise());
    }

    #[test]
    fn ise_does_nothing_in_american() {
        assert_no_lints(
            "We organize events and analyze the data.",
            PreferIse::new(Dialect::American),
        );
    }

    #[test]
    fn ise_does_nothing_in_canadian() {
        assert_no_lints(
            "We organize events and analyze the data.",
            PreferIse::new(Dialect::Canadian),
        );
    }

    #[test]
    fn ize_fixes_organise() {
        assert_suggestion_result("We organise events.", ize(), "We organize events.");
    }

    #[test]
    fn ize_keeps_yse() {
        assert_suggestion_result("We analyze the data.", ize(), "We analyse the data.");
    }

    #[test]
    fn ize_allows_oxford_spellings() {
        assert_no_lints("We organize events and analyse the data.", ize());
    }

    #[test]
    fn ize_allows_words_where_ise_is_not_a_suffix() {
        assert_no_lints(
            "To our surprise, they apprised us before we could prise the lid open, then reprised the song.",
            ize(),
        );
    }

    #[test]
    fn ize_does_nothing_in_american() {
        assert_no_lints(
            "We organise events and analyze the data.",
            PreferIze::new(Dialect::American),
        );
    }

    #[test]
    fn prefers_ise_by_default_in_british() {
        assert_lint_count(
            "We organize events.",
            LintGroup::new_curated(FstDictionary::curated(), Dialect::British),
            1,
        );
    }

    #[test]
    fn no_preference_by_default_in_american() {
        assert_lint_count(
            "We organize events.",
            LintGroup::new_curated(FstDictionary::curated(), Dialect::American),
            0,
        );
    }
}
