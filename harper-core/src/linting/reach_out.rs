use crate::{
    CharStringExt, Token, TokenStringExt,
    expr::{AnchorEnd, Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct ReachOut {
    expr: Box<dyn Expr>,
}

impl Default for ReachOut {
    fn default() -> Self {
        let inflections_of_reach = &["reach", "reached", "reaches", "reaching"];

        // let reach_out_expr = SequenceExpr::word_set(inflections_of_reach)
        //     .then_whitespace()
        //     .t_aco("out")
        //     .then_any_of(vec![
        //         Box::new(SequenceExpr::whitespace().then_word_set(&["to", "for"])),
        //         Box::new(AnchorEnd),
        //     ]);
        let reach_out_expr = SequenceExpr::word_set(inflections_of_reach)
            .then_whitespace()
            .t_aco("out")
            .then(AnchorEnd);

        Self {
            expr: Box::new(reach_out_expr),
        }
    }
}

impl ExprLinter for ReachOut {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;

        eprintln!("❤️ '{}'", span.get_content_string(src));
        let first_word = toks.first()?.span.get_content(src).to_lower();

        let last_letter_of_first_word = first_word.last().unwrap();

        let last_word = toks.last()?.span.get_content(src).to_lower();

        let instead: &[&str] = match (last_letter_of_first_word, last_word.as_ref()) {
            ('h', ['t', 'o']) => &["contact", "get in touch with"],
            ('h', ['f', 'o', 'r']) => &["seek", "get in touch for"],
            ('h', _) => &["get in touch"],

            ('d', ['t', 'o']) => &["contacted", "got in touch with", "gotten in touch with"],
            ('d', ['f', 'o', 'r']) => &["sought", "got in touch for", "gotten in touch for"],
            ('d', _) => &["got in touch"],

            ('s', ['t', 'o']) => &["contacts", "gets in touch with"],
            ('s', ['f', 'o', 'r']) => &["seeks", "gets in touch for"],
            ('s', _) => &["gets in touch"],

            ('g', ['t', 'o']) => &["contacting", "getting in touch with"],
            ('g', ['f', 'o', 'r']) => &["seeking", "getting in touch for"],
            ('g', _) => &["getting in touch"],
            _ => return None,
        };

        let suggestions = instead
            .iter()
            .map(|s| Suggestion::replace_with_match_case_str(s, span.get_content(src)))
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Redundancy,
            suggestions,
            message: "You can avoid using `reach out` by using `get in touch` instead.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &'static str {
        "Avoid using `reach out` by using `contact` or `get in touch` instead."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::{
        ReachOut,
        tests::{assert_lint_count, assert_top3_suggestion_result},
    };

    #[test]
    fn fix_reach_out_to() {
        assert_top3_suggestion_result(
            "They are the contact point for the Product Security Committee to reach out to for triaging and handling",
            ReachOut::default(),
            "They are the contact point for the Product Security Committee to get in touch with for triaging and handling",
        );
    }

    #[test]
    fn fix_reached_out_to() {
        assert_top3_suggestion_result(
            "I read different articles and also reached out to AI assistants for help",
            ReachOut::default(),
            "I read different articles and also contacted AI assistants for help",
        );
    }

    #[test]
    fn fix_reaches_out_to() {
        assert_top3_suggestion_result(
            "The front end reaches out to Google Generative API directly.",
            ReachOut::default(),
            "The front end contacts Google Generative API directly.",
        );
    }

    #[test]
    fn fix_reaching_out_to() {
        assert_top3_suggestion_result(
            "What is corepack reaching out to?",
            ReachOut::default(),
            "What is corepack contacting?",
        );
    }

    #[test]
    fn fix_reached_out_for_guidance() {
        assert_top3_suggestion_result(
            "thanks for clarifying, I reached out for further guidance",
            ReachOut::default(),
            "thanks for clarifying, I sought further guidance",
        );
    }

    #[test]
    fn fix_reached_out_for_help() {
        assert_top3_suggestion_result(
            "When I reached out for help in Matrix, I was made aware that linux-builder may not have caches set up for unstable.",
            ReachOut::default(),
            "When I sought help in Matrix, I was made aware that linux-builder may not have caches set up for unstable.",
        );
    }

    #[test]
    fn fix_reaching_out_for() {
        assert_top3_suggestion_result(
            "[QUESTION] reaching out for a contact, and to share some details about bark",
            ReachOut::default(),
            "[QUESTION] seeking a contact, and to share some details about bark",
        );
    }

    #[test]
    fn dont_flag_reaches_out_over() {
        assert_lint_count(
            "TestInstall reaches out over the internet",
            ReachOut::default(),
            0,
        );
    }

    #[test]
    fn fix_how_to_reach_out() {
        assert_top3_suggestion_result(
            "how to reach out",
            ReachOut::default(),
            "how to get in touch",
        );
    }

    #[test]
    fn fix_reach_out_for() {
        assert_top3_suggestion_result(
            "GitHub staff may reach out for further clarification or insight.",
            ReachOut::default(),
            "GitHub staff may get in touch for further clarification or insight.",
        );
    }
}
