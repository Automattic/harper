use std::sync::Arc;

use crate::language::german::parsers::PlainGerman;
use crate::language::german::spell::german_dictionary;
use crate::language::portuguese::parsers::PlainPortuguese;
use crate::language::portuguese::spell::portuguese_dictionary;
use crate::linting::LintGroup;
use crate::parsers::{Markdown, MarkdownOptions, OrgMode, Parser, PlainEnglish};
use crate::spell::{Dictionary, FstDictionary};
use crate::{Dialect, languages::LanguageFamily};

const LANGUAGE_NEUTRAL_RULES: &[&str] = &[
    "CommaFixes",
    "CorrectNumberSuffix",
    "CurrencyPlacement",
    "Dashes",
    "DotInitialisms",
    "EllipsisLength",
    "ExpandMemoryShorthands",
    "ExpandTimeShorthands",
    "LongSentences",
    "NoFrenchSpaces",
    "NumberSuffixCapitalization",
    "NumericRangeEnDash",
    "QuoteSpacing",
    "RepeatedWords",
    "SentenceCapitalization",
    "Spaces",
    "UnclosedQuotes",
    "UseEllipsisCharacter",
];

const GERMAN_RULES: &[&str] = &[
    "GermanFillerWords",
    "GermanNounCapitalization",
    "GermanSentenceCapitalization",
    "GermanSpellCheck",
];

const PORTUGUESE_RULES: &[&str] = &["PortugueseSpellCheck"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProseLanguage {
    English,
    German,
    Portuguese,
}

pub fn prose_language_for_dialect(dialect: Dialect) -> ProseLanguage {
    if dialect.is_german() {
        ProseLanguage::German
    } else if dialect.is_portuguese() {
        ProseLanguage::Portuguese
    } else {
        ProseLanguage::English
    }
}

pub fn dictionary_for_language(language: LanguageFamily) -> Arc<FstDictionary> {
    match language {
        LanguageFamily::English => FstDictionary::curated(),
        LanguageFamily::German => german_dictionary(),
        LanguageFamily::Portuguese => portuguese_dictionary(),
    }
}

pub fn dictionary_for_dialect(dialect: Dialect) -> Arc<FstDictionary> {
    dictionary_for_language(dialect.language_family())
}

pub fn parser_for_prose(
    language_id: &str,
    dialect: Dialect,
    markdown_options: MarkdownOptions,
) -> Option<Box<dyn Parser>> {
    match (language_id, prose_language_for_dialect(dialect)) {
        ("mail", ProseLanguage::German) => Some(Box::new(PlainGerman)),
        ("mail", ProseLanguage::Portuguese) => Some(Box::new(PlainPortuguese)),
        ("mail", ProseLanguage::English) => Some(Box::new(PlainEnglish)),

        ("markdown" | "quarto", ProseLanguage::German) => {
            Some(Box::new(Markdown::new_german(markdown_options)))
        }
        ("markdown" | "quarto", _) => Some(Box::new(Markdown::new(markdown_options))),

        ("org", ProseLanguage::German) => Some(Box::new(OrgMode::new_german())),
        ("org", _) => Some(Box::new(OrgMode::default())),

        ("plaintext" | "text", ProseLanguage::German) => Some(Box::new(PlainGerman)),
        ("plaintext" | "text", ProseLanguage::Portuguese) => Some(Box::new(PlainPortuguese)),
        ("plaintext" | "text", ProseLanguage::English) => Some(Box::new(PlainEnglish)),
        _ => None,
    }
}

pub fn add_language_specific_linters(
    out: &mut LintGroup,
    dialect: Dialect,
    dictionary: Arc<impl Dictionary + 'static>,
) {
    if dialect.is_german() {
        use crate::language::german::linting::german_noun_capitalization::GermanNounCapitalization;
        use crate::language::german::linting::german_sentence_capitalization::GermanSentenceCapitalization;
        use crate::language::german::linting::german_spell_check::GermanSpellCheck;

        out.add(
            "GermanSpellCheck",
            GermanSpellCheck::new(dictionary.clone()),
        );
        out.config.set_rule_enabled("GermanSpellCheck", true);
        out.add(
            "GermanNounCapitalization",
            GermanNounCapitalization::new(dictionary.clone()),
        );
        out.config
            .set_rule_enabled("GermanNounCapitalization", true);
        out.add(
            "GermanSentenceCapitalization",
            GermanSentenceCapitalization::new(dictionary.clone()),
        );
        out.config
            .set_rule_enabled("GermanSentenceCapitalization", true);
    }

    if dialect.is_portuguese() {
        use crate::language::portuguese::linting::portuguese_spell_check::PortugueseSpellCheck;

        out.add(
            "PortugueseSpellCheck",
            PortugueseSpellCheck::new(dictionary.clone()),
        );
        out.config.set_rule_enabled("PortugueseSpellCheck", true);
    }
}

pub fn rule_default_enabled(rule_name: &str, dialect: Dialect, default: bool) -> bool {
    if !default {
        return false;
    }

    if dialect.is_english() {
        return true;
    }

    if LANGUAGE_NEUTRAL_RULES.contains(&rule_name) {
        return true;
    }

    if dialect.is_german() {
        return GERMAN_RULES.contains(&rule_name);
    }

    if dialect.is_portuguese() {
        return PORTUGUESE_RULES.contains(&rule_name);
    }

    false
}
