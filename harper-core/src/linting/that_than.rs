use crate::linting::expr_linter::Chunk;
use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct ThatThan {
    expr: Box<dyn Expr>,
}

impl Default for ThatThan {
    fn default() -> Self {
        let adjective_er_that_nextword = SequenceExpr::default()
            .then_kind_except(
                TokenKind::is_comparative_adjective,
                &["better", "later", "number"],
            )
            .t_ws()
            .t_aco("that")
            .t_ws()
            .then_word_except(&["way"]);

        let more_less_adj_that_you_think = SequenceExpr::word_set(&["more", "less"])
            .t_ws()
            .then_kind_either(TokenKind::is_adjective, TokenKind::is_adverb)
            .t_ws()
            .t_aco("that")
            .t_ws()
            .t_aco("you")
            .t_ws()
            .t_aco("think");

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(adjective_er_that_nextword),
                Box::new(more_less_adj_that_you_think),
            ])),
        }
    }
}

impl ExprLinter for ThatThan {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let mut that_toks = toks
            .iter()
            .filter(|tok| tok.span.get_content(src).eq_ignore_ascii_case_str("that"));

        let that_tok = match (that_toks.next(), that_toks.next()) {
            (Some(tok), None) => tok,
            _ => return None,
        };

        Some(Lint {
            span: that_tok.span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "than",
                that_tok.span.get_content(src),
            )],
            message: "This looks like a comparison that should use `than` rather than `that`."
                .to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects the typo `that` to `than` in comparisons."
    }
}

#[cfg(test)]
mod tests {
    use super::ThatThan;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // adj-er that

    #[test]
    fn fix_slower_that() {
        assert_suggestion_result(
            "Local installed PHAR 5x times slower that the same PHAR installed globally",
            ThatThan::default(),
            "Local installed PHAR 5x times slower than the same PHAR installed globally",
        );
    }

    #[test]
    fn dont_flag_more_that() {
        assert_lint_count(
            "so it's probably more that Croatian had an easier test",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_easier_that_way() {
        assert_lint_count(
            "Given svelte now has signals, it might actually be easier that way.",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_better_that() {
        assert_lint_count(
            "So I am wondering if its better that I run SCENIC+ once on the integrated dataset or 3 times on the individual datasets",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    #[ignore = "not handled because 'better' results in false positives"]
    fn fix_better_that() {
        assert_suggestion_result(
            "Examples of how different cards perform far better that others.",
            ThatThan::default(),
            "Examples of how different cards perform far better than others.",
        );
    }

    #[test]
    fn fix_smaller_that() {
        assert_suggestion_result(
            "When the resulting part is smaller that the build plate, it gets re-arranged.",
            ThatThan::default(),
            "When the resulting part is smaller than the build plate, it gets re-arranged.",
        );
    }

    #[test]
    #[ignore = "not handled because 'bigger' results in false positives"]
    fn cant_flag_bigger_that() {
        assert_suggestion_result(
            "Enable bigger that 1024*768 window for world builder.",
            ThatThan::default(),
            "Enable bigger than 1024*768 window for world builder.",
        );
    }

    #[test]
    fn fix_longer_that() {
        assert_suggestion_result(
            "Window list in CodeBrowser can be longer that screen height.",
            ThatThan::default(),
            "Window list in CodeBrowser can be longer than screen height.",
        );
    }

    #[test]
    #[ignore = "'less that' also occurs in false positives"]
    fn fix_less_that() {
        assert_suggestion_result(
            "Collector Not collecting metrics if the collection interval is less that the metric generation interval.",
            ThatThan::default(),
            "Collector Not collecting metrics if the collection interval is less than the metric generation interval.",
        );
    }

    #[test]
    fn fix_faster_that() {
        assert_suggestion_result(
            "with the general case performing approximately 4x faster that a Vec based implementation",
            ThatThan::default(),
            "with the general case performing approximately 4x faster than a Vec based implementation",
        );
    }

    #[test]
    fn fix_taller_that() {
        assert_suggestion_result(
            "Notice that people we've already placed are not taller that the current person.",
            ThatThan::default(),
            "Notice that people we've already placed are not taller than the current person.",
        );
    }

    #[test]

    fn dont_fix_faster_that_way() {
        assert_lint_count(
            "You will get an answer quicker that way!",
            ThatThan::default(),
            0,
        )
    }

    #[test]
    fn dont_fix_lighter_that() {
        assert_lint_count(
            "This is the code for Seed-Studio-based timer and desk lighter that I built as a gift for a good friend.",
            ThatThan::default(),
            0,
        )
    }

    // more/less adj that

    #[test]
    fn dont_flag_more_explicit_that() {
        assert_lint_count(
            "make it more explicit that those files are auto ...",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_more_clear_that() {
        assert_lint_count(
            "Make it more clear that users need to download the VS tooling installer for .NET Core in VS.",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    fn fix_more_complicated_that_you_think() {
        assert_suggestion_result(
            "Space Invaders is more complicated that you think.",
            ThatThan::default(),
            "Space Invaders is more complicated than you think.",
        );
    }

    #[test]
    fn dont_flag_more_likely_that_you_will() {
        assert_lint_count(
            "It is more likely that you will win this round.",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    fn fix_less_obvious_that_you_think() {
        assert_suggestion_result(
            "It is less obvious that you think.",
            ThatThan::default(),
            "It is less obvious than you think.",
        );
    }

    #[test]
    fn fix_more_subtly_that_you_think() {
        assert_suggestion_result(
            "The issue is more subtly that you think.",
            ThatThan::default(),
            "The issue is more subtly than you think.",
        );
    }

    #[test]
    fn fix_more_complicated_that_you_think_uppercase() {
        assert_suggestion_result(
            "Space Invaders is more complicated THAT you think.",
            ThatThan::default(),
            "Space Invaders is more complicated THAN you think.",
        );
    }

    #[test]
    fn dont_flag_more_apparent_that_we_needed_logs() {
        assert_lint_count(
            "It became more apparent that we needed logs.",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_less_likely_that_they_will_fail() {
        assert_lint_count(
            "It is less likely that they will fail.",
            ThatThan::default(),
            0,
        );
    }

    // False positives from The Great Gatsby

    #[test]
    fn dont_flag_i_gathered_later_that() {
        assert_lint_count(
            "and I gathered later that he was a photographer",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_its_better_that() {
        assert_lint_count(
            "It’s better that the shock should all come at once.",
            ThatThan::default(),
            0,
        )
    }

    #[test]
    fn dont_flag_number_that_1663() {
        assert_lint_count(
            " 455 │ `MAJOR.MINOR.PATCH` version number that increments with:",
            ThatThan::default(),
            0,
        )
    }
}
