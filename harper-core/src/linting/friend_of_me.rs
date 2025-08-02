use crate::{
    Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct FriendOfMe {
    expr: Box<dyn Expr>,
}

impl Default for FriendOfMe {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["friend", "friends", "enemy", "enemies"])
                    .then_whitespace()
                    .t_aco("of")
                    .t_ws()
                    .then_object_pronoun(),
            ),
        }
    }
}

impl ExprLinter for FriendOfMe {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let obj_pron_tok = toks.last()?;
        let obj_pron_str = obj_pron_tok.span.get_content_string(src);

        let poss_pron_str = match obj_pron_str.as_str() {
            "me" => "mine",
            "you" => "yours",
            "him" => "his",
            // "her" is also a possessive determiner, which leads to many false positives
            "her" => return None,
            "it" => return None,
            "us" => "ours",
            "them" => "theirs",
            _ => return None,
        };

        Some(Lint {
            span: obj_pron_tok.span,
            lint_kind: LintKind::Grammar,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                poss_pron_str,
                obj_pron_tok.span.get_content(src),
            )],
            message: format!("Use `{poss_pron_str}` instead of `{obj_pron_str}`."),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects wrong pronoun usage in constructions like `a friend of me`."
    }
}

#[cfg(test)]
mod tests {
    use super::FriendOfMe;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn corrects_friend_of_me() {
        assert_suggestion_result(
            "Last year a friend of me died unexpectedly (not a close friend).",
            FriendOfMe::default(),
            "Last year a friend of mine died unexpectedly (not a close friend).",
        );
    }

    #[test]
    fn corrects_friend_of_you() {
        assert_suggestion_result(
            "imagine a friend of you wants to disturb your call, and send you a session-terminate with a wrong SID",
            FriendOfMe::default(),
            "imagine a friend of yours wants to disturb your call, and send you a session-terminate with a wrong SID",
        );
    }

    #[test]
    fn corrects_friend_of_us() {
        assert_suggestion_result(
            "You have denounced a friend of us! You have denounced an enemy of us.",
            FriendOfMe::default(),
            "You have denounced a friend of ours! You have denounced an enemy of ours.",
        );
    }

    #[test]
    fn corrects_friends_of_them() {
        assert_suggestion_result(
            "guest has friend and they see which friends of them are comming",
            FriendOfMe::default(),
            "guest has friend and they see which friends of theirs are comming",
        );
    }

    #[test]
    fn corrects_friend_of_him() {
        assert_suggestion_result(
            "Ah, got it, i thought you may was a friend of him",
            FriendOfMe::default(),
            "Ah, got it, i thought you may was a friend of his",
        );
    }

    #[test]
    fn corrects_friends_of_me() {
        assert_suggestion_result(
            "guest has friend and they see which friends of me are comming",
            FriendOfMe::default(),
            "guest has friend and they see which friends of mine are comming",
        );
    }

    #[test]
    fn corrects_friends_of_us() {
        assert_suggestion_result(
            "This project was created for friends of us.",
            FriendOfMe::default(),
            "This project was created for friends of ours.",
        );
    }
}
