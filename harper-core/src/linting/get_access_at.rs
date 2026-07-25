use crate::expr::{Expr, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

/// Corrects "get access at <resource>" to "get access to <resource>", while avoiding
/// false positives like "get access at all", "get access at least", URLs, or time expressions.
pub struct GetAccessAt {
    expr: SequenceExpr,
}

impl Default for GetAccessAt {
    fn default() -> Self {
        let verbs = Lrc::new(WordSet::new(&[
            "get",
            "gets",
            "got",
            "getting",
            "gained",
            "gaining",
            "obtain",
            "obtains",
            "obtained",
            "obtaining",
        ]));
        let access = Lrc::new(WordSet::new(&["access"]));
        let at = Lrc::new(WordSet::new(&["at"]));

        let pattern = SequenceExpr::with(verbs)
            .then_whitespace()
            .then(access)
            .then_whitespace()
            .then(at);

        Self { expr: pattern }
    }
}

impl ExprLinter for GetAccessAt {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let at_tok = toks.last()?;
        let at_idx = toks.len() - 1;

        // Look at the tokens following "at" to prevent false positives
        let next_tok = toks.get(at_idx + 1);
        let following_tok = toks.get(at_idx + 2);

        if let Some(next) = next_tok {
            let next_word = next.get_str(src).to_lowercase();

            // False positive checks: "at all", "at least"
            if next_word == "all" || next_word == "least" {
                return None;
            }

            // False positive check: "at this time", "at the moment", "at the level", etc.
            if next_word == "this" || next_word == "that" || next_word == "the" {
                if let Some(following) = following_tok {
                    let fol_word = following.get_str(src).to_lowercase();
                    if fol_word == "time"
                        || fol_word == "moment"
                        || fol_word == "stage"
                        || fol_word == "level"
                    {
                        return None;
                    }
                }
            }

            // False positive check: URLs / Web addresses
            if next_word.starts_with("http://")
                || next_word.starts_with("https://")
                || next_word.contains("www.")
            {
                return None;
            }
        }

        Some(Lint {
            span: at_tok.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::ReplaceWith("to".chars().collect())],
            message: "Did you mean `to` instead of `at` when specifying what is being accessed?"
                .to_owned(),
            priority: 45,
        })
    }

    fn description(&self) -> &str {
        "Flags 'get access at' and suggests replacing 'at' with 'to'."
    }
}

#[cfg(test)]
mod tests {
    use super::GetAccessAt;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn test_get_access_at_cpu() {
        assert_suggestion_result(
            "They really need to get access at some CPU time.",
            GetAccessAt::default(),
            "They really need to get access to some CPU time.",
        );
    }

    #[test]
    fn test_get_access_at_models() {
        assert_suggestion_result(
            "In order to get access at models, send a request.",
            GetAccessAt::default(),
            "In order to get access to models, send a request.",
        );
    }

    #[test]
    fn test_got_access_at_database() {
        assert_suggestion_result(
            "I got access at the server database.",
            GetAccessAt::default(),
            "I got access to the server database.",
        );
    }

    // --- False Positive Tests ---

    #[test]
    fn ignore_access_at_all() {
        assert_lint_count(
            "I didn't manage to get access at all.",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_least() {
        assert_lint_count(
            "Is it possible to get access at least to the file content?",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_this_time() {
        assert_lint_count(
            "The quickest method to get access at this time is to submit a form.",
            GetAccessAt::default(),
            0,
        );
    }

    #[test]
    fn ignore_access_at_url() {
        assert_lint_count(
            "Get access at https://example.com/login",
            GetAccessAt::default(),
            0,
        );
    }
}
