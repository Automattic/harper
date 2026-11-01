use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, preceded_by_word},
    },
};

/// Verbs of motion after which "to my defense" is correct ("rushed to my defense").
const MOTION_VERBS: &[&str] = &[
    "come",
    "comes",
    "came",
    "coming",
    "jump",
    "jumps",
    "jumped",
    "jumping",
    "leap",
    "leaps",
    "leapt",
    "leaped",
    "leaping",
    "rise",
    "rises",
    "rose",
    "risen",
    "rising",
    "rush",
    "rushes",
    "rushed",
    "rushing",
    "spring",
    "springs",
    "sprang",
    "sprung",
    "springing",
];

pub struct InOnesDefense {
    expr: SequenceExpr,
}

impl Default for InOnesDefense {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("to")
                .t_ws()
                .then_word_set(["my", "your", "his", "her", "its", "our", "their", "one's"])
                .t_ws()
                .then_word_set(["defense", "defence"]),
        }
    }
}

impl ExprLinter for InOnesDefense {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if preceded_by_word(ctx, |t| {
            t.get_ch(src).eq_any_ignore_ascii_case_str(MOTION_VERBS)
        }) {
            return None;
        }

        let span = toks[0].span;

        Some(Lint {
            span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "in",
                span.get_content(src),
            )],
            message:
                "When giving a justification, the idiom is `in my defense`, not `to my defense`."
                    .to_owned(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects `to my defense` to `in my defense` when it introduces a justification, but not after verbs of motion such as `came to my defense`."
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::InOnesDefense;

    #[test]
    fn fix_to_my_defense_from_issue() {
        assert_suggestion_result(
            "I did try to drive this thing. And to my defense, um, I bought this before I had other financial and time responsibilities.",
            InOnesDefense::default(),
            "I did try to drive this thing. And in my defense, um, I bought this before I had other financial and time responsibilities.",
        );
    }

    #[test]
    fn fix_sentence_initial_to_their_defence() {
        assert_suggestion_result(
            "To their defence, the docs never mentioned it.",
            InOnesDefense::default(),
            "In their defence, the docs never mentioned it.",
        );
    }

    #[test]
    fn fix_to_ones_defense() {
        assert_suggestion_result(
            "Everyone has something to say to one's defense.",
            InOnesDefense::default(),
            "Everyone has something to say in one's defense.",
        );
    }

    #[test]
    fn allows_rushed_to_my_defense() {
        assert_no_lints(
            "My friends rushed to my defense when the reviewer complained.",
            InOnesDefense::default(),
        );
    }

    #[test]
    fn allows_came_to_her_defence() {
        assert_no_lints(
            "Nobody came to her defence after the outage.",
            InOnesDefense::default(),
        );
    }

    #[test]
    fn allows_in_my_defense() {
        assert_no_lints(
            "In my defense, the tests were already failing.",
            InOnesDefense::default(),
        );
    }
}
