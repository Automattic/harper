use harper_brill::UPOS;

use crate::Document;
use crate::TokenStringExt;
use crate::expr::All;
use crate::expr::Expr;
use crate::expr::ExprExt;
use crate::expr::OwnedExprExt;
use crate::expr::SequenceExpr;
use crate::patterns::NominalPhrase;
use crate::patterns::Pattern;
use crate::patterns::UPOSSet;
use crate::patterns::WordSet;
use crate::{
    Token,
    linting::{Lint, LintKind, Suggestion},
};

use super::Linter;

pub struct ItsContraction {
    expr: Box<dyn Expr>,
}

impl Default for ItsContraction {
    fn default() -> Self {
        let positive = SequenceExpr::default().t_aco("its").then_whitespace().then(
            UPOSSet::new(&[UPOS::VERB, UPOS::AUX, UPOS::DET, UPOS::PRON])
                .or(WordSet::new(&["because"])),
        );

        let exceptions = SequenceExpr::default()
            .then_anything()
            .then_anything()
            .then(WordSet::new(&["own", "intended"]));

        let inverted = SequenceExpr::default().then_unless(exceptions);

        let expr = All::new(vec![Box::new(positive), Box::new(inverted)]).or_longest(
            SequenceExpr::aco("its")
                .t_ws()
                .then(UPOSSet::new(&[UPOS::ADJ]))
                .t_ws()
                .then(UPOSSet::new(&[UPOS::SCONJ, UPOS::PART])),
        );

        Self {
            expr: Box::new(expr),
        }
    }
}

impl Linter for ItsContraction {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for chunk in document.iter_chunks() {
            lints.extend(
                self.expr
                    .iter_matches(chunk, source)
                    .filter_map(|match_span| {
                        self.match_to_lint(&chunk[match_span.start..], source)
                    }),
            );
        }

        lints
    }

    fn description(&self) -> &str {
        "Detects the possessive `its` before `had`, `been`, or `got` and offers `it's` or `it has`."
    }
}

impl ItsContraction {
    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let offender = toks.first()?;
        let offender_chars = offender.span.get_content(source);

        if toks.get(2)?.kind.is_upos(UPOS::VERB)
            && NominalPhrase.matches(&toks[2..], source).is_some()
        {
            return None;
        }

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("it's", offender_chars),
                Suggestion::replace_with_match_case_str("it has", offender_chars),
            ],
            message: "Use `it's` (short for `it has` or `it is`) here, not the possessive `its`."
                .to_owned(),
            priority: 54,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::default;

    use super::ItsContraction;
    use crate::linting::tests::{assert_lint_count, assert_no_lints, assert_suggestion_result};

    #[test]
    fn fix_had() {
        assert_suggestion_result(
            "Its had an enormous effect.",
            ItsContraction::default(),
            "It's had an enormous effect.",
        );
    }

    #[test]
    fn fix_been() {
        assert_suggestion_result(
            "Its been months since we spoke.",
            ItsContraction::default(),
            "It's been months since we spoke.",
        );
    }

    #[test]
    fn fix_got() {
        assert_suggestion_result(
            "I think its got nothing to do with us.",
            ItsContraction::default(),
            "I think it's got nothing to do with us.",
        );
    }

    #[test]
    fn fixes_its_common() {
        assert_suggestion_result(
            "Its common for users to get frustrated.",
            ItsContraction::default(),
            "It's common for users to get frustrated.",
        );
    }

    #[test]
    fn ignore_correct_contraction() {
        assert_lint_count(
            "It's been a long year for everyone.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_possessive() {
        assert_lint_count(
            "The company revised its policies last week.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_coroutine() {
        assert_lint_count(
            "Launch each task within its own child coroutine.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn issue_381() {
        assert_suggestion_result(
            "Its a nice day.",
            ItsContraction::default(),
            "It's a nice day.",
        );
    }

    #[test]
    fn ignore_nominal_progressive() {
        assert_lint_count(
            "The class preserves its existing properties.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_nominal_perfect() {
        assert_lint_count(
            "The robot followed its predetermined route.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn ignore_nominal_long() {
        assert_lint_count(
            "I think of its exploding marvelous spectacular output.",
            ItsContraction::default(),
            0,
        );
    }

    #[test]
    fn corrects_because() {
        assert_suggestion_result(
            "Its because they don't want to.",
            ItsContraction::default(),
            "It's because they don't want to.",
        );
    }

    #[test]
    fn corrects_its_hard() {
        assert_suggestion_result(
            "Its hard to believe that.",
            ItsContraction::default(),
            "It's hard to believe that.",
        );
    }

    #[test]
    fn corrects_its_easy() {
        assert_suggestion_result(
            "Its easy if you try.",
            ItsContraction::default(),
            "It's easy if you try.",
        );
    }

    #[test]
    fn corrects_its_a_picnic() {
        assert_suggestion_result(
            "Its a beautiful day for a picnic",
            ItsContraction::default(),
            "It's a beautiful day for a picnic",
        );
    }

    #[test]
    fn corrects_its_my() {
        assert_suggestion_result(
            "Its my favorite song.",
            ItsContraction::default(),
            "It's my favorite song.",
        );
    }

    #[test]
    fn allows_its_new() {
        assert_no_lints(
            "The company announced its new product line. ",
            ItsContraction::default(),
        );
    }

    #[test]
    fn allows_its_own_charm() {
        assert_no_lints("The house has its own charm. ", ItsContraction::default());
    }

    #[test]
    fn allows_its_victory() {
        assert_no_lints(
            "The team celebrated its victory. ",
            ItsContraction::default(),
        );
    }

    #[test]
    fn allows_its_history() {
        assert_no_lints(
            "The country is proud of its history. ",
            ItsContraction::default(),
        );
    }

    #[test]
    fn allows_its_secrets() {
        assert_no_lints(
            "The book contains its own secrets. ",
            ItsContraction::default(),
        );
    }
}
