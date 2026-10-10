use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_word},
    },
};

const SPATIAL_ADVERBS: &[&str] = &["down", "out", "over", "right", "up"];
const VERBS_OF_EXISTENCE_OR_POSTURE: &[&str] = &["are", "be", "get", "stood", "was"];

pub struct Therein {
    expr: SequenceExpr,
}

impl Default for Therein {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_word()
                .t_ws()
                .t_aco("there")
                .t_ws_h()
                .t_aco("in"),
        }
    }
}

impl ExprLinter for Therein {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // Don't continue if the previous word is an object pronoun, spatial adverb,
        // or verb of existence/posture
        let previous_word = matched_tokens.first()?;
        if previous_word.kind.is_object_pronoun() {
            return None;
        }
        let previous_word = previous_word.get_ch(source);
        if previous_word.eq_any_ignore_ascii_case_str(SPATIAL_ADVERBS)
            || previous_word.eq_any_ignore_ascii_case_str(VERBS_OF_EXISTENCE_OR_POSTURE)
        {
            return None;
        }

        // Don't continue if the next word is a determiner or a likely year
        if followed_by_word(context, |t| {
            t.kind.is_determiner()
                || (t.kind.is_cardinal_number() && {
                    let chars = t.get_ch(source);
                    if chars.len() == 4 && chars.iter().all(|c| c.is_ascii_digit()) {
                        let year_str: String = chars.iter().collect();
                        year_str
                            .parse::<i32>()
                            .is_ok_and(|y| (1900..=2100).contains(&y))
                    } else {
                        false
                    }
                })
        }) {
            return None;
        }

        let span = matched_tokens.get(2..=4)?.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "therein",
                span.get_content(source),
            )],
            message: "Use the closed compound 'therein' here.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags occurrences of `there in` or `there-in` that should be the closed compound word `therein`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::Therein;

    #[test]
    fn fix_there_in_stands_space() {
        assert_suggestion_result(
            "WELL THERE IN STANDS THE ISSUE RIGHT THERE BRIGHT SPARK.",
            Therein::default(),
            "WELL THEREIN STANDS THE ISSUE RIGHT THERE BRIGHT SPARK.",
        );
    }

    #[test]
    fn contained_there_hyphen_in() {
        assert_suggestion_result(
            "contained there-in",
            Therein::default(),
            "contained therein",
        );
    }

    #[test]
    fn fix_there_in_is_hyphen() {
        assert_suggestion_result(
            "name and password contained there-in is only for accessing the PostgreSQL instance inside the created ephemeral container.",
            Therein::default(),
            "name and password contained therein is only for accessing the PostgreSQL instance inside the created ephemeral container.",
        );
    }

    #[test]
    fn dont_flag_there_in_a() {
        assert_no_lints(
            "It might have started happening after I tried to bump the ancient node version used there in a specific PR, but affecting unrelated ones.",
            Therein::default(),
        );
    }

    #[test]
    fn dont_flag_out_there_in_an() {
        assert_no_lints(
            "But if it actually became a language out there in an ecosystem, I worry that you might have people start abusing that.",
            Therein::default(),
        );
    }

    #[test]
    fn dont_flag_there_in_its() {
        assert_no_lints(
            "It was a rich cream color, bright with nickel, swollen here and there in its monstrous length with triumphant hat-boxes",
            Therein::default(),
        );
    }

    #[test]
    fn dont_flag_there_in_that() {
        assert_no_lints("It's over there in that box.", Therein::default());
    }

    #[test]
    fn dont_flag_there_in_the() {
        assert_no_lints(
            "What was it up there in the song that seemed to be calling her back inside?",
            Therein::default(),
        );
    }

    #[test]
    fn dont_flag_there_in_your() {
        assert_no_lints(
            "If you want to get the latest changes there in your external save button, then you could use the html.get method.",
            Therein::default(),
        );
    }

    #[test]
    fn dont_flag_there_in_year() {
        assert_no_lints(
            "... but I think by the time I got moved there in 2001, Casey had moved on.",
            Therein::default(),
        );
    }

    #[test]
    fn dont_flag_point_it_there_in_settings() {
        assert_no_lints(
            "JetBrains Junie can read `AGENTS.md` once you point it there in Settings → Tools → Junie → Project Settings → Guidelines Path (it is not automatic yet).",
            Therein::default(),
        );
    }

    #[test]
    fn dont_flag_gets_you_there_in_two_commands() {
        assert_no_lints(
            "Quick Start below gets you there in two commands:",
            Therein::default(),
        );
    }
}
