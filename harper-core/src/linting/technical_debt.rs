use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
    patterns::WordSet,
};

use std::cmp::min;

pub struct TechnicalDebt {
    expr: SequenceExpr,
}

impl Default for TechnicalDebt {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::optional(
                SequenceExpr::longest_of(vec![
                    Box::new(WordSet::new(&[
                        "are", "both", "multiple", "several", "these", "those", "were",
                    ])),
                    Box::new(
                        SequenceExpr::optional(SequenceExpr::word_set(&["are", "were"]).t_ws())
                            .then(SequenceExpr::word_set(&["few", "many"])),
                    ),
                ])
                .t_ws(),
            )
            .t_aco("technical")
            .t_ws()
            .t_aco("debts")
            .then_optional(
                SequenceExpr::whitespace().then(SequenceExpr::word_set(&["are", "were"]).t_ws()),
            ),
        }
    }
}

impl ExprLinter for TechnicalDebt {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, context, src));
        // [0/2/4 tokens before] [3 tokens "technical debts"] [0/2 tokens after]
        let idx = (0..min(5, toks.len()))
            .step_by(2)
            .position(|i| {
                toks[i + 1].kind.is_whitespace()
                    && toks[i].get_ch(src).eq_str("technical")
                    && toks[i + 2].get_ch(src).eq_str("debts")
            })
            .map(|p| p * 2)?;

        // ✅               TDs -> TD
        // ❌           are TDs -> are TD x 4
        // ✅      are many TDs -> is a lot of TD
        // ✅          many TDs -> a lot of TD x 2
        // ❌ (so/too) many TDs -> (so/too) much TD x 2

        let plural_to_singular = &[
            ("are", "is"),
            ("debts", "debt"),
            ("few", "little"),
            ("many", "a lot of"),
            ("these", "this"),
            ("those", "that"),
            ("were", "was"),
        ][..];

        let mut correction: Vec<char> = vec![];

        for i in 0..toks.len() {
            let raw = toks[i].get_ch(src);
            let xlated: Vec<char> = plural_to_singular
                .iter()
                .find(|(r, _)| raw.eq_str(r))
                .map(|(_, x)| x)
                .map(|x| x.chars().collect())
                .unwrap_or_else(|| raw.to_vec());
            correction.extend(xlated);
        }

        let span = toks.span()?;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                correction,
                span.get_content(src),
            )],
            message: "Technical debt should not be pluralized".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Checks for `technical debt` being used in the plural form."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::TechnicalDebt;

    #[test]
    fn test_technical_debt() {
        // ✅ TDs -> TD
        assert_suggestion_result(
            "technical debts",
            TechnicalDebt::default(),
            "technical debt",
        );
    }

    #[test]
    fn are_before_doesnt_change() {
        // ❌ are TDs -> are TD
        assert_suggestion_result(
            "Any lines of code we write are technical debts no matter what; its all about managing the debts to successfully engineer the solution for the business. I am ...Read more",
            TechnicalDebt::default(),
            "Any lines of code we write are technical debt no matter what; its all about managing the debt to successfully engineer the solution for the business. I am ...Read more",
        );
    }

    #[test]
    fn create_many_to_a_lot_of() {
        // ✅ many TDs -> a lot of TD
        assert_suggestion_result(
            "This visibility issue had create many technical debts (like unnecessary duplications) in my team, and it gets even worse the team were having a tight deadline.",
            TechnicalDebt::default(),
            "This visibility issue had create a lot of technical debt (like unnecessary duplications) in my team, and it gets even worse the team were having a tight deadline.",
        );
    }

    #[test]
    fn there_are_many() {
        // ✅ are many TDs -> is a lot of TD
        assert_suggestion_result(
            "the App has implemented simple and basic and there are many technical debts which should be done and several features that it could have. TODOS. just listed ...Read more",
            TechnicalDebt::default(),
            "the App has implemented simple and basic and there is a lot of technical debt which should be done and several features that it could have. TODOS. just listed ...Read more",
        );
    }

    #[test]
    fn we_write_are() {
        // ❌ are TDs -> are TD
        assert_suggestion_result(
            "Any lines of code we write are technical debts no matter what; its all about managing the debts to successfully engineer the solution for the business. I am ...Read more",
            TechnicalDebt::default(),
            "Any lines of code we write are technical debt no matter what; its all about managing the debt to successfully engineer the solution for the business. I am ...Read more",
        );
    }

    #[test]
    fn so_many_to_so_much() {
        // ❌ (so) many TDs -> (so) much TD
        assert_suggestion_result(
            "So many things have changed... and I accumulated so many technical debts! So, Drupal 8 came out and I still need to catch up the ongoing ...Read more",
            TechnicalDebt::default(),
            "So many things have changed... and I accumulated so much technical debt! So, Drupal 8 came out and I still need to catch up the ongoing ...Read more",
        );
    }

    #[test]
    fn a_and_b_are() {
        // ❌ are TDs -> are TD
        assert_suggestion_result(
            "Unfortunately, both the hard dependency on Elasticsearch and memory use are technical debts for our use case. We currently get our AAA reports from LDAP so ...",
            TechnicalDebt::default(),
            "Unfortunately, both the hard dependency on Elasticsearch and memory use are technical debt for our use case. We currently get our AAA reports from LDAP so ...",
        );
    }

    #[test]
    fn theres_many_to_a_lot_of() {
        // ✅ many TDs -> a lot of TD
        assert_suggestion_result(
            "since this is the first time I've done any js/react/html/css, there's many technical debts to work on. accessibility; semantics html; api key ...Read more",
            TechnicalDebt::default(),
            "since this is the first time I've done any js/react/html/css, there's a lot of technical debt to work on. accessibility; semantics html; api key ...Read more",
        );
    }

    #[test]
    fn both_are() {
        // ❌ are TDs -> are TD
        assert_suggestion_result(
            "Both are technical debts, and remedy is to switch code to use current APIs.",
            TechnicalDebt::default(),
            "Both are technical debt, and remedy is to switch code to use current APIs.",
        );
    }

    #[test]
    fn too_many_too_much() {
        // ❌ (too) many TDs -> (too) much TD
        assert_suggestion_result(
            "Now WinUI has too many technical debts.",
            TechnicalDebt::default(),
            "Now WinUI has too much technical debt.",
        );
    }
}
