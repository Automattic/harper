use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Sentence},
};

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
        let damage_idx = 0;
        let damage_tok = &toks[damage_idx];
        let damage_span = damage_tok.span;
        let damage_chars = damage_span.get_content(src);

        // Singular noun/verb lemma is not an error but during development we'll print uses of it
        //  to observe its context.
        if damage_chars.eq_ignore_ascii_case_chars(&['d', 'a', 'm', 'a', 'g', 'e']) {
            return None;
        }

        let maybe_prev_word = ctx.and_then(|(pre, _)| {
            let last_word = pre.last();
            match (last_word, pre.get(pre.len() - 2)) {
                (Some(sp), Some(w)) if sp.kind.is_whitespace() && w.kind.is_word() => Some(w),
                _ => None,
            }
        });

        #[derive(PartialEq)]
        enum CanPrecede {
            Unknown,
            NeitherNounNorVerb,
            Noun,
            Verb,
            EitherNounOrVerb,
        }

        let can_precede = maybe_prev_word.map_or(CanPrecede::Unknown, |prev_word| {
            let mut can: CanPrecede = CanPrecede::Unknown;
            if prev_word.kind.is_preposition()
                && !prev_word
                    .span
                    .get_content(src)
                    .eq_ignore_ascii_case_chars(&['t', 'o'])
            {
                can = CanPrecede::Noun;
            }

            if prev_word.kind.is_adjective() {
                if prev_word.kind.is_noun() {
                } else {
                    can = CanPrecede::Noun;
                }
            }
            if prev_word.kind.is_determiner() {
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

        // Check all the tokens for words that are used in the legal compesation context
        // TODO: this fails when "damages" is misuses in a diclaimer:
        // 1. "If you encounter any issues, errors, or damages resulting from the use of these templates,
        //     the repository author assumes no responsibility or liability."
        // 2. "The author will not be liable for any losses and/or damages in connection with the use of our website"
        if ctx.is_some_and(|(pre, aft)| {
            let keywords = &[
                "claim",
                "claims",
                "judgments",
                "liabilities",
                "liability",
                "liable",
                "settlements",
                "warranty",
            ][..];
            pre.iter().any(|t| {
                t.span
                    .get_content(src)
                    .eq_any_ignore_ascii_case_str(keywords)
            }) || aft.iter().any(|t| {
                t.span
                    .get_content(src)
                    .eq_any_ignore_ascii_case_str(keywords)
            })
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
    // Examples of the error from GitHub:

    use crate::linting::tests::assert_lint_count;

    #[test]
    fn fix_robust_against_damages() {
        assert_lint_count(
            "Flow networks robust against damages are simple model networks described in a series of publications by Kaluza et al.",
            super::Damages::default(),
            1,
        );
    }

    // POC to select vehicle damages on a car and mark the severity - sudheeshcm/vehicle-damage-selector.
    // This is a web application that detects damages on mangoes using a TensorFlow model with Django as the frontend framework
    // Detecting different types of damages of roads like cracks and potholes for the given image/video of the road.

    // Examples from GitHub where it seems to be used correctly in regard to financial compensation:

    // Code used for calculating damages in lost chance cases.
    // Where the dispute involves a claim for damages in respect of a motor accident for cost of rental of a replacement vehicle
    // Under this section, the Commercial Contributor would have to
    // defend claims against the other Contributors related to those
    // performance claims and warranties, and if a court requires any other
    // Contributor to pay any damages as a result, the Commercial Contributor
    // must pay those damages.

    // Examples from GitHub where it's not an error but a verb:

    // Profiles pb's and damages them when their runtime goes over a set value - sirhamsteralot/HaE-PBLimiter.
    // Opening Wayland-native terminal damages Firefox
    // Open File Requester damages underlaying windows when moved

    // Examples from GitHub that are too hard to call - maybe they are talking about financial compensation?

    // The goal is to estimate the damages of each link in the Graph object using the Damages result (estimating the damages for each segment of a Network).
    // This repository contains code to conduct statistical inference in cartel damages estimation. It will be updated to include a Stata .do file which approximates the standard error of total damages from a fixed effects panel data model, using the delta method.
    // Financial damages caused by received errors $$$$
    // It would be useful to be able to see asset-level damages after running FDA 2.0.
}
