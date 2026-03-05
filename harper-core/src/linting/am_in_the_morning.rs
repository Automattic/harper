use crate::linting::expr_linter::Chunk;
use crate::{
    Span, Token, TokenStringExt,
    expr::{Expr, FixedPhrase, LongestMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct AmInTheMorning {
    expr: Box<dyn Expr>,
}

impl Default for AmInTheMorning {
    fn default() -> Self {
        let am = WordSet::new(&["am", "a.m."]);
        let pm = WordSet::new(&["pm", "p.m."]);

        let maybe_ws_am = LongestMatchOf::new(vec![
            Box::new(SequenceExpr::with(am.clone())),
            Box::new(SequenceExpr::whitespace().then(am)),
        ]);
        let maybe_ws_pm = LongestMatchOf::new(vec![
            Box::new(SequenceExpr::with(pm.clone())),
            Box::new(SequenceExpr::whitespace().then(pm)),
        ]);

        let ws_in_periods = SequenceExpr::fixed_phrase(" in the ").then_word_set(&[
            "morning",
            "afternoon",
            "evening",
            "night",
        ]);

        let ws_at_periods = FixedPhrase::from_phrase(" at night");

        let expr = SequenceExpr::any_of(vec![Box::new(maybe_ws_am), Box::new(maybe_ws_pm)])
            .then_any_of(vec![Box::new(ws_in_periods), Box::new(ws_at_periods)]);

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for AmInTheMorning {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let all_after_number_span = toks[0..].span()?;
        let am_pm_idx = if toks[0].kind.is_whitespace() { 1 } else { 0 };

        let maybe_ws_am_pm_span = Span::new(toks[0].span.start, toks[am_pm_idx].span.end);
        let sugg_am_pm_only =
            Suggestion::ReplaceWith(maybe_ws_am_pm_span.get_content(src).to_vec());

        let ws_prep_period = Span::new(toks[am_pm_idx + 1].span.start, all_after_number_span.end);
        let sugg_prep_period_only =
            Suggestion::ReplaceWith(ws_prep_period.get_content(src).to_vec());

        Some(Lint {
            span: all_after_number_span,
            lint_kind: LintKind::Redundancy,
            suggestions: vec![sugg_am_pm_only, sugg_prep_period_only],
            message: "The time period is redundant because it repeats what's already specified by the AM/PM indicator.".to_owned(),            priority: 50,
        })
    }

    fn description(&self) -> &'static str {
        "Finds redundant am/pm indicators used together with time periods such as 'in the morning' or 'at night'."
    }
}

#[cfg(test)]
mod tests {
    use super::AmInTheMorning;
    use crate::linting::tests::{
        assert_lint_count, assert_nth_suggestion_result, assert_suggestion_result,
    };

    #[test]
    fn flag_at_4am_in_the_morning() {
        assert_lint_count("At 4am in the morning", AmInTheMorning::default(), 1);
    }

    #[test]
    fn fix_at_4am_in_the_morning() {
        assert_suggestion_result("At 4am in the morning", AmInTheMorning::default(), "At 4am");
    }

    #[test]
    fn flag_at_4_am_in_the_morning() {
        assert_lint_count("At 4 am in the morning", AmInTheMorning::default(), 1);
    }

    #[test]
    fn fix_at_4_am_in_the_morning() {
        assert_suggestion_result(
            "At 4 am in the morning",
            AmInTheMorning::default(),
            "At 4 am",
        );
    }

    #[test]
    fn flag_at_4am_in_the_morning_caps() {
        assert_lint_count("At 4AM in the morning", AmInTheMorning::default(), 1);
    }

    #[test]
    fn fix_at_4am_in_the_morning_caps() {
        assert_suggestion_result("At 4AM in the morning", AmInTheMorning::default(), "At 4AM");
    }

    #[test]
    fn flag_at_4_am_in_the_morning_caps() {
        assert_lint_count("At 4 AM in the morning", AmInTheMorning::default(), 1);
    }

    #[test]
    fn fix_at_4_am_in_the_morning_caps() {
        assert_suggestion_result(
            "At 4 AM in the morning",
            AmInTheMorning::default(),
            "At 4 AM",
        );
    }

    #[test]
    fn at_4_a_dot_m_dot_in_the_morning() {
        assert_lint_count("At 4 a.m. in the morning", AmInTheMorning::default(), 1)
    }

    #[test]
    fn fix_at_4_a_dot_m_dot_in_the_morning() {
        assert_suggestion_result(
            "At 4 a.m. in the morning",
            AmInTheMorning::default(),
            "At 4 a.m.",
        );
    }

    // real-world examples

    #[test]
    fn fix_real_world_1_am_in_the_morning() {
        assert_suggestion_result(
            "I wrote this whole program as a joke, at 1 AM in the morning. Nothing else to say.",
            AmInTheMorning::default(),
            "I wrote this whole program as a joke, at 1 AM. Nothing else to say.",
        );
        assert_nth_suggestion_result(
            "I wrote this whole program as a joke, at 1 AM in the morning. Nothing else to say.",
            AmInTheMorning::default(),
            "I wrote this whole program as a joke, at 1 in the morning. Nothing else to say.",
            1,
        );
    }

    #[test]
    fn fix_real_world_3am_in_the_morning() {
        assert_suggestion_result(
            "Luckily I was at home, but it was not fun at 3am in the morning.",
            AmInTheMorning::default(),
            "Luckily I was at home, but it was not fun at 3am.",
        );
        assert_nth_suggestion_result(
            "Luckily I was at home, but it was not fun at 3am in the morning.",
            AmInTheMorning::default(),
            "Luckily I was at home, but it was not fun at 3 in the morning.",
            1,
        );
    }

    #[test]
    fn fix_real_world_3am_at_night() {
        assert_suggestion_result(
            "If I want to run my script or some cron job at 3am at night, it seems to be not possible after macOS is in sleep mode.",
            AmInTheMorning::default(),
            "If I want to run my script or some cron job at 3am, it seems to be not possible after macOS is in sleep mode.",
        );
        assert_nth_suggestion_result(
            "If I want to run my script or some cron job at 3am at night, it seems to be not possible after macOS is in sleep mode.",
            AmInTheMorning::default(),
            "If I want to run my script or some cron job at 3 at night, it seems to be not possible after macOS is in sleep mode.",
            1,
        );
    }

    #[test]
    fn fix_real_world_9pm_at_night() {
        assert_suggestion_result(
            "The servers stop at 9PM at night and starts again at 9AM.",
            AmInTheMorning::default(),
            "The servers stop at 9PM and starts again at 9AM.",
        );
        assert_nth_suggestion_result(
            "The servers stop at 9PM at night and starts again at 9AM.",
            AmInTheMorning::default(),
            "The servers stop at 9 at night and starts again at 9AM.",
            1,
        );
    }

    #[test]
    fn fix_real_world_3_30_am_in_the_morning() {
        assert_suggestion_result(
            "Hello I can't believe my neighbor had the nerve to knock on my door at 3:30 AM in the morning.",
            AmInTheMorning::default(),
            "Hello I can't believe my neighbor had the nerve to knock on my door at 3:30 AM.",
        );
        assert_nth_suggestion_result(
            "Hello I can't believe my neighbor had the nerve to knock on my door at 3:30 AM in the morning.",
            AmInTheMorning::default(),
            "Hello I can't believe my neighbor had the nerve to knock on my door at 3:30 in the morning.",
            1,
        );
    }

    #[test]
    fn fix_real_world_5_pm_in_the_afternoon_caps_dots() {
        assert_suggestion_result(
            "Style issues get a blue marker: It's 5 P.M. in the afternoon.",
            AmInTheMorning::default(),
            "Style issues get a blue marker: It's 5 P.M..",
        );
        assert_nth_suggestion_result(
            "Style issues get a blue marker: It's 5 P.M. in the afternoon.",
            AmInTheMorning::default(),
            "Style issues get a blue marker: It's 5 in the afternoon.",
            1,
        );
    }

    #[test]
    fn fix_real_world_5_pm_in_the_afternoon_caps() {
        assert_suggestion_result(
            "Its a impressively versatile tool if youd like to tell a colleague from over sea's about at 5 PM in the afternoon on Monday, 27 May 2007.",
            AmInTheMorning::default(),
            "Its a impressively versatile tool if youd like to tell a colleague from over sea's about at 5 PM on Monday, 27 May 2007.",
        );
        assert_nth_suggestion_result(
            "Its a impressively versatile tool if youd like to tell a colleague from over sea's about at 5 PM in the afternoon on Monday, 27 May 2007.",
            AmInTheMorning::default(),
            "Its a impressively versatile tool if youd like to tell a colleague from over sea's about at 5 in the afternoon on Monday, 27 May 2007.",
            1,
        );
    }

    #[test]
    fn fix_real_world_6_pm_in_the_evening() {
        assert_suggestion_result(
            "I am in China and it is six pm in the evening.",
            AmInTheMorning::default(),
            "I am in China and it is six pm.",
        );
        assert_nth_suggestion_result(
            "I am in China and it is six pm in the evening.",
            AmInTheMorning::default(),
            "I am in China and it is six in the evening.",
            1,
        );
    }

    #[test]
    fn fix_real_world_4_am_in_the_morning() {
        assert_suggestion_result(
            "On the second application, we normally have the 503 between 1am and 4 am in the morning, almost every day.",
            AmInTheMorning::default(),
            "On the second application, we normally have the 503 between 1am and 4 am, almost every day.",
        );
        assert_nth_suggestion_result(
            "On the second application, we normally have the 503 between 1am and 4 am in the morning, almost every day.",
            AmInTheMorning::default(),
            "On the second application, we normally have the 503 between 1am and 4 in the morning, almost every day.",
            1,
        );
    }
}
