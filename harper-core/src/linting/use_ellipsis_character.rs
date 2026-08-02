use super::{Lint, LintKind, Linter, Suggestion};
use crate::TokenStringExt;

const UNICODE_ELLIPSIS: char = '…';

/// Converts three periods used as an ellipsis into the Unicode ellipsis character.
#[derive(Debug, Default)]
pub struct UseEllipsisCharacter;

impl Linter for UseEllipsisCharacter {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for tok in document.iter_ellipsiss() {
            let tok_content = document.get_span_content(&tok.span);

            if tok_content != ['.', '.', '.'] {
                continue;
            }

            lints.push(Lint {
                span: tok.span,
                lint_kind: LintKind::Formatting,
                suggestions: vec![Suggestion::ReplaceWith(vec![UNICODE_ELLIPSIS])],
                message: "Use the Unicode ellipsis character (…).".to_owned(),
                priority: 31,
            });
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Replaces three-period ellipses with the single Unicode ellipsis character."
    }
}

#[cfg(test)]
mod tests {
    use super::UseEllipsisCharacter;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn corrects_basic_ellipsis() {
        assert_suggestion_result("...", UseEllipsisCharacter, "…");
    }

    #[test]
    fn corrects_sentence_final_ellipsis() {
        assert_suggestion_result("Wait...", UseEllipsisCharacter, "Wait…");
    }

    #[test]
    fn corrects_ellipsis_with_trailing_space() {
        assert_suggestion_result("Wait... now", UseEllipsisCharacter, "Wait… now");
    }

    #[test]
    fn corrects_ellipsis_in_quotes() {
        assert_suggestion_result("\"...\"", UseEllipsisCharacter, "\"…\"");
    }

    #[test]
    fn corrects_ellipsis_after_word() {
        assert_suggestion_result("maybe...", UseEllipsisCharacter, "maybe…");
    }

    #[test]
    fn corrects_ellipsis_before_word() {
        assert_suggestion_result("...maybe", UseEllipsisCharacter, "…maybe");
    }

    #[test]
    fn corrects_multiple_ellipses() {
        assert_suggestion_result("... and ...", UseEllipsisCharacter, "… and …");
    }

    #[test]
    fn corrects_adjacent_to_punctuation() {
        assert_suggestion_result("Wait...!", UseEllipsisCharacter, "Wait…!");
    }

    #[test]
    fn corrects_parenthetical_ellipsis() {
        assert_suggestion_result("(...) ", UseEllipsisCharacter, "(…) ");
    }

    #[test]
    fn allows_unicode_ellipsis() {
        assert_no_lints("…", UseEllipsisCharacter);
    }

    #[test]
    fn allows_unicode_ellipsis_in_sentence() {
        assert_no_lints("Wait…", UseEllipsisCharacter);
    }

    #[test]
    fn allows_four_periods() {
        assert_no_lints("....", UseEllipsisCharacter);
    }

    #[test]
    fn allows_two_periods() {
        assert_no_lints("..", UseEllipsisCharacter);
    }

    #[test]
    fn allows_long_period_run() {
        assert_no_lints(".....", UseEllipsisCharacter);
    }

    #[test]
    fn flags_single_three_period_ellipsis_once() {
        assert_lint_count("Wait... now", UseEllipsisCharacter, 1);
    }
}
