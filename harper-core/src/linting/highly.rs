use std::iter::once;

use crate::{
    CharStringExt, Lint, Token, TokenKind,
    expr::{Expr, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

const OK_AFTER_HIGHLY: &[&str] = &[
    "accurate",
    "appreciated",
    "available",
    "beneficial",
    "capable",
    "compatible",
    "competitive",
    "complex",
    "compressed",
    "concurrent",
    "condensed",
    "configurable",
    "controllable",
    "controlled",
    "correlated",
    "critical",
    "customizable",
    "customized",
    "dangerous",
    "dependent",
    "desirable",
    "detailed",
    "distracting",
    "distributed",
    "effective",
    "efficient",
    "encapsulated",
    "encouraged",
    "extensible",
    "flexible",
    "grateful",
    "gratified",
    "impressed",
    "indebted",
    "informative",
    "integrated",
    "likely",
    "obliged",
    "opinionated",
    "optimized",
    "performant",
    "pleased",
    "portable",
    "powerful",
    "precise",
    "probable",
    "realistic",
    "recommended",
    "regarded",
    "relevant",
    "respected",
    "responsive",
    "scalable",
    "sceptical",
    "searchable",
    "secure",
    "sensitive",
    "significant",
    "skeptical",
    "skilled",
    "stable",
    "successful",
    "susceptible",
    "thankful",
    // token-efficient
    "toxic",
    "tuned",
    "unlikely",
    "unstructured",
    "valued",
    "variable",
    "versatile",
];

const APPROPRIATE: &[(&[&str], &str)] = &[
    (&["strongly"], "advised"),
    (&["strictly", "tightly"], "controlled"),
    (&["fully"], "developed"),
    (&["widely"], "embraced"),
    (&["deeply"], "honored"),
    (&["deeply"], "honoured"),
    (&["heavily"], "inspired"),
    (&["glaringly"], "obvious"),
    (&["slickly"], "produced"),
    (&["closely"], "related"),
    (&["warmly"], "welcomed"),
];

const OK_AFTER_HIGHLY_IN: &[&str] = &[
    "accord",
    "agreement",
    "conceit",
    "debt",
    "demand",
    "doubt",
    "error",
    "favor",
    "favour",
    "love",
    "need",
    "order",
    "sympathy",
    "tune",
    "vogue",
];

pub struct Highly {
    expr: SequenceExpr,
}

impl Default for Highly {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("highly").t_ws().then_any_of([
                Box::new(
                    SequenceExpr::default()
                        .then_kind_either(TokenKind::is_adjective, TokenKind::is_adverb)
                        .but_not(SequenceExpr::word_set(OK_AFTER_HIGHLY)),
                ),
                Box::new(
                    SequenceExpr::aco("in")
                        .t_ws_h()
                        .then_mass_noun()
                        .but_not(SequenceExpr::anything().t_any().t_set(OK_AFTER_HIGHLY_IN)),
                ),
            ]),
        }
    }
}

impl ExprLinter for Highly {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let hi_tok = toks.first()?;
        let adjadv_tok = toks.get(2)?;
        let adjadv_ch = adjadv_tok.get_ch(src);

        let hi_span = hi_tok.span;

        // Find specific tailored replacements based on the adjective/adverb
        let specific_intensifiers = APPROPRIATE
            .iter()
            .find(|(_, adjadv)| adjadv_ch.eq_str(adjadv))
            .map(|(intensifiers, _)| *intensifiers)
            .unwrap_or(&[]);

        let suggestions: Vec<Suggestion> = once(&"very")
            .chain(specific_intensifiers.iter())
            .map(|intensifier| {
                Suggestion::replace_with_match_case_str(intensifier, hi_span.get_content(src))
            })
            .collect();

        Some(Lint {
            span: hi_span,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "`Highly` might not sound natural in this context.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Replaces `highly` with `very` or more appropriate words in contexts that don't sound natural."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Highly;

    #[test]
    fn controlled() {
        assert_suggestion_result(
            "this fast/deep thinking mode can be switched in a highly controlled fashion.",
            Highly::default(),
            "this fast/deep thinking mode can be switched in a highly controlled fashion.",
        );
    }

    #[test]
    fn experimental() {
        assert_suggestion_result(
            "It is also highly experimental.",
            Highly::default(),
            "It is also very experimental.",
        );
    }

    #[test]
    fn fitting() {
        assert_suggestion_result(
            "producing highly fitting vocal expression trained on a massive 1.8 million-hour bilingual corpus.",
            Highly::default(),
            "producing very fitting vocal expression trained on a massive 1.8 million-hour bilingual corpus.",
        );
    }

    #[test]
    fn obvious() {
        assert_suggestion_result(
            "from highly obvious to extremely subtle.",
            Highly::default(),
            "from glaringly obvious to extremely subtle.",
        );
    }

    #[test]
    fn produced() {
        assert_suggestion_result(
            "The flaws of the game showed Epic that there was an opening for a more highly produced battle royale game in the market",
            Highly::default(),
            "The flaws of the game showed Epic that there was an opening for a more slickly produced battle royale game in the market",
        );
    }

    #[test]
    fn replayable() {
        assert_suggestion_result(
            "Bungie marketed Destiny as a highly replayable infinite content machine.",
            Highly::default(),
            "Bungie marketed Destiny as a very replayable infinite content machine.",
        );
    }

    #[test]
    fn valuable() {
        assert_suggestion_result(
            "Automating the R&D process in data science is a highly valuable yet underexplored area in industry.",
            Highly::default(),
            "Automating the R&D process in data science is a highly valuable yet underexplored area in industry.",
        );
    }

    #[test]
    fn welcomed() {
        assert_suggestion_result(
            "Your PR of new Quant models is highly welcomed.",
            Highly::default(),
            "Your PR of new Quant models is warmly welcomed.",
        );
    }
}
