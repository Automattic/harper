//! Entries mined from English quotations must not hide a German typo.
//!
//! `tire` sat in the dictionary as a lower-case noun, left over from a quoted
//! English title. Matching is case-insensitive, so `Tire` — the commonest typo
//! for `Tiere`, and one every school text produces — passed as a word.

#[cfg(test)]
mod tests {
    use crate::Document;
    use crate::language::german::dialects::GermanDialect;
    use crate::language::german::linting::new_curated_german;
    use crate::language::german::parsers::PlainGerman;
    use crate::language::german::spell::curated_german_dictionary;
    use crate::linting::{LintKind, Linter};

    fn misspelled(text: &str) -> Vec<String> {
        let dict = curated_german_dictionary();
        let mut linter = new_curated_german(GermanDialect::Standard, dict.clone());
        let document = Document::new(text, &PlainGerman, &dict);
        linter
            .lint(&document)
            .into_iter()
            .filter(|lint| lint.lint_kind == LintKind::Spelling)
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    #[test]
    fn tire_for_tiere_is_reported() {
        assert_eq!(misspelled("Wir haben viele Tire gesehen."), ["Tire"]);
    }

    #[test]
    fn tiere_is_not_reported() {
        assert!(misspelled("Wir haben viele Tiere gesehen.").is_empty());
    }
}
