use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, preceded_by_word},
    },
};

pub struct OffLimits {
    expr: SequenceExpr,
}

impl Default for OffLimits {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("off").t_ws_h().t_aco("limit"),
        }
    }
}

impl ExprLinter for OffLimits {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if preceded_by_word(ctx, |t| {
            t.get_ch(src).eq_any_ignore_ascii_case_str(&[
                "back",
                "backed",
                "backing",
                "backs",
                "switch",
                "switched",
                "switching",
                "switches",
                "turn",
                "turned",
                "turning",
                "turns",
            ])
        }) {
            return None;
        }

        toks.last().map(|t| {
            let span = t.span;
            let ch = span.get_content(src);

            Lint {
                span,
                lint_kind: LintKind::Usage,
                suggestions: vec![Suggestion::ReplaceWith(
                    ch.iter()
                        .copied()
                        .chain(std::iter::once(((ch[ch.len() - 1] as u8) - 1) as char))
                        .collect(),
                )],
                message: "This idiom always uses the plural form.".to_owned(),
                ..Default::default()
            }
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `off-limit` to `off-limits`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::OffLimits;

    #[test]
    fn fix_no_off_limit_contacts() {
        assert_suggestion_result(
            "no task duration bonus (no successful insertion), no force penalty, and no off-limit contacts",
            OffLimits::default(),
            "no task duration bonus (no successful insertion), no force penalty, and no off-limits contacts",
        );
    }

    #[test]
    fn fix_any_off_limit_values() {
        assert_suggestion_result(
            "create html-pages, and optionally warn contacts about any off-limit values",
            OffLimits::default(),
            "create html-pages, and optionally warn contacts about any off-limits values",
        );
    }

    #[test]
    fn fix_in_two_places() {
        assert_suggestion_result(
            "\"off-limit airspace\": no-fly zones. a target is off-limit if it belongs to one of these classes.",
            OffLimits::default(),
            "\"off-limits airspace\": no-fly zones. a target is off-limits if it belongs to one of these classes.",
        );
    }

    #[test]
    fn fix_is_off_limit_space() {
        assert_suggestion_result(
            "one can have this issue if one of the joint position is off limit",
            OffLimits::default(),
            "one can have this issue if one of the joint position is off limits",
        );
    }

    #[test]
    fn dont_flag_back_off() {
        assert_no_lints(
            "In fact, setting it to Never is the only way right now to have a back off limit be applied and effective.",
            OffLimits::default(),
        );
    }

    #[test]
    fn dont_flag_try_backing_off() {
        assert_no_lints(
            "try backing off limit screw another quarter turn",
            OffLimits::default(),
        );
    }

    #[test]
    fn dont_flag_backs_off() {
        assert_no_lints(
            "Backs off limit when system is stressed, increases when healthy.",
            OffLimits::default(),
        );
    }

    #[test]
    fn dont_flag_turn_off() {
        assert_no_lints("Turn off LIMIT and OFFSET?", OffLimits::default());
    }

    #[test]
    fn dont_flag_turned_off() {
        assert_no_lints(
            "I have turned off limit child processes in developer settings.",
            OffLimits::default(),
        );
    }

    #[test]
    fn dont_flag_turning_off() {
        assert_no_lints(
            "should address turning off limit scaling for pods, which would address your immediate problem.",
            OffLimits::default(),
        );
    }

    #[test]
    fn dont_flag_turns_off_limit() {
        assert_no_lints("LIMIT=-1 turns off limit checking.", OffLimits::default());
    }

    #[test]
    fn dont_flag_switch_off() {
        assert_no_lints("Switch off limit exceeded Info(REAL)", OffLimits::default());
    }

    #[test]
    fn dont_flag_switched_off() {
        assert_no_lints(
            "Switched off Limit IP Address Tracking.",
            OffLimits::default(),
        );
    }

    #[test]
    fn dont_flag_switching_off() {
        assert_no_lints(
            "You can do that by switching off Limit IP Address tracking on that specific WiFi network.",
            OffLimits::default(),
        );
    }
}
