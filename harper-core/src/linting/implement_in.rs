use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ImplementIn {
    expr: SequenceExpr,
}

impl Default for ImplementIn {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(&[
                "implement",
                "implemented",
                "implementing",
                "implements",
            ])
            .t_ws()
            // Note: I would use .then_optional() here but see #3958
            .then_any_of([
                Box::new(SequenceExpr::aco("into")),
                Box::new(SequenceExpr::any_word().t_ws().t_aco("into")),
            ]),
        }
    }
}

impl ExprLinter for ImplementIn {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (vtok, midtok, itok) = match toks.len() {
            3 => (&toks[0], None, &toks[2]),
            5 => (&toks[0], Some(&toks[2]), &toks[4]),
            _ => return None,
        };

        // "Implement" and "implements" are nouns as well as verbs.
        // Check for either followed by a verb.
        // brought/came/come/coming/fall/fell/imported/introduced/going/put into are common.
        if vtok.kind.is_noun() && midtok.is_some_and(|t| t.kind.is_verb() && !t.kind.is_noun()) {
            return None;
        }

        // If "Into" is in title case and the middle word isn't there or is "the"
        //   it's probably the rust trait.
        if (midtok.is_none() || midtok.is_some_and(|t| t.get_ch(src).eq_ch(&['t', 'h', 'e'])))
            && itok.get_ch(src) == &['I', 'n', 't', 'o']
        {
            return None;
        }

        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "in",
                span.get_content(src),
            )],
            message: "Prefer `in` over `into` in this usage.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects nonstandard `implement into` to `implement in`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ImplementIn;

    #[test]
    fn fix_implement_something_lowercase() {
        assert_suggestion_result(
            "How to implement overlay into your code?",
            ImplementIn::default(),
            "How to implement overlay in your code?",
        );
    }

    #[test]
    fn fix_implement_something_titlecase() {
        assert_suggestion_result(
            "Implement Connectors into the Testbed",
            ImplementIn::default(),
            "Implement Connectors in the Testbed",
        );
    }

    #[test]
    fn fix_implement_something() {
        assert_suggestion_result(
            "So why not implement it into links or w3m?",
            ImplementIn::default(),
            "So why not implement it in links or w3m?",
        );
    }

    #[test]
    fn fix_implement_into() {
        assert_suggestion_result(
            "Use ngx-daterangepicker-material as Storybook Angular (Issue about Ranges Clicked and Dates Updated) when implement into other project",
            ImplementIn::default(),
            "Use ngx-daterangepicker-material as Storybook Angular (Issue about Ranges Clicked and Dates Updated) when implement in other project",
        );
    }

    #[test]
    fn fix_implemented_into() {
        assert_suggestion_result(
            "Demo of MCTS AI implemented into a platformer with a level editor.",
            ImplementIn::default(),
            "Demo of MCTS AI implemented in a platformer with a level editor.",
        );
    }

    #[test]
    fn fix_implemented_something() {
        assert_suggestion_result(
            "Trained and implemented autoencoder into a pipeline with Google Cloud transcription API",
            ImplementIn::default(),
            "Trained and implemented autoencoder in a pipeline with Google Cloud transcription API",
        );
    }

    #[test]
    fn fix_implementing_something() {
        assert_suggestion_result(
            "Problem with implementing library into the Vue.js webpack-based project",
            ImplementIn::default(),
            "Problem with implementing library in the Vue.js webpack-based project",
        );
    }

    #[test]
    fn fix_implementing_something_title_case() {
        assert_suggestion_result(
            "Implementing breadcrumbs into site with AngularJS v1.2.16.",
            ImplementIn::default(),
            "Implementing breadcrumbs in site with AngularJS v1.2.16.",
        );
    }

    #[test]
    fn fix_implements_something() {
        assert_suggestion_result(
            "A plugin that implements latex into discord.",
            ImplementIn::default(),
            "A plugin that implements latex in discord.",
        );
    }

    #[test]
    fn dont_flag_implements_into_trait() {
        assert_no_lints(
            "I suppose there is a concern that somebody may define a struct which implements Into",
            ImplementIn::default(),
        );
    }

    #[test]
    fn dont_flag_implementing_into_trait() {
        assert_no_lints(
            "There might still be cases where one cannot implement From but implementing Into is possible.",
            ImplementIn::default(),
        );
    }

    #[test]
    fn dont_flag_implementing_the_into_trait() {
        assert_no_lints(
            "I'm having trouble implementing the Into trait for a generic struct in Rust.",
            ImplementIn::default(),
        );
    }
}
