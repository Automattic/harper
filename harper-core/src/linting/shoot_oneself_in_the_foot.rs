use crate::{
    CharStringExt, Span, Token,
    expr::{Expr, ReflexivePronoun, SequenceExpr},
    linting::Suggestion,
    patterns::WordSet,
};

use super::{ExprLinter, Lint, LintKind};
use crate::linting::expr_linter::Chunk;

pub struct ShootOneselfInTheFoot {
    pattern: Box<dyn Expr>,
}

impl Default for ShootOneselfInTheFoot {
    fn default() -> Self {
        let verb_forms = WordSet::new(&["shoot", "shooting", "shoots", "shot", "shooted"]);

        let body_parts = WordSet::new(&["foot", "feet", "leg", "legs"]);

        let pattern = SequenceExpr::with(verb_forms)
            .t_ws()
            .then(ReflexivePronoun::default())
            .t_ws()
            .then_preposition()
            .t_ws()
            .then_determiner()
            .t_ws()
            .then(body_parts);
        Self {
            pattern: Box::new(pattern),
        }
    }
}

impl ExprLinter for ShootOneselfInTheFoot {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.pattern.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let pron = &toks.get(2)?.span.get_content(src);
        let prep = &toks.get(4)?.span.get_content(src);
        let det = &toks.get(6)?.span.get_content(src);
        let body_part = &toks.get(8)?.span.get_content(src);

        let plural_pron = pron.ends_with_ignore_ascii_case_str("elves");
        let plural_foot = toks.get(8)?.kind.is_plural_noun();

        let is_in = prep.eq_ignore_ascii_case_str("in");
        let is_the = det.eq_ignore_ascii_case_str("the");
        let is_foot = body_part.eq_ignore_ascii_case_str("foot");

        let foot_ok = is_foot || (plural_pron && plural_foot);

        if is_in && is_the && foot_ok {
            return None;
        }

        let in_the_foot = Span::new(toks.get(4)?.span.start, toks.get(8)?.span.end);

        let mut suggestions = vec![Suggestion::replace_with_match_case_str(
            "in the foot",
            in_the_foot.get_content(src),
        )];

        if plural_pron {
            suggestions.push(Suggestion::replace_with_match_case_str(
                "in the feet",
                in_the_foot.get_content(src),
            ));
        }

        Some(Lint {
            span: in_the_foot,
            lint_kind: LintKind::Miscellaneous,
            suggestions,
            message: "The standard idiom is 'shoot oneself in the foot'.".to_string(),
            priority: 50,
        })
    }

    fn description(&self) -> &str {
        "Corrects nonstandard variants of 'shoot oneself in the foot'."
    }
}

#[cfg(test)]
mod tests {
    use super::ShootOneselfInTheFoot;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn ignore_correct() {
        assert_lint_count(
            "Don't shoot yourself in the foot.",
            ShootOneselfInTheFoot::default(),
            0,
        );
    }

    #[test]
    fn ignore_title_case() {
        assert_lint_count(
            "Don't Shoot Yourself In The Foot.",
            ShootOneselfInTheFoot::default(),
            0,
        );
    }

    #[test]
    fn ignore_all_caps() {
        assert_lint_count(
            "DON'T SHOOT YOURSELF IN THE FOOT.",
            ShootOneselfInTheFoot::default(),
            0,
        );
    }

    #[test]
    fn fix_shoot_leg() {
        assert_suggestion_result(
            "I managed to shoot myself in the leg when using CF Workers deployment",
            ShootOneselfInTheFoot::default(),
            "I managed to shoot myself in the foot when using CF Workers deployment",
        );
    }

    #[test]
    fn fix_shoot_into_foot() {
        assert_suggestion_result(
            "Or should we keep them to prevent users from shooting themselves into the foot?",
            ShootOneselfInTheFoot::default(),
            "Or should we keep them to prevent users from shooting themselves in the foot?",
        );
    }

    #[test]
    fn fix_shoot_into_feet() {
        assert_suggestion_result(
            "(to prevent you from shooting yourself into the feet)",
            ShootOneselfInTheFoot::default(),
            "(to prevent you from shooting yourself in the foot)",
        );
    }

    #[test]
    fn ignore_themselves_foot() {
        assert_lint_count(
            "Thou shalt not make a rule that prevents C++ programmers from shooting themselves in the foot.",
            ShootOneselfInTheFoot::default(),
            0,
        );
    }

    #[test]
    fn ignore_ourselves_feet() {
        assert_lint_count(
            "It will help avoiding shooting ourselves in the feet.",
            ShootOneselfInTheFoot::default(),
            0,
        );
    }

    #[test]
    fn fix_a_foot() {
        assert_suggestion_result(
            "Shot ourselves in a foot, \"Wrong X-Request-Key\" error #589.",
            ShootOneselfInTheFoot::default(),
            "Shot ourselves in the foot, \"Wrong X-Request-Key\" error #589.",
        );
    }

    #[test]
    fn ignore_shoots_himself() {
        assert_suggestion_result(
            "the administrator shoots himself in the foot and then hops around",
            ShootOneselfInTheFoot::default(),
            "the administrator shoots himself in the foot and then hops around",
        );
    }

    #[test]
    fn ignore_shooting_oneself_in_the_foot() {
        assert_lint_count(
            "A historical document of shooting oneself in the foot, if you will.",
            ShootOneselfInTheFoot::default(),
            0,
        );
    }

    #[test]
    fn fix_oneself_in_a_foot() {
        assert_suggestion_result(
            "Forgetting to declare some variable local withing a function definition is a common way to shoot oneself in a foot",
            ShootOneselfInTheFoot::default(),
            "Forgetting to declare some variable local withing a function definition is a common way to shoot oneself in the foot",
        );
    }

    #[test]
    fn fix_oneself_in_the_feet() {
        assert_suggestion_result(
            "Forgetting to declare some variable local withing a function definition is a common way to shoot oneself in the feet",
            ShootOneselfInTheFoot::default(),
            "Forgetting to declare some variable local withing a function definition is a common way to shoot oneself in the foot",
        );
    }

    #[test]
    fn fix_oneself_into_the_leg() {
        assert_suggestion_result(
            "Forgetting to declare some variable local withing a function definition is a common way to shoot oneself into the foot",
            ShootOneselfInTheFoot::default(),
            "Forgetting to declare some variable local withing a function definition is a common way to shoot oneself in the foot",
        );
    }

    #[test]
    fn ignore_oneself_in_the_toes() {
        assert_lint_count(
            "Forgetting to declare some variable local withing a function definition is a common way to shoot oneself in the toes",
            ShootOneselfInTheFoot::default(),
            0,
        );
    }
}
