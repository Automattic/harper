use crate::{
    CharStringExt, Dialect, Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{LintKind, Suggestion},
    patterns::{InflectionOfBe, WordSet},
};

use super::{ExprLinter, Lint};
use crate::linting::expr_linter::Chunk;

pub struct InOnTheCards {
    expr: Box<dyn Expr>,
    dialect: Dialect,
}

impl InOnTheCards {
    pub fn new(dialect: Dialect) -> Self {
        // Quick research suggested that Australian and Canadian English agree with American English.
        let preposition = match dialect {
            Dialect::British => "in",
            _ => "on",
        };

        let pre_context = FirstMatchOf::new(vec![
            Box::new(InflectionOfBe::new()),
            Box::new(WordSet::new(&[
                "isn't", "it's", "wasn't", "weren't", "not", "isnt", "its", "wasnt", "werent",
            ])),
        ]);

        let expr = SequenceExpr::with(pre_context)
            .t_ws()
            .t_aco(preposition)
            .then_fixed_phrase(" the cards");

        Self {
            expr: Box::new(expr),
            dialect,
        }
    }
}

impl ExprLinter for InOnTheCards {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let prep_span = toks[2].span;
        let prep = prep_span.get_content(src);

        let new_prep = [
            match prep[0] {
                'i' => 'o',
                'o' => 'i',
                'I' => 'O',
                'O' => 'I',
                _ => return None,
            },
            prep[1],
        ];

        let sugg = Suggestion::ReplaceWith(new_prep.to_vec());

        let message = format!(
            "Use `{} the cards` instead of `{} the cards` in {} English.",
            new_prep.to_string(),
            prep.to_string(),
            self.dialect,
        );

        Some(Lint {
            span: prep_span,
            lint_kind: LintKind::Regionalism,
            suggestions: vec![sugg],
            message,
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Corrects either `in the cards` or `on the cards` to the other, depending on the dialect."
    }
}

#[cfg(test)]
mod tests {
    use super::InOnTheCards;
    use crate::{
        Dialect,
        linting::tests::{assert_lint_count, assert_suggestion_result},
    };

    // On the cards

    #[test]
    fn correct_are_on_for_american() {
        assert_suggestion_result(
            "Both these features are on the cards, but for now we want to let users know if they have requested an invalid example.",
            InOnTheCards::new(Dialect::American),
            "Both these features are in the cards, but for now we want to let users know if they have requested an invalid example.",
        );
    }

    #[test]
    fn dont_correct_is_on_for_british() {
        assert_lint_count(
            "Yes, I think this is on the cards.",
            InOnTheCards::new(Dialect::British),
            0,
        );
    }

    #[test]
    fn correct_not_on_for_american() {
        assert_suggestion_result(
            "If a permanent unique identifier is not on the cards any time soon for WebHID, we should consider a WebUSB alternative.",
            InOnTheCards::new(Dialect::American),
            "If a permanent unique identifier is not in the cards any time soon for WebHID, we should consider a WebUSB alternative.",
        );
    }

    #[test]
    fn correct_be_on_for_american() {
        assert_suggestion_result(
            "a full breach of genomics (patient?) data can be on the cards since S3 AWS bucket credentials can be slurped from the process's memory",
            InOnTheCards::new(Dialect::American),
            "a full breach of genomics (patient?) data can be in the cards since S3 AWS bucket credentials can be slurped from the process's memory",
        );
    }

    #[test]
    fn correct_was_on_for_american() {
        assert_suggestion_result(
            "Virtualising the message summaries ObservableCollection was on the cards so I also take note of your last point.",
            InOnTheCards::new(Dialect::American),
            "Virtualising the message summaries ObservableCollection was in the cards so I also take note of your last point.",
        );
    }

    #[test]
    fn correct_isnt_on_no_apostrophe_for_american() {
        assert_suggestion_result(
            "parallelising that part isnt on the cards since there would be no noticeable ...",
            InOnTheCards::new(Dialect::American),
            "parallelising that part isnt in the cards since there would be no noticeable ...",
        );
    }

    #[test]
    fn correct_its_on_for_american() {
        assert_suggestion_result(
            "Regarding extensive documentation, as mentioned, its on the cards, project being sponsored by the aforementioned organisations.",
            InOnTheCards::new(Dialect::American),
            "Regarding extensive documentation, as mentioned, its in the cards, project being sponsored by the aforementioned organisations.",
        );
    }

    #[test]
    fn correct_were_on_for_american() {
        assert_suggestion_result(
            "lots of high altitudes were on the cards again",
            InOnTheCards::new(Dialect::American),
            "lots of high altitudes were in the cards again",
        );
    }

    #[test]
    fn correct_isnt_on_for_american() {
        assert_suggestion_result(
            "downgrading to an end-of-life operating system isn't on the cards",
            InOnTheCards::new(Dialect::American),
            "downgrading to an end-of-life operating system isn't in the cards",
        );
    }

    #[test]
    fn correct_wasnt_on_for_american() {
        assert_suggestion_result(
            "it's only a middleground for an org because passwordless wasn't on the cards previously",
            InOnTheCards::new(Dialect::American),
            "it's only a middleground for an org because passwordless wasn't in the cards previously",
        );
    }

    // In the cards

    #[test]
    fn correct_was_in_for_british() {
        assert_suggestion_result(
            "Just wondering if it was in the cards or not for something like the Quest3 to get support in the future.",
            InOnTheCards::new(Dialect::British),
            "Just wondering if it was on the cards or not for something like the Quest3 to get support in the future.",
        );
    }

    #[test]
    fn dont_correct_is_in_for_american() {
        assert_lint_count(
            "Not sure if such a project is in the cards",
            InOnTheCards::new(Dialect::American),
            0,
        );
    }

    #[test]
    fn correct_not_in_for_british() {
        assert_suggestion_result(
            "Is that just not in the cards for WASM at this time?",
            InOnTheCards::new(Dialect::British),
            "Is that just not on the cards for WASM at this time?",
        );
    }

    #[test]
    fn correct_be_in_for_british() {
        assert_suggestion_result(
            "Would this be in the cards?",
            InOnTheCards::new(Dialect::British),
            "Would this be on the cards?",
        );
    }

    #[test]
    fn correct_are_in_for_british() {
        assert_suggestion_result(
            "Manifest files are in the cards but haven't been implemented yet.",
            InOnTheCards::new(Dialect::British),
            "Manifest files are on the cards but haven't been implemented yet.",
        );
    }

    #[test]
    fn correct_its_in_for_british() {
        assert_suggestion_result(
            "As far as an error, that probably would be helpful but doesn't sound like its in the cards.",
            InOnTheCards::new(Dialect::British),
            "As far as an error, that probably would be helpful but doesn't sound like its on the cards.",
        );
    }

    #[test]
    fn correct_were_in_for_british() {
        assert_suggestion_result(
            "a year or two given the major overhauls that were in the cards at the time",
            InOnTheCards::new(Dialect::British),
            "a year or two given the major overhauls that were on the cards at the time",
        );
    }

    #[test]
    fn correct_isnt_in_for_british() {
        assert_suggestion_result(
            "I'm going to close this as opting out of the installation framework that Electron gives us isn't in the cards for the project at this time.",
            InOnTheCards::new(Dialect::British),
            "I'm going to close this as opting out of the installation framework that Electron gives us isn't on the cards for the project at this time.",
        );
    }

    #[test]
    fn correct_wasnt_in_for_british() {
        assert_suggestion_result(
            "doing something better than just swapping our internal log package for glog wasn’t in the cards back then",
            InOnTheCards::new(Dialect::British),
            "doing something better than just swapping our internal log package for glog wasn’t on the cards back then",
        );
    }

    #[test]
    fn correct_werent_in_for_british() {
        assert_suggestion_result(
            "I had thought stacked borrows was mostly in a final tweaking phase and major changes weren't in the cards.",
            InOnTheCards::new(Dialect::British),
            "I had thought stacked borrows was mostly in a final tweaking phase and major changes weren't on the cards.",
        );
    }
}
