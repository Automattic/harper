use crate::{
    expr::{All, Expr, FirstMatchOf, SequenceExpr}, linting::{ExprLinter, Lint, LintKind, Suggestion}, Token, TokenStringExt
};

pub struct ThatThan {
    expr: Box<dyn Expr>,
}

impl Default for ThatThan {
    fn default() -> Self {
        let comparativer_that = SequenceExpr::default()
            .then_comparative_adjective()
            .t_ws()
            .t_aco("that");

        let more_or_less_positive_that = SequenceExpr::word_set(&["more", "less"])
            .t_ws()
            .then_positive_adjective()
            .t_ws()
            .t_aco("that");        

        let exceptioner = SequenceExpr::default()
            .t_any().t_any().then_likely_homograph();

        let more_exception = SequenceExpr::default()
            .t_any().t_any().t_any().t_any().t_aco("that");

        let er_that = Box::new(All::new(vec![
            Box::new(comparativer_that),
            Box::new(exceptioner),
        ]));

        let more_that = Box::new(All::new(vec![
            Box::new(more_or_less_positive_that),
            Box::new(more_exception),
        ]));

        let expr = FirstMatchOf::new(vec![
            er_that,
            more_that,
        ]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for ThatThan {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() != 3 && toks.len() != 5 {
            return None;
        }
        if toks.len() == 5 {
            // more _ adj _ that
            let more_or_less_tok = toks.first()?;
            let that_tok = toks.last()?;

            return Some(Lint {
                span: that_tok.span,
                lint_kind: LintKind::Typo,
                suggestions: vec![Suggestion::replace_with_match_case_str(
                    "than",
                    that_tok.span.get_content(src),
                )],
                message: "This looks like a comparison that should use the word `than` instead of `that`.".to_string(),
                priority: 31,
            })
        }

        // adjer _ that

        let comparative_then_whitespace_toks = &toks[..2];
        let comparative_then_whitespace_str = comparative_then_whitespace_toks
            .span()?
            .get_content_string(src);
        let that_tok = toks.last()?;
        eprintln!("❤️ '{}'", comparative_then_whitespace_str);

        Some(Lint {
            span: that_tok.span,
            lint_kind: LintKind::Typo,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "than",
                that_tok.span.get_content(src),
            )],
            message:
                "This looks like a comparison that should use the word `than` instead of `that`."
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
    use crate::linting::{
        ThatThan,
        tests::{assert_lint_count, assert_suggestion_result},
    };

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
    #[ignore = "false positive that can probably be fixed"]
    fn dont_flag_easier_that_way() {
        assert_lint_count(
            "Given svelte now has signals, it might actually be easier that way.",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    #[ignore = "false positive might be difficult to fix"]
    fn dont_flag_better_that() {
        assert_lint_count(
            "So I am wondering if its better that I run SCENIC+ once on the integrated dataset or 3 times on the individual datasets",
            ThatThan::default(),
            0,
        );
    }

    #[test]
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
    fn fix_bigger_that() {
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
    #[ignore = "false positive that can probably be fixed"]
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
    #[ignore = "false positive that might be difficult to fix"]
    fn dont_flag_more_explicit_that() {
        assert_lint_count(
            "make it more explicit that those files are auto ...",
            ThatThan::default(),
            0,
        );
    }

    #[test]
    #[ignore = "false positive that might be difficult to fix"]
    fn dont_flag_more_clear_that() {
        assert_lint_count(
            "Make it more clear that users need to download the VS tooling installer for .NET Core in VS.",
            ThatThan::default(),
            0,
        );
    }
}
