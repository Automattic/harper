use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct InFavourOfDoing {
    expr: SequenceExpr,
}

impl Default for InFavourOfDoing {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("in")
                .t_ws()
                .t_set(&["favour", "favor"])
                .t_ws()
                .then_verb_progressive_form(),
        }
    }
}

impl ExprLinter for InFavourOfDoing {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        let fav_idx = 2;

        Some(Lint {
            span: toks[fav_idx].span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::InsertAfter(" of".chars().collect())],
            message: "The word `of` is missing.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects missing `of` in `in favor/favour of doing`, etc."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::InFavourOfDoing;

    #[test]
    fn fix_favour_doing() {
        assert_suggestion_result(
            "I don't think anyone is in favour doing nothing, but I wouldn't know which of the first two is best.",
            InFavourOfDoing::default(),
            "I don't think anyone is in favour of doing nothing, but I wouldn't know which of the first two is best.",
        );
    }

    #[test]
    fn fix_favor_getting() {
        assert_suggestion_result(
            "I've definitely been in favor getting Observable as a language primitive since the beginning.",
            InFavourOfDoing::default(),
            "I've definitely been in favor of getting Observable as a language primitive since the beginning.",
        );
    }

    #[test]
    fn fix_favor_having() {
        assert_suggestion_result(
            "I'm in favor having all-in-one @std/std as convenient entrypoint for all modules.",
            InFavourOfDoing::default(),
            "I'm in favor of having all-in-one @std/std as convenient entrypoint for all modules.",
        );
    }

    #[test]
    fn fix_favor_using() {
        assert_suggestion_result(
            "I'm closing this in favor using the official guide.",
            InFavourOfDoing::default(),
            "I'm closing this in favor of using the official guide.",
        );
    }

    #[test]
    fn fix_favour_having() {
        assert_suggestion_result(
            "Would you be in favour having a feature to suppress local configuration files via an env var e.g. DISABLE_LOCAL_CONFIG or similar?",
            InFavourOfDoing::default(),
            "Would you be in favour of having a feature to suppress local configuration files via an env var e.g. DISABLE_LOCAL_CONFIG or similar?",
        );
    }

    #[test]
    fn fix_favour_putting() {
        assert_suggestion_result(
            "I've abandoned just optimising the cpu renderers (which could then just be compiled on linux as well) in favour putting the work onto gpu.",
            InFavourOfDoing::default(),
            "I've abandoned just optimising the cpu renderers (which could then just be compiled on linux as well) in favour of putting the work onto gpu.",
        );
    }

    #[test]
    fn fix_favour_using() {
        assert_suggestion_result(
            "This is also the reason I am personally not in favour using AI for this task because of the latter.",
            InFavourOfDoing::default(),
            "This is also the reason I am personally not in favour of using AI for this task because of the latter.",
        );
    }
}
