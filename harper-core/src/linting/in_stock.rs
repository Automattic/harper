use crate::{
    Lint, Token, TokenKind,
    char_string::CharStringExt,
    expr::{AnchorEnd, Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
    patterns::{InflectionOfBe, Word},
};

pub struct InStock {
    expr: SequenceExpr,
}

impl Default for InStock {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_of([
                Box::new(InflectionOfBe::default()) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::word_set(&["have", "had", "has", "having"]).then_optional(
                        SequenceExpr::whitespace().t_set(&[
                            "it",
                            "one",
                            "some",
                            "them",
                            "inventory",
                        ]),
                    ),
                ),
                Box::new(Word::new("back")),
            ])
            .t_ws()
            .then_word_seq(&["on", "stock"])
            .then_any_of([
                Box::new(AnchorEnd) as Box<dyn Expr>,
                Box::new(
                    SequenceExpr::whitespace()
                        .then_kind_either(TokenKind::is_conjunction, TokenKind::is_preposition),
                ),
            ]),
        }
    }
}

impl ExprLinter for InStock {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let tok = find_the_only_token_matching(toks, src, |t, _| t.get_ch(src).eq_str("on"))?;

        Some(Lint {
            span: tok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "in",
                tok.get_ch(src),
            )],
            message: "The correct usage is `in stock` not `on stock`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `on stock` to `in stock`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::InStock;

    #[test]
    fn fix_are() {
        assert_suggestion_result(
            "don't want to redirect to product page if not enough products are on stock",
            InStock::default(),
            "don't want to redirect to product page if not enough products are in stock",
        );
    }

    #[test]
    fn fix_at() {
        assert_suggestion_result(
            "Python script to check if a product is back on stock at flipkart.com - shyamjos/flipkart_checker.",
            InStock::default(),
            "Python script to check if a product is back in stock at flipkart.com - shyamjos/flipkart_checker.",
        );
    }

    #[test]
    fn fix_back() {
        assert_suggestion_result(
            "notify customer when an article is back on stock",
            InStock::default(),
            "notify customer when an article is back in stock",
        );
    }

    #[test]
    fn fix_be() {
        assert_suggestion_result(
            "Could there be some complete stock goods listing regardless how many products should be on stock?",
            InStock::default(),
            "Could there be some complete stock goods listing regardless how many products should be in stock?",
        );
    }

    #[test]
    fn fix_has() {
        assert_suggestion_result(
            "I think it would be best if the BOM and CPL files could be updated with a component that JLCPBC has on stock.",
            InStock::default(),
            "I think it would be best if the BOM and CPL files could be updated with a component that JLCPBC has in stock.",
        );
    }

    #[test]
    fn fix_has_but() {
        assert_suggestion_result(
            "Can't Notify customer if product has inventory on stock but not on particular source",
            InStock::default(),
            "Can't Notify customer if product has inventory in stock but not on particular source",
        );
    }

    #[test]
    fn fix_have() {
        assert_suggestion_result(
            "classic product that we don't have on stock or we may still have few",
            InStock::default(),
            "classic product that we don't have in stock or we may still have few",
        );
    }

    #[test]
    fn fix_have_them() {
        assert_suggestion_result(
            "but we also may have them on stock for x reasons",
            InStock::default(),
            "but we also may have them in stock for x reasons",
        );
    }

    #[test]
    fn fix_is() {
        assert_suggestion_result(
            "why not immidiatly show the 2000 as that is what is on stock.",
            InStock::default(),
            "why not immidiatly show the 2000 as that is what is in stock.",
        );
    }

    // Avoid potential false positives

    #[test]
    fn dont_flag_is_on_stock_samsung_roms() {
        assert_no_lints(
            "So i am pretty sure issue is on stock samsung roms",
            InStock::default(),
        );
    }

    #[test]
    fn dont_flag_on_stock_samba_lens() {
        assert_no_lints(
            "Samba lens is missing on Stock Lenses page",
            InStock::default(),
        );
    }

    #[test]
    fn dont_flag_on_stock_os() {
        assert_no_lints(
            "I'm actually not sure if that prop is really doing anything even on stock OS.",
            InStock::default(),
        );
    }

    #[test]
    fn dont_flag_on_stock_pixel() {
        assert_no_lints(
            "Storage Manager link is invisible on stock Pixel OS too.",
            InStock::default(),
        );
    }
}
