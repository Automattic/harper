use crate::{
    Document,
    linting::{Linter, spell_check::SpellCheck},
    spell::Dictionary,
};

pub struct SpellCheckWithUsernames<T: Dictionary> {
    spell_check: SpellCheck<T>,
}

impl<T: Dictionary> SpellCheckWithUsernames<T> {
    pub fn new(dictionary: T, dialect: crate::Dialect) -> Self {
        Self {
            spell_check: SpellCheck::new(dictionary, dialect),
        }
    }
}

impl<T: Dictionary> Linter for SpellCheckWithUsernames<T> {
    fn lint(&mut self, document: &Document) -> Vec<super::Lint> {
        // Get all spelling lints from the underlying SpellCheck
        let all_lints = self.spell_check.lint(document);

        // Filter out username mentions by checking for preceding @ symbol
        all_lints
            .into_iter()
            .filter(|lint| {
                !document
                    .token_indices_intersecting(lint.span)
                    .first()
                    .is_some_and(|&first_to_idx| {
                        document
                            .get_token_offset(first_to_idx, -1)
                            .is_some_and(|prev_tok| prev_tok.kind.is_at())
                    })
            })
            .collect()
    }

    fn description(&self) -> &'static str {
        "Looks and provides corrections for misspelled words, ignoring usernames."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        linting::tests::{assert_lint_count, assert_no_lints},
        spell::FstDictionary,
    };

    use super::SpellCheckWithUsernames;

    #[test]
    fn dont_flag_username_mention_2403() {
        assert_no_lints(
            "@asafm this looks great",
            SpellCheckWithUsernames::new(FstDictionary::curated(), crate::Dialect::American),
        );
    }

    #[test]
    fn flag_username_without_mention_2403() {
        assert_lint_count(
            "asafm this looks great",
            SpellCheckWithUsernames::new(FstDictionary::curated(), crate::Dialect::American),
            1,
        );
    }

    #[test]
    fn dont_flag_email_addresses() {
        assert_no_lints(
            "contact me at asafm@example.com",
            SpellCheckWithUsernames::new(FstDictionary::curated(), crate::Dialect::American),
        );
    }
}
