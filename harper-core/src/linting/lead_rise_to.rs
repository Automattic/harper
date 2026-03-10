use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct LeadRiseTo {
    expr: Box<dyn Expr>,
}

impl Default for LeadRiseTo {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["lead", "led", "leads", "leading"])
                    .t_ws()
                    .t_aco("rise")
                    .t_ws()
                    .t_aco("to"),
            ),
        }
    }
}

impl ExprLinter for LeadRiseTo {
    type Unit = Chunk;

    fn description(&self) -> &str {
        "Corrects `leads rise to` to `gives rise to`."
    }

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let ltok = toks.first()?;
        let lspan = ltok.span;
        let lchars = lspan.get_content(src);

        const GAVE: &[char] = &['g', 'a', 'v', 'e'];
        const GIVE: &[char] = &['g', 'i', 'v', 'e'];
        const GIVEN: &[char] = &['g', 'i', 'v', 'e', 'n'];
        const GIVES: &[char] = &['g', 'i', 'v', 'e', 's'];
        const GIVING: &[char] = &['g', 'i', 'v', 'i', 'n', 'g'];

        const GAVE_GIVEN: &[&[char]] = &[GAVE, GIVEN];
        const GIVE_GAVE_GIVEN: &[&[char]] = &[GIVE, GAVE, GIVEN];

        let gchars: &[&[char]] = match lchars {
            ['l', 'e', 'a', 'd'] => GIVE_GAVE_GIVEN,
            ['l', 'e', 'd'] => GAVE_GIVEN,
            ['l', 'e', 'a', 'd', 's'] => &[GIVES][..],
            ['l', 'e', 'a', 'd', 'i', 'n', 'g'] => &[GIVING][..],
            _ => return None,
        };

        let suggestions: Vec<Suggestion> = gchars
            .iter()
            .map(|l| Suggestion::replace_with_match_case(l.to_vec(), lspan.get_content(src)))
            .collect();

        Some(Lint {
            span: lspan,
            lint_kind: LintKind::Usage,
            suggestions,
            message: "The correct idiom is `give rise to`.".to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::LeadRiseTo;
    use crate::linting::tests::{assert_suggestion_result, assert_top3_suggestion_result};

    #[test]
    fn fix_led_simple_past() {
        assert_suggestion_result(
            "In this way, it led rise to a kind of monotheism in Egypt.",
            LeadRiseTo::default(),
            "In this way, it gave rise to a kind of monotheism in Egypt.",
        );
    }

    #[test]
    fn fix_led_past_participle() {
        assert_top3_suggestion_result(
            "This had led rise to some issues, such as #2777 and some over Slack",
            LeadRiseTo::default(),
            "This had given rise to some issues, such as #2777 and some over Slack",
        );
    }

    #[test]
    fn fix_lead_spello_for_led() {
        assert_top3_suggestion_result(
            "This lead rise to a fair number of complaints over image quality which were not down to RPT.",
            LeadRiseTo::default(),
            "This gave rise to a fair number of complaints over image quality which were not down to RPT.",
        );
    }

    #[test]
    fn fix_lead_not_spello() {
        assert_suggestion_result(
            "Philosophy is important because it raises the questions that lead rise to the sciences.",
            LeadRiseTo::default(),
            "Philosophy is important because it raises the questions that give rise to the sciences.",
        );
    }

    #[test]
    fn fix_leads() {
        assert_suggestion_result(
            "This leads rise to another question of mine",
            LeadRiseTo::default(),
            "This gives rise to another question of mine",
        );
    }

    #[test]
    fn fix_leading() {
        assert_suggestion_result(
            "The severe bushfires have also created their own weather, leading rise to a phenomenon known as pyrocumulonimbus (pyroCB) storms.",
            LeadRiseTo::default(),
            "The severe bushfires have also created their own weather, giving rise to a phenomenon known as pyrocumulonimbus (pyroCB) storms.",
        );
    }
}
