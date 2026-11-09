use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct DenyOffer {
    expr: SequenceExpr,
}

impl Default for DenyOffer {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["deny", "denies", "denied", "denying"])
                .t_ws()
                .then_optional(SequenceExpr::default().then_determiner().t_ws())
                .t_set(["offer", "offers"]),
        }
    }
}

impl ExprLinter for DenyOffer {
    type Unit = Chunk;

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let span = matched_tokens.first()?.span;

        let corrections = match matched_tokens.first()?.get_ch(source).last()? {
            'y' | 'Y' => &["decline", "reject"],
            'd' | 'D' => &["declined", "rejected"],
            's' | 'S' => &["declines", "rejects"],
            'g' | 'G' => &["declining", "rejecting"],
            _ => return None,
        };

        let suggestions = corrections
            .iter()
            .map(|correction| {
                Suggestion::replace_with_match_case_str(correction, span.get_content(source))
            })
            .collect();

        let message =
            "You would `deny` permission or a request, but `decline` or `reject` an offer."
                .to_owned();

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `deny` when used with `offer` to `decline` or `reject`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::DenyOffer;

    #[test]
    fn fix_deny_the_offer() {
        assert_suggestion_result(
            "I did not want to accept or deny the offer without floating to other maintainers",
            DenyOffer::default(),
            "I did not want to accept or decline the offer without floating to other maintainers",
        );
    }

    #[test]
    fn deny_your_offer() {
        assert_suggestion_result(
            "I understand, why you'd want this, but must deny your offer to expand my participation in your project quite so dramatically at this time.",
            DenyOffer::default(),
            "I understand, why you'd want this, but must reject your offer to expand my participation in your project quite so dramatically at this time.",
        );
    }

    #[test]
    fn denied_their_offer() {
        assert_suggestion_result(
            "So I denied their offer of employment and went back to finish my masters degree",
            DenyOffer::default(),
            "So I declined their offer of employment and went back to finish my masters degree",
        );
    }

    #[test]
    fn dying_offers() {
        assert_suggestion_result(
            "Seemingly denying an offer is translated as accepting it",
            DenyOffer::default(),
            "Seemingly rejecting an offer is translated as accepting it",
        );
    }

    #[test]
    fn deny_offers_and_denying_offers() {
        assert_suggestion_result(
            "For details about how a Marketplace administrator can deny offers in the Back Office, see Approving or denying offers.",
            DenyOffer::default(),
            "For details about how a Marketplace administrator can decline offers in the Back Office, see Approving or declining offers.",
        );
    }
}
