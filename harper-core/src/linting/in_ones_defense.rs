use crate::{
    CharStringExt, Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_hyphen, followed_by_word, preceded_by_word},
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

const VERBS_BEFORE_ON: &[&str] = &[
    "capitalise",
    "capitalised",
    "capitalising",
    "capitalize",
    "capitalized",
    "capitalizing",
    "spend",
    "spending",
    "spent",
    "work",
    "worked",
    "working",
];
const OTHER_BEFORE_ON: &[&str] = &["congrats", "impact", "money"];

pub struct InOnesDefense {
    expr: SequenceExpr,
}

impl Default for InOnesDefense {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_set(["to", "on"])
                .t_ws()
                .then_word_set(["my", "your", "his", "her", "its", "our", "their", "one's"])
                .t_ws()
                .then_word_set(["defense", "defence"]),
        }
    }
}

enum Prep {
    To,
    On,
}
use Prep::*;

impl ExprLinter for InOnesDefense {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let preptok = &toks[0];
        let prepch = preptok.get_ch(src);

        let prep = if prepch.eq_str("to") {
            To
        } else if prepch.eq_str("on") {
            On
        } else {
            return None;
        };

        match prep {
            To => {
                if preceded_by_word(ctx, |t| {
                    t.get_ch(src).eq_any_ignore_ascii_case_str(MOTION_VERBS)
                }) {
                    return None;
                }
            }
            On => {
                let _pron = toks[2].get_ch(src);
                if followed_by_hyphen(ctx)
                    || followed_by_word(ctx, |t| t.kind.is_noun() || t.get_ch(src).eq_str("of"))
                    || preceded_by_word(ctx, |t| {
                        let before = t.get_ch(src);
                        before.eq_any_ignore_ascii_case_str(VERBS_BEFORE_ON)
                            || before.eq_any_ignore_ascii_case_str(OTHER_BEFORE_ON)
                    })
                {
                    return None;
                }
            }
        }

        Some(Lint {
            span: preptok.span,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case_str("in", prepch)],
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

    #[test]
    fn fix_on_my_defense() {
        assert_suggestion_result(
            "But, on my defense, it didn't make sense (to me) that Bismark runs out of memory",
            InOnesDefense::default(),
            "But, in my defense, it didn't make sense (to me) that Bismark runs out of memory",
        );
    }

    #[test]
    fn fix_on_my_defence() {
        assert_suggestion_result(
            "Well, on my defence, it didn't work when done for the first time",
            InOnesDefense::default(),
            "Well, in my defence, it didn't work when done for the first time",
        );
    }

    #[test]
    fn dont_flag_on_your_defense_logic() {
        assert_no_lints("Iterate on your defense logic.", InOnesDefense::default());
    }

    #[test]
    fn dont_flag_on_defense_hyphen() {
        assert_no_lints(
            "Can we identify a news outlet, based on its defense-related news?",
            InOnesDefense::default(),
        );
    }

    #[test]
    fn dont_flag_congrats_on_your_defense() {
        assert_no_lints("Congrats on your defense!", InOnesDefense::default());
    }

    #[test]
    fn dont_flag_impact_on_your_defense() {
        assert_no_lints(
            "Flimsy+ and Slow Healer+ have no impact on your Defense.",
            InOnesDefense::default(),
        );
    }

    #[test]
    fn dont_flag_spent_on_defense() {
        assert_no_lints(
            "I'd be happy if our defense budget was actually spent on our defense, and not proxy wars",
            InOnesDefense::default(),
        );
    }

    #[test]
    fn dont_flag_on_defence_of() {
        assert_no_lints(
            "Further down he doubles down on his defence of Minsky.",
            InOnesDefense::default(),
        );
    }

    #[test]
    fn dont_flag_on_defense_of() {
        assert_no_lints(
            "I'd like to double down on your defense of OP.",
            InOnesDefense::default(),
        );
    }

    #[test]
    fn dont_flag_working_on_defense() {
        assert_no_lints(
            "You'll be working on your defense from jail, of course.",
            InOnesDefense::default(),
        );
    }

    #[test]
    fn dont_flag_spend_money_on_defense() {
        assert_no_lints(
            "The EU can afford to not spend money on their defense because we exist",
            InOnesDefense::default(),
        );
    }
}
