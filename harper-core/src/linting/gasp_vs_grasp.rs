use crate::{
    Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::Chunk},
};

pub struct GaspVsGrasp {
    expr: FirstMatchOf,
}

impl Default for GaspVsGrasp {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new(vec![
                Box::new(
                    SequenceExpr::word_set(&["gasp", "gasped", "gasping", "gasps"])
                        .t_ws()
                        .t_aco("at")
                        .t_ws()
                        .t_set(&["straw", "straws"]),
                ),
                Box::new(SequenceExpr::aco("last").t_ws().t_set(&["grasp", "grasps"])),
            ]),
        }
    }
}

fn gasp_at_straws_to_grasp(
    toks: &[Token],
    src: &[char],
    _ctx: Option<(&[Token], &[Token])>,
) -> Option<Lint> {
    let (verb, straw) = (0, 4);
    let verb_span = toks[verb].span;
    let (verb, straw) = (&toks[verb], &toks[straw]);

    let grasp_form = match &verb.kind {
        k if k.is_verb_lemma() => "grasp",
        k if k.is_verb_past_form() => "grasped",
        k if k.is_verb_progressive_form() => "grasping",
        k if k.is_verb_third_person_singular_present_form() => "grasps",
        _ => "grasp",
    };

    let lint_kind = LintKind::Malapropism;
    let suggestions = vec![Suggestion::replace_with_match_case_str(
        grasp_form,
        verb_span.get_content(src),
    )];
    let message = "Did you mean `grasp at straws`?".to_string();

    Some(Lint {
        span: verb_span,
        lint_kind,
        suggestions,
        message,
        ..Default::default()
    })
}

fn last_grasp_to_gasp(
    toks: &[Token],
    src: &[char],
    _ctx: Option<(&[Token], &[Token])>,
) -> Option<Lint> {
    let (last, grasp) = (0, 2);
    let action_span = toks[grasp].span;
    let (last, action) = (&toks[last], &toks[grasp]);

    let lint_kind = LintKind::Malapropism;
    let suggestions = vec![Suggestion::replace_with_match_case_str(
        "gasp",
        action_span.get_content(src),
    )];
    let message = "Did you mean `last gasp`?".to_string();

    Some(Lint {
        span: action_span,
        lint_kind,
        suggestions,
        message,
        ..Default::default()
    })
}

impl ExprLinter for GaspVsGrasp {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));
        (match toks.len() {
            3 => last_grasp_to_gasp,
            5 => gasp_at_straws_to_grasp,
            _ => return None,
        })(toks, src, ctx)
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the wrong word in the idioms `grasp at straws` and `last gasp`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::GaspVsGrasp;

    // Grasp -> gasp

    #[test]
    fn fix_desperate_last_grasp_for_help() {
        assert_suggestion_result(
            "In a desperate last grasp for help, he posted on a forum asking for advice.",
            GaspVsGrasp::default(),
            "In a desperate last gasp for help, he posted on a forum asking for advice.",
        );
    }

    #[test]
    fn fix_communisms_last_grasp() {
        assert_suggestion_result(
            "Intersectionality or whatever communism's last grasp is called this week is going away.",
            GaspVsGrasp::default(),
            "Intersectionality or whatever communism's last gasp is called this week is going away.",
        );
    }

    // Gasp -> grasp

    #[test]
    fn fix_just_gasping_at_straws() {
        assert_suggestion_result(
            "It's not even doing that - we're just gasping at straws here.",
            GaspVsGrasp::default(),
            "It's not even doing that - we're just grasping at straws here.",
        );
    }

    #[test]
    fn fix_truely_gasping_at_straws() {
        assert_suggestion_result(
            "Oh come, this is truely gasping at straws.",
            GaspVsGrasp::default(),
            "Oh come, this is truely grasping at straws.",
        );
    }
}
