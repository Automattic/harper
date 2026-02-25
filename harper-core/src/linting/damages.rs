use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Sentence},
};

static KEYWORDS: &[&str] = &[
    "case",
    "cases",
    "claim",
    "claims",
    "judgment",
    "judgments",
    "liabilities",
    "liability",
    "liable",
    "settlement",
    "settlements",
    "warranty",
];

pub struct Damages {
    expr: Box<dyn Expr>,
}

impl Default for Damages {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::word_set(&["damages", "damage"])),
        }
    }
}

impl ExprLinter for Damages {
    type Unit = Sentence;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let (pretoks, postoks) = ctx?;
        let damage_idx = 0;
        let damage_tok = &toks[damage_idx];
        let damage_span = damage_tok.span;
        let damage_chars = damage_span.get_content(src);

        // Singular noun/verb lemma is not an error but during development we'll print uses of it
        //  to observe its context.
        if damage_chars.eq_ignore_ascii_case_chars(&['d', 'a', 'm', 'a', 'g', 'e']) {
            return None;
        }

        // If the word after "damages" is a noun or object pronoun, it's the object and "damages" is a verb.
        let next_word_tok = match (postoks.first(), postoks.get(1)) {
            (Some(sp), Some(w)) if sp.kind.is_whitespace() && w.kind.is_word() => Some(w),
            _ => None,
        };

        if next_word_tok.is_some_and(|nwt| nwt.kind.is_object_pronoun() || nwt.kind.is_noun()) {
            return None;
        }

        // The word before "damages" may help us narrow down whether it's a noun or verb.
        let prev_word_tok = match (pretoks.get(pretoks.len() - 2), pretoks.last()) {
            (Some(w), Some(sp)) if sp.kind.is_whitespace() && w.kind.is_word() => Some(w),
            _ => None,
        };

        #[derive(PartialEq)]
        enum CanPrecede {
            Unknown,
            NeitherNounNorVerb,
            Noun,
            Verb,
            EitherNounOrVerb,
        }

        // Try to disambiguate whether "damages" is a noun or verb.
        let can_precede = prev_word_tok.map_or(CanPrecede::Unknown, |prev_word| {
            let mut can: CanPrecede = CanPrecede::Unknown;

            if (prev_word.kind.is_adjective()
                || prev_word.kind.is_determiner()
                || prev_word.kind.is_preposition())
                && !prev_word
                    .span
                    .get_content(src)
                    .eq_ignore_ascii_case_chars(&['t', 'o'])
            {
                can = CanPrecede::Noun;
            }

            if prev_word.kind.is_auxiliary_verb() {
                can = if can == CanPrecede::Noun {
                    CanPrecede::EitherNounOrVerb
                } else {
                    CanPrecede::Verb
                };
            }

            can
        });

        if can_precede == CanPrecede::Verb {
            return None;
        }

        // We now know "damages" isn't unambiguously a verb, but it could still be an ambiguous verb-noun.
        // Or it could be a noun. Or it could still be unknown.

        // Check if it's the object of the verb "to pay"
        let pay_det = SequenceExpr::word_set(&["paid", "pay", "paying", "pays"])
            .then_optional(SequenceExpr::default().t_ws().then_determiner())
            .t_ws();

        if pretoks
            .windows(2)
            .enumerate()
            .rev()
            .take_while(|(i, _)| pay_det.run(*i, pretoks, src).is_none())
            .count()
            < pretoks.len() / 2
        {
            return None;
        }

        // Check all the tokens for words that are used in the legal compesation context
        // TODO: this fails when "damages" is misuses in a diclaimer:
        // 1. "If you encounter any issues, errors, or damages resulting from the use of these templates,
        //     the repository author assumes no responsibility or liability."
        // 2. "The author will not be liable for any losses and/or damages in connection with the use of our website"
        if pretoks.iter().any(|t| {
            t.span
                .get_content(src)
                .eq_any_ignore_ascii_case_str(KEYWORDS)
        }) || postoks.iter().any(|t| {
            t.span
                .get_content(src)
                .eq_any_ignore_ascii_case_str(KEYWORDS)
        }) {
            return None;
        }

        Some(Lint {
            span: damage_span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                damage_chars[..6].to_vec(),
                damage_chars,
            )],
            message: "Singular `damage` is correct when not refering to a court case.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Checks for plural `damages` not in the context of a court case."
    }
}

#[cfg(test)]
mod tests {
    use super::Damages;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    // Examples of the error from GitHub:

    #[test]
    fn fix_robust_against_damages_by_prev_preposition() {
        assert_suggestion_result(
            "Flow networks robust against damages are simple model networks described in a series of publications by Kaluza et al.",
            Damages::default(),
            "Flow networks robust against damage are simple model networks described in a series of publications by Kaluza et al.",
        );
    }

    #[test]
    fn fix_vehicle_damages_on_a_car_by_fall_through() {
        assert_suggestion_result(
            "POC to select vehicle damages on a car and mark the severity - sudheeshcm/vehicle-damage-selector.",
            Damages::default(),
            "POC to select vehicle damage on a car and mark the severity - sudheeshcm/vehicle-damage-selector.",
        );
    }

    #[test]
    fn fix_damages_on_mangoes() {
        assert_suggestion_result(
            "This is a web application that detects damages on mangoes using a TensorFlow model with Django as the frontend framework",
            Damages::default(),
            "This is a web application that detects damage on mangoes using a TensorFlow model with Django as the frontend framework",
        );
    }

    #[test]
    fn fix_types_of_damages_of_roads() {
        assert_suggestion_result(
            "Detecting different types of damages of roads like cracks and potholes for the given image/video of the road.",
            Damages::default(),
            "Detecting different types of damage of roads like cracks and potholes for the given image/video of the road.",
        );
    }

    // Examples from GitHub where it seems to be used correctly in regard to financial compensation:

    // TODO: would the word "calculate" before "damages" be a good heuristic?
    #[test]
    fn ignore_damages_in_lost_chance_cases() {
        assert_no_lints(
            "Code used for calculating damages in lost chance cases.",
            Damages::default(),
        );
    }

    #[test]
    fn ignore_claim_for_damages() {
        assert_no_lints(
            "Where the dispute involves a claim for damages in respect of a motor accident for cost of rental of a replacement vehicle",
            Damages::default(),
        );
    }

    #[test]
    fn ignore_pay_damages() {
        assert_no_lints(
            "Under this section, the Commercial Contributor would have to
            defend claims against the other Contributors related to those
            performance claims and warranties, and if a court requires any other
            Contributor to pay any damages as a result, the Commercial Contributor
            must pay those damages.",
            Damages::default(),
        );
    }

    // Examples from GitHub where it's not an error but a verb:

    #[test]
    fn ignore_damages_them() {
        assert_no_lints(
            "Profiles pb's and damages them when their runtime goes over a set value - sirhamsteralot/HaE-PBLimiter.",
            Damages::default(),
        );
    }

    #[test]
    fn ignore_damages_firefox() {
        assert_no_lints(
            "Opening Wayland-native terminal damages Firefox",
            Damages::default(),
        );
    }

    #[test]
    fn ignore_damages_underlaying_windows() {
        assert_no_lints(
            "Open File Requester damages underlaying windows when moved",
            Damages::default(),
        );
    }

    // Examples from GitHub that are too hard to call - maybe they are talking about financial compensation?

    #[test]
    #[ignore = "too close to call for now"]
    fn ignore_estimate_the_damages_and_the_damages_result() {
        assert_no_lints(
            "The goal is to estimate the damages of each link in the Graph object using the Damages result (estimating the damages for each segment of a Network).",
            Damages::default(),
        );
    }

    // https://github.com › dpasmat › cartel-damages-inference
    #[test]
    #[ignore = "too close to call for now"]
    fn ignore_damages_inference() {
        assert_no_lints(
            "This repository contains code to conduct statistical inference in cartel damages estimation. It will be updated to include a Stata .do file which approximates the standard error of total damages from a fixed effects panel data model, using the delta method.",
            Damages::default(),
        );
    }

    #[test]
    #[ignore = "too close to call for now"]
    fn ignore_received_errors() {
        assert_no_lints(
            "Financial damages caused by received errors $$$$.",
            Damages::default(),
        );
    }

    #[test]
    #[ignore = "too close to call for now"]
    fn ignore_asset_level_damages() {
        assert_no_lints(
            "It would be useful to be able to see asset-level damages after running FDA 2.0.",
            Damages::default(),
        );
    }
}
