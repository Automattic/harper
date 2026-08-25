use crate::{
    CharStringExt, Dialect, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct TillDate {
    expr: SequenceExpr,
    dialect: Dialect,
}

impl TillDate {
    pub fn new(dialect: Dialect) -> Self {
        Self {
            expr: SequenceExpr::word_set(&["till", "til", "'til", "til'"])
                .t_ws()
                .t_aco("date"),
            dialect,
        }
    }
}

impl ExprLinter for TillDate {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        // TODO: It might be better to also have a preference to flag this even in Indian English?
        if self.dialect == Dialect::Indian {
            return None;
        }

        const GOOD_TOKENS: &[&str] = &["good", "valid"];

        // "Good Till Date" (GTD) is a term of art in finance.
        // Don't flag:
        //              [good] [ws] <till date pattern>
        // [good] [ws] [apostrophe] <till date pattern>
        if let Some((before, _)) = ctx
            && let [before @ .., prevprev, prev] = before
        {
            if prev.kind.is_whitespace()
                && prevprev
                    .get_ch(src)
                    .eq_any_ignore_ascii_case_str(GOOD_TOKENS)
            {
                return None;
            }

            if prev.kind.is_apostrophe()
                && prevprev.kind.is_whitespace()
                && let [.., prevprevprev] = before
                && prevprevprev
                    .get_ch(src)
                    .eq_any_ignore_ascii_case_str(GOOD_TOKENS)
            {
                return None;
            }
        }

        let till_idx = 0;
        let till_tok = &toks[till_idx];
        let till_span = till_tok.span;

        Some(Lint {
            span: till_span,
            lint_kind: LintKind::Regionalism,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "to",
                till_span.get_content(src),
            )],
            message: "Outside Indian English, prefer `to date`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the Indian English `till date` to `to date` when Indian English is not the selected dialect."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Dialect,
        linting::tests::{assert_no_lints, assert_suggestion_result},
    };

    use super::TillDate;

    #[test]
    fn fix_till_date_double_l() {
        assert_suggestion_result(
            "This repository contains all my REXX codes till date.",
            TillDate::new(Dialect::American),
            "This repository contains all my REXX codes to date.",
        );
    }

    #[test]
    fn fix_till_date_single_l() {
        assert_suggestion_result(
            "streams recorded on the platform from 1930 til date",
            TillDate::new(Dialect::American),
            "streams recorded on the platform from 1930 to date",
        );
    }

    #[test]
    fn dont_flag_til_date_apostrophe_before_single_l() {
        assert_no_lints(
            "A good 'til date (GTD) order is an instruction to buy or sell an asset that remains open until a future date specified by the trader.",
            TillDate::new(Dialect::American),
        );
    }

    #[test]
    fn dont_flag_good_til_date_apostrophe_after_single_l() {
        assert_no_lints(
            "Can someone give me some information as to what the Good Til' date means in the context of buying shares?",
            TillDate::new(Dialect::American),
        );
    }

    #[test]
    fn dont_flag_good_till_date_double_l_all_caps() {
        assert_no_lints(
            "TERMS AND CONDITIONS FOR GOOD TILL DATE (GTD) ORDERS.",
            TillDate::new(Dialect::American),
        );
    }

    #[test]
    fn dont_flag_valid_till_date() {
        assert_no_lints(
            "The quotation states: \"Good till date (GTD) orders are valid until a specified date.\"",
            TillDate::new(Dialect::American),
        );
    }
}
