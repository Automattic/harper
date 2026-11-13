use crate::{
    CharStringExt, Lint, Lrc, Token,
    expr::{All, Expr, FirstMatchOf, OwnedExprExt, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
    patterns::WordSet,
};

pub struct RegimenRegiment {
    expr: All,
}

const WORDS_BEFORE_REGIMEN: &[&str] = &[
    "antibiotic",
    "antiphlogistic",
    "chemo",
    "chemotherapeutic",
    "chemotherapy",
    "conditioning",
    "dosage",
    "dose",
    "dosing",
    "drug",
    "metastatic",
    "patient's",
    "patients",
    "therapeutic",
    "treating",
    "treatment",
];

const WORDS_AFTER_REGIMEN_WITH: &[&str] = &["autologous", "hepatitis"];

const WORDS_BEFORE_REGIMENT: &[&str] = &[
    "artillery",
    "cavalry",
    "engineering",
    "infantry",
    "parachute",
];

const WORDS_AFTER_REGIMENT_FOR: &[&str] = &["duty", "foreign", "service"];

const REGIMENT_SG_PL: &[&str] = &["regiment", "regiments"];

impl Default for RegimenRegiment {
    fn default() -> Self {
        let regimen_sgpl = Lrc::new(WordSet::new(REGIMENT_SG_PL));

        Self {
            // Using "regimen" mistakenly for "regiment" is very rare.
            // But using "regiment" mistakenly for "regimen" is quite common.
            // Check for "regimen(s)" with any of "regiment(s)" collocations
            expr: SequenceExpr::any_of([
                Box::new(
                    SequenceExpr::word_set(WORDS_BEFORE_REGIMEN)
                        .t_ws()
                        .then(regimen_sgpl.clone()),
                ),
                Box::new(
                    SequenceExpr::with(regimen_sgpl.clone())
                        .t_ws()
                        .t_aco("with")
                        .t_ws()
                        .t_set(WORDS_AFTER_REGIMEN_WITH),
                ),
            ])
            .but_not(FirstMatchOf::new([
                Box::new(
                    SequenceExpr::word_set(WORDS_BEFORE_REGIMENT)
                        .t_ws()
                        .then(regimen_sgpl.clone()),
                ),
                Box::new(
                    SequenceExpr::with(regimen_sgpl.clone())
                        .t_ws()
                        .t_aco("for")
                        .t_ws()
                        .t_set(WORDS_AFTER_REGIMENT_FOR),
                ),
            ])),
        }
    }
}

impl ExprLinter for RegimenRegiment {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = find_the_only_token_matching(toks, src, |t, _| {
            t.get_ch(src).eq_any_ignore_ascii_case_str(REGIMENT_SG_PL)
        })?
        .span;

        let mut correction = span.get_content(src).to_vec();
        let last_letter = correction.pop()?;

        match last_letter {
            's' | 'S' => {
                _ = correction.pop();
                correction.push(last_letter);
            }
            't' | 'T' => {}
            _ => return None,
        }

        let suggestions = vec![Suggestion::ReplaceWith(correction)];

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "Did you mean `regimen` (a routine)? A `regiment` is a military unit."
                .to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects mistaken use of `regiment` (military unit) when `regimen` (routine) was intended."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::RegimenRegiment;

    #[test]
    fn fix_chemotherapeutic_regiments_caps() {
        assert_suggestion_result(
            "ANALYSIS OF CHEMOTHERAPEUTIC REGIMENTS FOR THE MANAGEMENT OF PTLD.",
            RegimenRegiment::default(),
            "ANALYSIS OF CHEMOTHERAPEUTIC REGIMENS FOR THE MANAGEMENT OF PTLD.",
        );
    }

    #[test]
    fn fix_chemotherapy_regiments() {
        assert_suggestion_result(
            "ICER was counted to summarize the cost-effectiveness of two chemotherapy regiments (Taxane versus FAC).",
            RegimenRegiment::default(),
            "ICER was counted to summarize the cost-effectiveness of two chemotherapy regimens (Taxane versus FAC).",
        );
    }

    #[test]
    fn fix_chemotherapy_regiment() {
        assert_suggestion_result(
            "Comparison of four chemotherapy regiment for advanced nonsmall cell lung cancer.",
            RegimenRegiment::default(),
            "Comparison of four chemotherapy regimen for advanced nonsmall cell lung cancer.",
        );
    }

    #[test]
    fn fix_dose_regiment() {
        assert_suggestion_result(
            "P110-4 Low compared to high dose regiment of induction chemotherapy in nasopharingeal cancer: a systematic review meta-analysis.",
            RegimenRegiment::default(),
            "P110-4 Low compared to high dose regimen of induction chemotherapy in nasopharingeal cancer: a systematic review meta-analysis.",
        );
    }

    #[test]
    fn fix_dosing_regiment() {
        assert_suggestion_result(
            "Dr Lisette Rozeman speaks with ecancer at ESMO 2018 in Munich about the study into a more tolerable dosing regiment of the combination of ...",
            RegimenRegiment::default(),
            "Dr Lisette Rozeman speaks with ecancer at ESMO 2018 in Munich about the study into a more tolerable dosing regimen of the combination of ...",
        );
    }

    #[test]
    fn fix_drug_regiment() {
        assert_suggestion_result(
            "Three drug regiment (Irinotecan, Carboplatin and Etoposide) as a front line treatment in extensive disease small-cell lung cancer: phase II trial.",
            RegimenRegiment::default(),
            "Three drug regimen (Irinotecan, Carboplatin and Etoposide) as a front line treatment in extensive disease small-cell lung cancer: phase II trial.",
        );
    }

    #[test]
    fn fix_therapeutic_regiment() {
        assert_suggestion_result(
            "A Triple Therapeutic Regiment Consisted of Colchicine, Thalidomide and Total Glucosides of Paeony Is Effective and Well‐Tolerated for",
            RegimenRegiment::default(),
            "A Triple Therapeutic Regimen Consisted of Colchicine, Thalidomide and Total Glucosides of Paeony Is Effective and Well‐Tolerated for",
        );
    }

    // Avoid false positives

    #[test]
    fn dont_flag_artillery_regiments() {
        assert_no_lints(
            "Artillery Regiments with Minden associations (see below) also wear red roses.",
            RegimenRegiment::default(),
        );
    }

    #[test]
    fn dont_flag_cavalry_regiments() {
        assert_no_lints(
            "Army—Cavalry Regiments For India—Question",
            RegimenRegiment::default(),
        );
    }
}
