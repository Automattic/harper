use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, surrounded_by_words},
    },
};

pub struct ToTo {
    expr: SequenceExpr,
}

impl Default for ToTo {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::aco("to").t_ws_h().t_aco("to"),
        }
    }
}

impl ExprLinter for ToTo {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        if toks[1].kind.is_whitespace()
            && surrounded_by_words(ctx, |before, after| {
                if after.kind.is_verb_lemma()
                    && before.get_ch(src).eq_any_ignore_ascii_case_str(&[
                        // Prepositions
                        "according",
                        // Verbs. If this list grows large, adding a dictionary annotation flag should be considered.
                        "appeal",
                        "appealed",
                        "appealing",
                        "appeals",
                        "apply",
                        "applied",
                        "applies",
                        "applying",
                        "connect",
                        "connected",
                        "connecting",
                        "connects",
                        "have",
                        "had",
                        "having",
                        "has",
                        "need",
                        "needed",
                        "needing",
                        "needs",
                        "pray",
                        "prayed",
                        "praying",
                        "prays",
                        "refer",
                        "referred",
                        "referring",
                        "refers",
                        "resort",
                        "resorted",
                        "resorting",
                        "resorts",
                    ])
                {
                    return true;
                }
                false
            })
        {
            return None;
        }

        let to_to_span = toks.span()?;
        let first_to_w_sep = &toks[0..=1].span()?;

        let to_do = first_to_w_sep
            .get_content(src)
            .iter()
            .chain(['d', 'o'].iter())
            .copied()
            .collect::<Vec<char>>();

        let suggestions = vec![
            Suggestion::replace_with_match_case_str("to", to_to_span.get_content(src)),
            Suggestion::replace_with_match_case(to_do, to_to_span.get_content(src)),
        ];

        Some(Lint {
            span: to_to_span,
            lint_kind: LintKind::Typo,
            suggestions,
            message: "Is the second `to` supposed to be `do` or not there at all?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the typo `to to` by either removing the duplication or changing it to `to do`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ToTo;

    // to do / to-do tests

    #[test]
    fn should_be_to_do_with_space() {
        assert_suggestion_result(
            "I need to add that to my to to list first.",
            ToTo::default(),
            "I need to add that to my to do list first.",
        );
    }

    #[test]
    fn should_be_to_do_with_hyphen() {
        assert_suggestion_result(
            "I need to add that to my to-to list first.",
            ToTo::default(),
            "I need to add that to my to-do list first.",
        );
    }

    #[test]
    fn triple_to_should_be_to_do_to() {
        assert_suggestion_result(
            "a little example of what you'd need to to to create and resolve a promise off the main thread",
            ToTo::default(),
            "a little example of what you'd need to do to create and resolve a promise off the main thread",
        );
    }

    // extraneous "to" tests

    #[test]
    fn should_be_single_to() {
        assert_suggestion_result(
            "What is the modern way to to do XML transforms in Python?",
            ToTo::default(),
            "What is the modern way to do XML transforms in Python?",
        );
    }

    // exceptions where "to to" is legit

    #[test]
    fn dont_flag_applied_to_to_stand_out() {
        assert_no_lints(
            "Indeed says to store messages to jobs I've applied to to stand out",
            ToTo::default(),
        )
    }

    #[test]
    fn dont_flag_apply_to_to_be() {
        assert_no_lints(
            "I'm expecting the company that I'm applying to to be efficient, to assign me tasks that aren't a waste of time",
            ToTo::default(),
        )
    }

    #[test]
    fn dont_flag_connected_to_to_allow() {
        assert_no_lints(
            "Create headless \"VS Code Server\" that can be connected to to allow editing remote code within it's environment",
            ToTo::default(),
        );
    }

    #[test]
    fn dont_flag_have_to_to_take() {
        assert_no_lints(
            "Do what you have to to take care of yourself and get what you need.",
            ToTo::default(),
        )
    }

    #[test]
    fn dont_flag_need_to_to_increase() {
        assert_no_lints(
            "something that they're wasting their time doing, so adjust as you need to to increase their productivity",
            ToTo::default(),
        );
    }

    #[test]
    fn dont_flag_pray_to_to_find() {
        assert_no_lints(
            "what dieties would you pray to to find lost things and for a housing opportunity?",
            ToTo::default(),
        );
    }

    #[test]
    fn dont_flag_referred_to_to_exist() {
        assert_no_lints(
            "IsValid requires a drive being referred to to exist",
            ToTo::default(),
        );
    }

    #[test]
    fn dont_flag_referred_to_to_write() {
        assert_no_lints(
            "Could you tell me which paper you referred to to write this algorithm?",
            ToTo::default(),
        );
    }

    #[test]
    fn dont_flag_refer_to_to_create() {
        assert_no_lints(
            "Which Packages do I need to refer to to create my own NUnit console runner?",
            ToTo::default(),
        );
    }

    #[test]
    fn dont_flag_need_to_to_stay() {
        assert_no_lints(
            "Yes, if I fail of course I'll do whatever I need to to stay healthy",
            ToTo::default(),
        )
    }

    #[test]
    fn dont_flag_refer_to_to_ensure() {
        assert_no_lints(
            "It could be a convenience list that one could refer to to ensure that they're not about to say a banned word.",
            ToTo::default(),
        );
    }

    // Still mistakes after keywords above since they lack the required following infitive verb

    #[test]
    fn fix_according_to_to() {
        assert_suggestion_result(
            "this URL is not formatted strictly according to to RFC2396",
            ToTo::default(),
            "this URL is not formatted strictly according to RFC2396",
        );
    }

    #[test]
    fn fix_applied_to_to() {
        assert_suggestion_result(
            "TG_IR.corr(xlim) is not applied to to correction graphs",
            ToTo::default(),
            "TG_IR.corr(xlim) is not applied to correction graphs",
        );
    }

    #[test]
    fn fix_refer_refer_to() {
        assert_suggestion_result(
            "but there may be a chance to to refer non existing actor",
            ToTo::default(),
            "but there may be a chance to refer non existing actor",
        )
    }

    #[test]
    fn fix_refers_to_to() {
        assert_suggestion_result(
            "Getting \"{{component}} refers to to a value, but is being used as a type\" error when attemping to identify an argument that refers to a Vue Component",
            ToTo::default(),
            "Getting \"{{component}} refers to a value, but is being used as a type\" error when attemping to identify an argument that refers to a Vue Component",
        );
    }
}
