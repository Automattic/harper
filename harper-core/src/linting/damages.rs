use crate::{
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, expr_linter::Chunk, debug::format_lint_match},
    Token, Lint
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
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(&self, toks: &[Token], src: &[char], ctx: Option<(&[Token], &[Token])>) -> Option<Lint> {
        eprintln!("🩵 {}", format_lint_match(toks, ctx, src));
        None
    }

    fn description(&self) -> &str {
        "Checks for plural `damages` not in the context of a court case."
    }
}

#[cfg(test)]
mod tests {



    // Examples of the error from GitHub:

    // Flow networks robust against damages are simple model networks described in a series of publications by Kaluza et al.
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