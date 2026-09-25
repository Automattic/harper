//! German language module implementation of LanguageModule trait.

use std::sync::Arc;

use crate::language::german::dialects::GermanDialect;
use crate::language::german::language_detection::GermanDetector;
use crate::language::german::linting::{new_curated_german, weir_rules};
use crate::language::german::parsers::PlainGerman;

use crate::linting::LintGroup;
use crate::parsers::Parser;
use crate::spell::Dictionary;

use crate::language::module::LanguageModule;

/// German language module implementing the LanguageModule trait.
pub struct GermanModule;

impl LanguageModule for GermanModule {
    type Dialect = GermanDialect;
    type Detector = GermanDetector;

    fn default_dialect() -> Self::Dialect {
        GermanDialect::default()
    }

    fn detector() -> Self::Detector {
        GermanDetector
    }

    fn plain_parser() -> impl Parser + 'static {
        PlainGerman
    }

    fn dictionary() -> Arc<dyn Dictionary> {
        // Use compound-aware dictionary for German which provides comprehensive word coverage
        // with lazy compound checking to avoid memory explosion from pre-generating all compounds
        use crate::language::german::spell::compound_aware_german_dictionary;
        compound_aware_german_dictionary()
    }

    fn rust_lint_group(dictionary: Arc<impl Dictionary + 'static>) -> LintGroup {
        use crate::language::german::linting::{
            german_absolute_superlative::GermanAbsoluteSuperlative,
            german_common_typos::GermanCommonTypos, german_filler_words::GermanFillerWords,
            german_fixed_nominalization::GermanFixedNominalization,
            german_genitive_after_nominative_article::GermanGenitiveAfterNominativeArticle,
            german_noun_capitalization::GermanNounCapitalization,
            german_preposition_case::GermanPrepositionCase,
            german_sentence_capitalization::GermanSentenceCapitalization,
            german_spell_check::GermanSpellCheck, german_subordinate_comma::GermanSubordinateComma,
            german_wider_wieder::GermanWiderWieder, german_year_preposition::GermanYearPreposition,
        };

        let mut group = LintGroup::empty();
        group.add(
            "GermanSpellCheck",
            GermanSpellCheck::new(dictionary.clone()),
        );
        group.add(
            "GermanNounCapitalization",
            GermanNounCapitalization::new(dictionary.clone()),
        );
        group.add(
            "GermanSentenceCapitalization",
            GermanSentenceCapitalization::new(dictionary.clone()),
        );
        group.add("GermanFillerWords", GermanFillerWords::default());
        group.add("GermanWiderWieder", GermanWiderWieder);
        group.add("GermanCommonTypos", GermanCommonTypos);
        group.add("GermanAbsoluteSuperlative", GermanAbsoluteSuperlative);
        group.add("GermanFixedNominalization", GermanFixedNominalization);
        group.add("GermanSubordinateComma", GermanSubordinateComma);
        group.add("GermanYearPreposition", GermanYearPreposition);
        group.add("GermanPrepositionCase", GermanPrepositionCase::new());
        group.add(
            "GermanGenitiveAfterNominativeArticle",
            GermanGenitiveAfterNominativeArticle::new(),
        );
        group
    }

    fn weir_lint_group() -> LintGroup {
        weir_rules::lint_group()
    }

    fn curated_lint_group(
        dialect: Self::Dialect,
        dictionary: Arc<impl Dictionary + 'static>,
    ) -> LintGroup {
        new_curated_german(dialect, dictionary)
    }
}
