mod no_oxford_comma;
mod oxford_comma;

pub use no_oxford_comma::NoOxfordComma;
pub use oxford_comma::OxfordComma;

#[cfg(test)]
mod tests {
    use crate::{EnglishDialect, linting::tests::assert_lint_count, spell::FstDictionary};

    #[test]
    fn by_default_one_must_be_enabled_and_one_must_be_disabled() {
        assert_lint_count(
            "One, two and three. But four, five, and six.",
            crate::linting::LintGroup::new_curated(
                FstDictionary::curated(),
                EnglishDialect::American,
            ),
            1,
        );
    }
}
