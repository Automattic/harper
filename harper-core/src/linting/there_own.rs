use crate::char_string::CharStringExt;
use crate::linting::expr_linter::{Chunk, preceded_by_word};
use crate::{
    Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

/// Flags "there own", "they're own", and "theyre own" and suggests "their own".
pub struct ThereOwn {
    expr: SequenceExpr,
}

impl Default for ThereOwn {
    fn default() -> Self {
        let expr = SequenceExpr::word_set(&["there", "they're", "theyre"])
            .t_ws()
            .t_aco("own");

        Self { expr }
    }
}

impl ExprLinter for ThereOwn {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let offender = matched_tokens.first()?;
        let template = offender.span.get_content(source);

        // When "there" is preceded by a noun or pronoun, it functions as an
        // adverb ("in that place") and "own" is the main verb:
        //   "people there own nice cars"
        //   "companies there own the property"
        // This does not apply to "they're" or "theyre" variants, which are
        // always possessive errors before "own".
        if template.eq_str("there") && preceded_by_word(ctx, |pw| pw.kind.is_nominal()) {
            return None;
        }

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str("their", template)],
            message: "Did you mean the possessive `their`?".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `there own`, `they're own`, and `theyre own` to `their own`."
    }
}

#[cfg(test)]
mod tests {
    use super::ThereOwn;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_there_own() {
        assert_suggestion_result(
            "Users can split data on there own topics.",
            ThereOwn::default(),
            "Users can split data on their own topics.",
        );
    }

    #[test]
    fn corrects_theyre_own() {
        assert_suggestion_result(
            "Everybody has they're own preferences.",
            ThereOwn::default(),
            "Everybody has their own preferences.",
        );
    }

    #[test]
    fn corrects_theyre_no_apostrophe() {
        assert_suggestion_result(
            "It would be helpful for people building theyre own rockets.",
            ThereOwn::default(),
            "It would be helpful for people building their own rockets.",
        );
    }

    #[test]
    fn preserves_capitalization() {
        assert_suggestion_result(
            "There own connection pool must be configured.",
            ThereOwn::default(),
            "Their own connection pool must be configured.",
        );
    }

    #[test]
    fn does_not_flag_correct_their_own() {
        assert_lint_count("They manage their own servers.", ThereOwn::default(), 0);
    }

    #[test]
    fn does_not_flag_there_without_own() {
        assert_lint_count(
            "Put the chairs over there by the window.",
            ThereOwn::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_verb_own_after_noun() {
        assert_lint_count("People there own nice cars.", ThereOwn::default(), 0);
    }

    #[test]
    fn does_not_flag_verb_own_with_determiner() {
        assert_lint_count("Companies there own the property.", ThereOwn::default(), 0);
    }

    // Examples from issue #3276 — feature request author's positive cases.

    #[test]
    fn issue_3276_customise_default_lvl() {
        assert_suggestion_result(
            "Provide user the option in setting to customise there own default Effort lvl.",
            ThereOwn::default(),
            "Provide user the option in setting to customise their own default Effort lvl.",
        );
    }

    #[test]
    fn issue_3276_modules_c_files() {
        assert_suggestion_result(
            "Allow for modules to provide there own c files.",
            ThereOwn::default(),
            "Allow for modules to provide their own c files.",
        );
    }

    #[test]
    fn issue_3276_scripts_create_connection() {
        assert_suggestion_result(
            "I have a number of scripts that all create there own connection.",
            ThereOwn::default(),
            "I have a number of scripts that all create their own connection.",
        );
    }

    #[test]
    fn issue_3276_silent_appstore_updates() {
        assert_suggestion_result(
            "allowing 3rd party appstores to silently update apps installed by theyre own.",
            ThereOwn::default(),
            "allowing 3rd party appstores to silently update apps installed by their own.",
        );
    }

    #[test]
    fn issue_3276_theyre_own_command() {
        assert_suggestion_result(
            "But because they are they're own command they have their own runtime.",
            ThereOwn::default(),
            "But because they are their own command they have their own runtime.",
        );
    }

    #[test]
    fn issue_3276_theyre_own_library() {
        assert_suggestion_result(
            "Components I'm working on that will eventually become they're own library.",
            ThereOwn::default(),
            "Components I'm working on that will eventually become their own library.",
        );
    }

    // Issue #3276's explicit false-positive example.

    #[test]
    fn issue_3276_false_positive_expensive_cars() {
        assert_lint_count("People there own expensive cars.", ThereOwn::default(), 0);
    }

    // Sentence-initial "There own" stays a true positive (no preceding word, so
    // the adverbial-there guard correctly does not trigger).
    #[test]
    fn sentence_initial_there_own_still_flagged() {
        assert_suggestion_result(
            "There own employees disagreed with the policy.",
            ThereOwn::default(),
            "Their own employees disagreed with the policy.",
        );
    }
}
