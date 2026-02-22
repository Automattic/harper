use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::{LintKind, Suggestion};
use crate::token_string_ext::TokenStringExt;

use super::{ExprLinter, Lint};
use crate::linting::expr_linter::Chunk;

pub struct LetToDo {
    expr: Box<dyn Expr>,
}

impl Default for LetToDo {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["let", "lets", "let's"])
                    .t_ws()
                    .then_any_of(vec![
                        Box::new(SequenceExpr::default().then_object_pronoun()),
                        Box::new(SequenceExpr::word_set(&[
                            // Elective existential indefinite pronouns
                            "anybody",
                            "anyone",
                            // Universal indefinite pronouns
                            "everybody",
                            "everyone",
                            // Negative indefinite pronouns (correct)
                            "nobody",
                            // Negative indefinite pronouns (incorrect)
                            "noone",
                            // Assertive existential indefinite pronouns
                            "somebody",
                            "someone",
                        ])),
                        Box::new(
                            SequenceExpr::word_set(&["any", "every", "no", "some"])
                                .t_ws()
                                .then_word_set(&["body", "one"]),
                        ),
                    ])
                    .t_ws()
                    .t_aco("to"),
            ),
        }
    }
}

impl ExprLinter for LetToDo {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks[toks.len() - 2..].span()?,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::Remove],
            message: "The word `to` should not be used with `let` in this way.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &'static str {
        "Corrects extraneous `to` after `let`."
    }
}

#[cfg(test)]
mod tests {
    use super::LetToDo;
    use crate::linting::tests::assert_suggestion_result;

    // let

    #[test]
    fn let_me_to() {
        assert_suggestion_result(
            "Let me to decide show player controls or not",
            LetToDo::default(),
            "Let me decide show player controls or not",
        );
    }

    #[test]
    fn let_you_to() {
        assert_suggestion_result(
            "Azure provider does not let you to ask for approval",
            LetToDo::default(),
            "Azure provider does not let you ask for approval",
        );
    }

    #[test]
    fn let_us_to() {
        assert_suggestion_result(
            "Currently JavaScript syntax does not let us to pick one or more properties from an object",
            LetToDo::default(),
            "Currently JavaScript syntax does not let us pick one or more properties from an object",
        );
    }

    #[test]
    fn let_it_to() {
        assert_suggestion_result(
            "I modified the load_checkpoints and let it to only load the autoencoder's checkpoint.",
            LetToDo::default(),
            "I modified the load_checkpoints and let it only load the autoencoder's checkpoint.",
        );
    }

    #[test]
    fn let_him_to() {
        assert_suggestion_result(
            "let him to delete the information he wrote",
            LetToDo::default(),
            "let him delete the information he wrote",
        );
    }

    #[test]
    fn let_anybody_to() {
        assert_suggestion_result(
            "but at least will not let anybody to sub from tls as far as i understood this properly",
            LetToDo::default(),
            "but at least will not let anybody sub from tls as far as i understood this properly",
        );
    }

    #[test]
    fn let_anyone_to() {
        assert_suggestion_result(
            "How would you let anyone to help you?",
            LetToDo::default(),
            "How would you let anyone help you?",
        );
    }

    #[test]
    fn let_any_one_to() {
        assert_suggestion_result(
            "set up a mcp server to let any one to query how to use the api in the repo",
            LetToDo::default(),
            "set up a mcp server to let any one query how to use the api in the repo",
        );
    }

    #[test]
    fn let_everybody_to() {
        assert_suggestion_result(
            "on a project that let everybody to create things",
            LetToDo::default(),
            "on a project that let everybody create things",
        );
    }

    #[test]
    fn let_everyone_to() {
        assert_suggestion_result(
            "We want to let everyone to be able to select between the servers we have right now",
            LetToDo::default(),
            "We want to let everyone be able to select between the servers we have right now",
        );
    }

    #[test]
    fn let_every_one_to() {
        assert_suggestion_result(
            "you SHOULDN'T DO THIS because you let every one to upload an executable",
            LetToDo::default(),
            "you SHOULDN'T DO THIS because you let every one upload an executable",
        );
    }

    #[test]
    fn let_nobody_to() {
        assert_suggestion_result(
            "i wouldn't let nobody to snoop on user's data",
            LetToDo::default(),
            "i wouldn't let nobody snoop on user's data",
        );
    }

    #[test]
    fn let_no_one_to() {
        assert_suggestion_result(
            "hide video download link in wordpress - let no one to see video download link",
            LetToDo::default(),
            "hide video download link in wordpress - let no one see video download link",
        );
    }

    #[test]
    fn let_somebody_to() {
        assert_suggestion_result(
            "So it should be the same let somebody to make required for reviewers.",
            LetToDo::default(),
            "So it should be the same let somebody make required for reviewers.",
        );
    }

    #[test]
    fn let_some_one_to() {
        assert_suggestion_result(
            "let some one to help me who can do it",
            LetToDo::default(),
            "let some one help me who can do it",
        );
    }

    // lets

    #[test]
    fn lets_me_to() {
        assert_suggestion_result(
            "Also, clicking on the gear lets me to put login credentials",
            LetToDo::default(),
            "Also, clicking on the gear lets me put login credentials",
        );
    }

    #[test]
    fn lets_you_to() {
        assert_suggestion_result(
            "A chrome extension which lets you to create your own customised text snippets and use in your browser.",
            LetToDo::default(),
            "A chrome extension which lets you create your own customised text snippets and use in your browser.",
        );
    }

    #[test]
    fn lets_us_to() {
        assert_suggestion_result(
            "A menu which lets us to get money",
            LetToDo::default(),
            "A menu which lets us get money",
        );
    }

    #[test]
    fn lets_anyone_to() {
        assert_suggestion_result(
            "fake ssh server that lets anyone to connect and monitor their activty",
            LetToDo::default(),
            "fake ssh server that lets anyone connect and monitor their activty",
        );
    }

    #[test]
    fn lets_everybody_to() {
        assert_suggestion_result(
            "Use set.seed function which lets everybody to check your result on their computers.",
            LetToDo::default(),
            "Use set.seed function which lets everybody check your result on their computers.",
        );
    }

    #[test]
    fn lets_everyone_to() {
        assert_suggestion_result(
            "what lets everyone to know what to expect",
            LetToDo::default(),
            "what lets everyone know what to expect",
        );
    }

    #[test]
    fn lets_someone_to() {
        assert_suggestion_result(
            "it works correctly and lets someone to connect using telnet",
            LetToDo::default(),
            "it works correctly and lets someone connect using telnet",
        );
    }

    // erroneous let's

    #[test]
    fn lets_me_to_apostrophe() {
        assert_suggestion_result(
            "I need be able to clone receiver, which crossbeam let's me to do.",
            LetToDo::default(),
            "I need be able to clone receiver, which crossbeam let's me do.",
        );
    }

    #[test]
    fn lets_us_to_apostrophe() {
        assert_suggestion_result(
            "this option let's us to cut the whole project in smaller pieces, like time based sprites.",
            LetToDo::default(),
            "this option let's us cut the whole project in smaller pieces, like time based sprites.",
        );
    }

    #[test]
    fn lets_you_to_apostrophe() {
        assert_suggestion_result(
            "Let's you to migrate/transfer save files from older outward versions to outward definitive edition.",
            LetToDo::default(),
            "Let's you migrate/transfer save files from older outward versions to outward definitive edition.",
        );
    }

    #[test]
    fn lets_him_to_apostrophe() {
        assert_suggestion_result(
            "A good woman gives wings to a man and let's him to take on the whole world",
            LetToDo::default(),
            "A good woman gives wings to a man and let's him take on the whole world",
        );
    }

    #[test]
    fn lets_her_to_apostrophe() {
        assert_suggestion_result(
            "if Igor let's her to be with Wonder she would crush everyone",
            LetToDo::default(),
            "if Igor let's her be with Wonder she would crush everyone",
        );
    }

    #[test]
    fn lets_it_to_apostrophe() {
        assert_suggestion_result(
            "It's as accurate as your GPS let's it to be.",
            LetToDo::default(),
            "It's as accurate as your GPS let's it be.",
        );
    }

    #[test]
    fn lets_them_to_apostrophe() {
        assert_suggestion_result(
            "it does not cut the dataset into 2 separate subsets in each step but let's them to get predictions from both branches",
            LetToDo::default(),
            "it does not cut the dataset into 2 separate subsets in each step but let's them get predictions from both branches",
        );
    }

    #[test]
    fn lets_anyone_to_apostrophe() {
        assert_suggestion_result(
            "It let's anyone to discover, take, or even teach a class.",
            LetToDo::default(),
            "It let's anyone discover, take, or even teach a class.",
        );
    }

    #[test]
    fn lets_any_one_to_apostrophe() {
        assert_suggestion_result(
            "Whyyyy hashmeet let's any one to create their own tinder like swipe clubs or groups?",
            LetToDo::default(),
            "Whyyyy hashmeet let's any one create their own tinder like swipe clubs or groups?",
        );
    }

    #[test]
    fn lets_everyone_to_apostrophe() {
        assert_suggestion_result(
            "How do you feel that America let's everyone to have guns",
            LetToDo::default(),
            "How do you feel that America let's everyone have guns",
        );
    }

    #[test]
    fn lets_someone_to_apostrophe() {
        assert_suggestion_result(
            "Commands and attributes let's someone to compare it with other TRV",
            LetToDo::default(),
            "Commands and attributes let's someone compare it with other TRV",
        );
    }
}
