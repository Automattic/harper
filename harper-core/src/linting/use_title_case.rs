use crate::{Document, TokenStringExt, spell::Dictionary, title_case::try_make_title_case};

use super::{Lint, LintKind, Linter, Suggestion};

pub struct UseTitleCase<D: Dictionary + 'static> {
    dict: D,
}

impl<D: Dictionary + 'static> UseTitleCase<D> {
    pub fn new(dict: D) -> Self {
        Self { dict }
    }
}

impl<D: Dictionary + 'static> Linter for UseTitleCase<D> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for heading in document.iter_headings() {
            let Some(span) = heading.span() else {
                continue;
            };

            if let Some(title_case) =
                try_make_title_case(heading, document.get_source(), self.dict.get_word_map())
            {
                lints.push(Lint {
                    span,
                    lint_kind: LintKind::Capitalization,
                    suggestions: vec![Suggestion::ReplaceWith(title_case)],
                    message: "Try to use title case in headings.".to_owned(),
                    priority: 127,
                });
            }
        }

        lints
    }

    fn description(&self) -> &str {
        "Prompts you to use title case in relevant headings."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};
    use crate::spell::FstDictionary;

    use super::UseTitleCase;

    #[test]
    fn simple_correction() {
        assert_suggestion_result(
            "# This is a title",
            UseTitleCase::new(FstDictionary::curated()),
            "# This Is a Title",
        );
    }

    #[test]
    fn double_correction() {
        assert_suggestion_result(
            "# This is a title\n\n## This is a subtitle",
            UseTitleCase::new(FstDictionary::curated()),
            "# This Is a Title\n\n## This Is a Subtitle",
        );
    }

    #[test]
    fn doesnt_lowercase_this_in_github_template_title() {
        assert_no_lints(
            "# How Has This Been Tested?",
            UseTitleCase::new(FstDictionary::curated()),
        );
    }

    #[test]
    fn shoud_uppercase_possessive_determiners() {
        assert_suggestion_result(
            "# my/our/your/his/her/its/their",
            UseTitleCase::new(FstDictionary::curated()),
            "# My/Our/Your/His/Her/Its/Their",
        );
    }
}
