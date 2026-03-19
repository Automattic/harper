use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::patterns::WordSet;
use crate::{Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind};
use crate::linting::expr_linter::Chunk;

pub struct VerbToAdjective {
    expr: SequenceExpr,
}

impl Default for VerbToAdjective {
    fn default() -> Self {
        let expr = SequenceExpr::word_set(&["the", "a", "an"])
            .t_ws()
            .then_kind_where(|kind| kind.is_degree_adverb())
            .t_ws()
            .then_kind_where(|kind| kind.is_noun() && kind.is_verb_progressive_form())
            .t_ws()
            .then(WordSet::new(&["of"]));

        Self { expr }
    }
}

impl ExprLinter for VerbToAdjective {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let words: Vec<_> = matched_tokens
            .iter()
            .filter(|tok| tok.kind.is_word())
            .collect();
        let [_, adverb, noun, _] = words.as_slice() else {
            return None;
        };

        Some(Lint {
            span: matched_tokens.span()?,
            lint_kind: LintKind::Typo,
            suggestions: vec![],
            message: format!(
                "`{}` is an adverb. Before the noun `{}`, this phrase more likely needs an adjective.",
                adverb.span.get_content_string(source),
                noun.span.get_content_string(source),
            ),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Looks for article-led gerund noun phrases like `a fully accounting of`, where an adjective is more likely than an adverb."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_lint_count, assert_lint_message, assert_no_lints};

    use super::VerbToAdjective;

    fn assert_each_lints(cases: &[&str]) {
        for case in cases {
            assert_lint_count(case, VerbToAdjective::default(), 1);
        }
    }

    fn assert_each_no_lints(cases: &[&str]) {
        for case in cases {
            assert_no_lints(case, VerbToAdjective::default());
        }
    }

    #[test]
    fn flags_issue_2855_and_similar_gerund_noun_phrases() {
        assert_each_lints(&[
            "By scheduling time to do a fully accounting of where your CPU cycles are going, you can preemptively save yourself (and your contributors) a lot of time.",
            "We need a fully understanding of the risk before launch.",
            "The mockup offered a fully rendering of the scene.",
            "An entirely recording of the hearing was never released.",
            "The appendix contained a completely listing of every exception.",
            "The report ended with a highly framing of the debate.",
            "A fully mapping of the cave is still useful to explorers.",
            "The board requested a truly accounting of the losses.",
        ]);
    }

    #[test]
    fn message_points_to_the_adverb_and_noun() {
        assert_lint_message(
            "We need a fully understanding of the risk before launch.",
            VerbToAdjective::default(),
            "`fully` is an adverb. Before the noun `understanding`, this phrase more likely needs an adjective.",
        );
    }

    #[test]
    fn allows_the_issue_2855_regressions() {
        assert_each_no_lints(&[
            "South Korea Set To Get a Fully Functioning Google Maps integration.",
            "This allows our clients to embrace a truly data-driven approach.",
            "Before he could press the transmit button, the sphere emitted a high-pitched whine.",
            "The target of coordinated, algorithmic pursuit kept moving.",
            "A fully functioning prototype shipped yesterday.",
            "A truly remarkable outcome followed.",
            "A completely different problem remains.",
            "A fully rendered image loaded instantly.",
        ]);
    }

    #[test]
    fn allows_correct_adjective_plus_gerund_noun_of_phrases() {
        assert_each_no_lints(&[
            "We need a full understanding of the risk before launch.",
            "The audit delivered a full accounting of the losses.",
            "The appendix included a complete listing of the requirements.",
            "The demo provided a detailed rendering of the scene.",
            "They archived a proper recording of the interview.",
            "The memo offered a clear framing of the issue.",
            "The atlas provides a partial mapping of the cave.",
            "The exhibit included the recording of the speech.",
        ]);
    }

    #[test]
    fn allows_nonmatching_determiner_phrases_and_previous_regressions() {
        assert_each_no_lints(&[
            "I really like my new car.",
            "I want you to write a new sentence for me.",
            "Ensure the correct term is used for individuals residing abroad.",
            "It is something that causes amazement.",
            "Can you suggest a correct auxiliary?",
            "This is the email address that will receive the submitted form data.",
            "Not the unexplored territories, ripe for discovery, but the areas actively erased, obscured, or simply deemed unworthy of representation?",
            "A local mapping service is sufficient here.",
            "The transmit queue drained normally.",
            "A mostly accurate summary is still useful.",
        ]);
    }
}
