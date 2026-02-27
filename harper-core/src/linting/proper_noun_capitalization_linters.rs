use crate::expr::{Expr, ExprMap, FixedPhrase};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::{ExprLinter, LintGroup};
use super::{Lint, LintKind, Suggestion};
use crate::Document;
use crate::linting::expr_linter::Chunk;
use crate::parsers::PlainEnglish;
use crate::{Token, TokenStringExt};

/// A linter that corrects the capitalization of multi-word proper nouns.
/// They are corrected to a "canonical capitalization" provided at construction.
///
/// If you would like to add a proper noun to Harper, see `proper_noun_rules.json`.
pub struct ProperNounCapitalizationLinter {
    pattern_map: ExprMap<Document>,
    description: String,
}

impl ProperNounCapitalizationLinter {
    /// Create a linter that corrects the capitalization of phrases provided.
    pub fn new_strs(
        canonical_versions: impl IntoIterator<Item = impl AsRef<str>>,
        description: impl ToString,
    ) -> Self {
        let mut expr_map = ExprMap::default();

        for can_vers in canonical_versions {
            let doc = Document::new_basic_tokenize(can_vers.as_ref(), &PlainEnglish);

            let expr = FixedPhrase::from_document(&doc);

            expr_map.insert(expr, doc);
        }

        Self {
            pattern_map: expr_map,
            description: description.to_string(),
        }
    }
}

impl ExprLinter for ProperNounCapitalizationLinter {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.pattern_map
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let canonical_case = self.pattern_map.lookup(0, matched_tokens, source).unwrap();

        let mut broken = false;

        for (err_token, correct_token) in matched_tokens.iter().zip(canonical_case.fat_tokens()) {
            let err_chars = err_token.span.get_content(source);
            if err_chars != correct_token.content {
                broken = true;
                break;
            }
        }

        if !broken {
            return None;
        }

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Capitalization,
            suggestions: vec![Suggestion::ReplaceWith(
                canonical_case.get_source().to_vec(),
            )],
            message: self.description.to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        self.description.as_str()
    }
}

#[derive(Serialize, Deserialize)]
struct RuleEntry {
    canonical: Vec<String>,
    description: String,
}

/// For the time being, this panics on invalid JSON.
/// Do not use with user provided JSON.
fn lint_group_from_json(json: &str) -> LintGroup {
    let mut group = LintGroup::empty();

    let rules: HashMap<String, RuleEntry> = serde_json::from_str(json).unwrap();

    for (key, rule) in rules.into_iter() {
        group.add_chunk_expr_linter(
            key,
            Box::new(ProperNounCapitalizationLinter::new_strs(
                rule.canonical,
                rule.description,
            )),
        );
    }

    group.set_all_rules_to(Some(true));

    group
}

pub fn lint_group() -> LintGroup {
    lint_group_from_json(include_str!("../../proper_noun_rules.json"))
}

#[cfg(test)]
mod tests {
    use super::lint_group;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn americas_lowercase() {
        assert_suggestion_result("south america", lint_group(), "South America");
        assert_suggestion_result("north america", lint_group(), "North America");
    }

    #[test]
    fn americas_uppercase() {
        assert_suggestion_result("SOUTH AMERICA", lint_group(), "South America");
        assert_suggestion_result("NORTH AMERICA", lint_group(), "North America");
    }

    #[test]
    fn americas_allow_correct() {
        assert_lint_count("South America", lint_group(), 0);
        assert_lint_count("North America", lint_group(), 0);
    }

    #[test]
    fn issue_798() {
        assert_suggestion_result(
            "The United states is a big country.",
            lint_group(),
            "The United States is a big country.",
        );
    }

    #[test]
    fn united_nations_uppercase() {
        assert_suggestion_result("UNITED NATIONS", lint_group(), "United Nations");
    }

    #[test]
    fn united_arab_emirates_lowercase() {
        assert_suggestion_result("UNITED ARAB EMIRATES", lint_group(), "United Arab Emirates");
    }

    #[test]
    fn united_nations_allow_correct() {
        assert_lint_count("United Nations", lint_group(), 0);
    }

    #[test]
    fn meta_allow_correct() {
        assert_lint_count("Meta Quest", lint_group(), 0);
    }

    #[test]
    fn microsoft_lowercase() {
        assert_suggestion_result(
            "microsoft visual studio",
            lint_group(),
            "Microsoft Visual Studio",
        );
    }

    #[test]
    fn microsoft_first_word_is_correct() {
        assert_suggestion_result(
            "Microsoft visual studio",
            lint_group(),
            "Microsoft Visual Studio",
        );
    }

    #[test]
    fn test_atlantic_ocean_lowercase() {
        assert_suggestion_result("atlantic ocean", lint_group(), "Atlantic Ocean");
    }

    #[test]
    fn test_pacific_ocean_lowercase() {
        assert_suggestion_result("pacific ocean", lint_group(), "Pacific Ocean");
    }

    #[test]
    fn test_indian_ocean_lowercase() {
        assert_suggestion_result("indian ocean", lint_group(), "Indian Ocean");
    }

    #[test]
    fn test_southern_ocean_lowercase() {
        assert_suggestion_result("southern ocean", lint_group(), "Southern Ocean");
    }

    #[test]
    fn test_arctic_ocean_lowercase() {
        assert_suggestion_result("arctic ocean", lint_group(), "Arctic Ocean");
    }

    #[test]
    fn test_mediterranean_sea_lowercase() {
        assert_suggestion_result("mediterranean sea", lint_group(), "Mediterranean Sea");
    }

    #[test]
    fn test_caribbean_sea_lowercase() {
        assert_suggestion_result("caribbean sea", lint_group(), "Caribbean Sea");
    }

    #[test]
    fn test_south_china_sea_lowercase() {
        assert_suggestion_result("south china sea", lint_group(), "South China Sea");
    }

    #[test]
    fn test_atlantic_ocean_correct() {
        assert_lint_count("Atlantic Ocean", lint_group(), 0);
    }

    #[test]
    fn test_pacific_ocean_correct() {
        assert_lint_count("Pacific Ocean", lint_group(), 0);
    }

    #[test]
    fn test_indian_ocean_correct() {
        assert_lint_count("Indian Ocean", lint_group(), 0);
    }

    #[test]
    fn test_mediterranean_sea_correct() {
        assert_lint_count("Mediterranean Sea", lint_group(), 0);
    }

    #[test]
    fn test_south_china_sea_correct() {
        assert_lint_count("South China Sea", lint_group(), 0);
    }

    #[test]
    fn day_one_in_sentence() {
        assert_suggestion_result(
            "I love day one. It is the best journaling app.",
            lint_group(),
            "I love Day One. It is the best journaling app.",
        );
    }

    #[test]
    fn gilded_age_in_sentence() {
        assert_suggestion_result(
            "Mani-Chess Destiny is a JavaScript based computer game built off of chess, but in the style of the gilded age.",
            lint_group(),
            "Mani-Chess Destiny is a JavaScript based computer game built off of chess, but in the style of the Gilded Age.",
        );
    }
}
