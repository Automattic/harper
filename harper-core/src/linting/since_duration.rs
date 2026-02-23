use crate::expr::{DurationExpr, Expr, SequenceExpr};
use crate::{CharStringExt, Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

const AGO_VARIANTS: [&[char]; 3] = [&['a', 'g', 'o'], &['A', 'g', 'o'], &['A', 'G', 'O']];
const FOR_VARIANTS: [&[char]; 3] = [&['f', 'o', 'r'], &['F', 'o', 'r'], &['F', 'O', 'R']];

fn match_case_string<'a>(template: &[char], variants: [&'a [char]; 3]) -> &'a [char] {
    let c1 = template.first().copied().unwrap();
    let c2 = template.get(1).copied().unwrap_or(' ');
    if c1.is_uppercase() && c2.is_uppercase() {
        variants[2]
    } else if c1.is_uppercase() {
        variants[1]
    } else {
        variants[0]
    }
}

pub struct SinceDuration {
    expr: Box<dyn Expr>,
}

impl Default for SinceDuration {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::any_capitalization_of("since")
                    .then_whitespace()
                    .then(DurationExpr)
                    .then_optional(
                        SequenceExpr::default()
                            .t_ws()
                            .then_word_set(&["ago", "old"]),
                    ),
            ),
        }
    }
}

impl ExprLinter for SinceDuration {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let last = toks.last()?;
        if last
            .span
            .get_content(src)
            .eq_any_ignore_ascii_case_chars(&[&['a', 'g', 'o'], &['o', 'l', 'd']])
        {
            return None;
        }

        let since_duration_span = toks.span()?;

        let mut since_point_in_time = since_duration_span.get_content(src).to_vec();
        since_point_in_time.push(' ');
        let unit_template = toks.last()?.span.get_content(src);
        since_point_in_time.extend(
            match_case_string(unit_template, AGO_VARIANTS)
                .iter()
                .copied(),
        );
        let ago_suggestion = Suggestion::ReplaceWith(since_point_in_time);

        let duration = toks[1..].span()?.get_content(src);
        let since_template = toks.first()?.span.get_content(src);
        let mut for_duration = match_case_string(since_template, FOR_VARIANTS).to_vec();
        for_duration.extend(duration);
        let for_suggestion = Suggestion::ReplaceWith(for_duration);

        Some(Lint {
            span: since_duration_span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![for_suggestion, ago_suggestion],
            message: "For a duration, use 'for' instead of 'since'. Or for a point in time, add 'ago' at the end.".to_string(),
            priority: 50,
        })
    }

    fn description(&self) -> &str {
        "Detects the use of 'since' with a duration instead of a point in time."
    }
}

#[cfg(test)]
mod tests {
    use super::SinceDuration;
    use crate::linting::tests::{
        assert_lint_count, assert_no_lints, assert_top3_suggestion_result,
    };

    #[test]
    fn catches_spelled() {
        assert_lint_count(
            "I have been waiting since two hours.",
            SinceDuration::default(),
            1,
        );
    }

    #[test]
    fn permits_spelled_with_ago() {
        assert_no_lints(
            "I have been waiting since two hours ago.",
            SinceDuration::default(),
        );
    }

    #[test]
    fn catches_numerals() {
        assert_lint_count(
            "I have been waiting since 2 hours.",
            SinceDuration::default(),
            1,
        );
    }

    #[test]
    fn permits_numerals_with_ago() {
        assert_no_lints(
            "I have been waiting since 2 hours ago.",
            SinceDuration::default(),
        );
    }

    #[test]
    fn correct_without_issues() {
        assert_top3_suggestion_result(
            "I'm running v2.2.1 on bare metal (no docker, vm) since two weeks without issues.",
            SinceDuration::default(),
            "I'm running v2.2.1 on bare metal (no docker, vm) for two weeks without issues.",
        );
    }

    #[test]
    fn correct_anything_back() {
        assert_top3_suggestion_result(
            "I have not heard anything back since three months.",
            SinceDuration::default(),
            "I have not heard anything back for three months.",
        );
    }

    #[test]
    fn correct_get_done() {
        assert_top3_suggestion_result(
            "I am trying to get this done since two days, someone please help.",
            SinceDuration::default(),
            "I am trying to get this done for two days, someone please help.",
        );
    }

    #[test]
    fn correct_deprecated() {
        assert_top3_suggestion_result(
            "This project is now officially deprecated, since I worked with virtualabs on the next version of Mirage since three years now: an ecosystem of tools named WHAD.",
            SinceDuration::default(),
            "This project is now officially deprecated, since I worked with virtualabs on the next version of Mirage for three years now: an ecosystem of tools named WHAD.",
        );
    }

    #[test]
    fn correct_same() {
        assert_top3_suggestion_result(
            "Same! Since two days.",
            SinceDuration::default(),
            "Same! For two days.",
        );
    }

    #[test]
    fn correct_what_changed() {
        assert_top3_suggestion_result(
            "What changed since two weeks?",
            SinceDuration::default(),
            "What changed since two weeks ago?",
        );
    }

    #[test]
    fn correct_with_period() {
        assert_top3_suggestion_result(
            "I have been waiting since two hours.",
            SinceDuration::default(),
            "I have been waiting since two hours ago.",
        );
    }

    #[test]
    fn correct_with_exclamation() {
        assert_top3_suggestion_result(
            "I have been waiting since two hours!",
            SinceDuration::default(),
            "I have been waiting since two hours ago!",
        );
    }

    #[test]
    fn correct_with_question_mark() {
        assert_top3_suggestion_result(
            "Have you been waiting since two hours?",
            SinceDuration::default(),
            "Have you been waiting for two hours?",
        );
    }

    #[test]
    fn correct_with_comma() {
        assert_top3_suggestion_result(
            "Since two days, I have been trying to get this done.",
            SinceDuration::default(),
            "For two days, I have been trying to get this done.",
        );
    }

    #[test]
    fn correct_for_title_case() {
        assert_top3_suggestion_result(
            "Since 45 Minutes I See The Following Picture In The Terminal.",
            SinceDuration::default(),
            "For 45 Minutes I See The Following Picture In The Terminal.",
        );
    }

    #[test]
    fn correct_for_all_caps() {
        assert_top3_suggestion_result(
            "STOPPED SINCE 12 HOURS WITH EXIT CODE 0",
            SinceDuration::default(),
            "STOPPED FOR 12 HOURS WITH EXIT CODE 0",
        );
    }

    #[test]
    fn correct_ago_title_case() {
        assert_top3_suggestion_result(
            "It Is In Development Since Two Years.",
            SinceDuration::default(),
            "It Is In Development Since Two Years Ago.",
        );
    }

    #[test]
    fn correct_ago_all_caps() {
        assert_top3_suggestion_result(
            "BUG: SINCE 6 MONTHS UNLOAD CHECKPOINT",
            SinceDuration::default(),
            "BUG: SINCE 6 MONTHS AGO UNLOAD CHECKPOINT",
        );
    }

    #[test]
    #[ignore = "We can't yet handle modifiers like 'over'. Plus it doesn't work with 'ago'."]
    fn not_yet_handled() {
        assert_top3_suggestion_result(
            "It's an asked feature since over 9 years",
            SinceDuration::default(),
            "It's an asked feature for over 9 years.",
        );
    }

    #[test]
    #[ignore = "We can't yet handle modifiers like 'more than'. Plus it doesn't work with 'ago'."]
    fn not_yet_handled_2() {
        assert_top3_suggestion_result(
            "It's an asked feature since more than 9 years",
            SinceDuration::default(),
            "It's an asked feature for more than 9 years.",
        );
    }

    #[test]
    #[ignore = "We can't yet handle indefinite numbers."]
    fn not_yet_handled_3() {
        assert_top3_suggestion_result(
            "I use a Wacom Cintiq 27QHDT since several years on Linux",
            SinceDuration::default(),
            "I use a Wacom Cintiq 27QHDT for several years on Linux",
        );
    }

    #[test]
    fn ignore_since_years_old() {
        assert_no_lints(
            "I've been coding since 11 years old and I'm now 57",
            SinceDuration::default(),
        );
    }
}
