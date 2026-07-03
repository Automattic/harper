use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_word},
    },
    patterns::{RelativePronoun, SingleTokenPattern},
};

pub struct NoLongerPronoun {
    expr: SequenceExpr,
}

impl Default for NoLongerPronoun {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("no")
                .t_ws()
                .t_aco("longer")
                .t_ws()
                .then_subject_pronoun()
                .then_optional(SequenceExpr::whitespace().t_set(&["am", "are", "is"])),
        }
    }
}

impl ExprLinter for NoLongerPronoun {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let no_longer = &toks[..3];
        let sp = &toks[3..=3];
        let pron_etc = &toks[4..];

        if followed_by_word(ctx, |t| RelativePronoun::default().matches_token(t, src)) {
            return None;
        }

        let correction: Vec<char> = pron_etc
            .iter()
            .chain(sp.iter())
            .chain(no_longer.iter())
            .flat_map(|tok| tok.get_ch(src))
            .copied()
            .collect();

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::WordOrder,
            // NOTE: `replace_with_match_case` messes up with `I` due to being index-based
            suggestions: vec![Suggestion::ReplaceWith(correction)],
            message: "Consider moving the pronoun to before `no longer`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Detects incorrect word order where `no longer` incorrectly precedes a subject pronoun."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::NoLongerPronoun;

    #[test]
    fn fix_no_longer_you_need_to() {
        assert_suggestion_result(
            "It works great for poetry and no longer you need to create keys on pypi",
            NoLongerPronoun::default(),
            "It works great for poetry and you no longer need to create keys on pypi",
        )
    }

    #[test]
    fn fix_no_longer_it() {
        assert_suggestion_result(
            "does that mean that i have make myself an intranet library and no longer it automatically creates a library with my username?",
            NoLongerPronoun::default(),
            "does that mean that i have make myself an intranet library and it no longer automatically creates a library with my username?",
        )
    }

    #[test]
    fn fix_no_longer_i_am_able_verify() {
        assert_suggestion_result(
            "the hash value got replaced by the app's name and no longer I am able to verify OTP automatically",
            NoLongerPronoun::default(),
            "the hash value got replaced by the app's name and I am no longer able to verify OTP automatically",
        );
    }

    #[test]
    fn fix_no_longer_i_am_able_see() {
        assert_suggestion_result(
            "there is also 'effect' to be applied, but no longer I am able to see and control their colors",
            NoLongerPronoun::default(),
            "there is also 'effect' to be applied, but I am no longer able to see and control their colors",
        );
    }

    #[test]
    #[ignore = "I think the word 'even' being in the wrong place here is a separate mistake"]
    fn no_longer_we_are_interested() {
        assert_suggestion_result(
            "I do not also care about what you think and even no longer we are interested to demo tamota buggy feature for our class",
            NoLongerPronoun::default(),
            "I do not also care about what you think and we are no longer even interested to demo tamota buggy feature for our class",
        );
    }

    // Potential false positives

    #[test]
    fn allow_i_who() {
        assert_no_lints(
            "It is no longer I who live, but Christ who lives in me",
            NoLongerPronoun::default(),
        );
    }

    #[test]
    #[ignore = "If this turns out to be common, checking for a modal verb before or 'to' after might work"]
    fn different_mistake_missing_word() {
        assert_no_lints(
            "If I understand this change correctly this change would no longer you to duplicate materials or make them unique to pass in unique parameters",
            NoLongerPronoun::default(),
        )
    }
}
