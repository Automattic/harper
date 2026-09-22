use crate::{
    CharStringExt, Lint, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_word},
    },
};

pub struct BetterOffServed {
    expr: SequenceExpr,
}

impl Default for BetterOffServed {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::word_seq(&["better", "off", "served"])
                .t_ws()
                .then_any_word(),
        }
    }
}

impl ExprLinter for BetterOffServed {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        cttx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        enum Which {
            KeepOff,
            KeepServed,
        }
        use Which::*;

        let next_tok = toks.last()?;
        let next_ch = next_tok.get_ch(src);

        /* Flowchart produced by Google Search's AI by analysing the real-world examples I collected:

               Is "served" followed by "by"?
                         /          \
                       YES           NO
                       /              \
        Fix to: "better served"        Is there a trailing "-ing" verb OR "if"?
        (Passive Agent Trigger)          /          \
                                       YES           NO
                                       /              \
                      Fix to: "better off"          Is next word "with"/"as"/"over"/"in"?
                      (Gerund/Cond Trigger)          /          \
                                                   YES           NO
                                                   /              \
                               Fix to: "better served"         Leave alone: "neither"
                               (Instrumental Trigger)          (Literal / False Positive)
        */

        let which = if next_ch.eq_str("by") {
            KeepServed
        } else if next_tok.kind.is_verb_progressive_form() || next_ch.eq_str("if") {
            KeepOff
        } else if next_ch.eq_any_ignore_ascii_case_str(&["with", "as", "over", "in"]) {
            KeepServed
        } else if next_ch.eq_any_ignore_ascii_case_str(&["not", "just"]) {
            if followed_by_word(cttx, |t| t.kind.is_verb_progressive_form()) {
                KeepOff
            } else {
                KeepServed
            }
        } else {
            // Literal expressions like "served cold" fall through here safely
            return None;
        };

        // 0      1 2   3 4      5 6
        // better _ off _ served _ NEXTWORD
        // remove off -> remove tokens 1&2 or tokens 2&3
        // remove served -> remove tokens 3&4 or tokens 4&5

        let remove_index = match which {
            KeepOff => 3,
            KeepServed => 1,
        };

        Some(Lint {
            span: toks[remove_index..=remove_index + 1].span()?,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::Remove],
            message: "Are you confusing `better off` and `better served`?".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `better off served` to `better off` or `better served`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::BetterOffServed;

    #[test]
    fn fix_by_buffering() {
        assert_suggestion_result(
            "Chatty traffic is better off served by the AFD buffering.",
            BetterOffServed::default(),
            "Chatty traffic is better served by the AFD buffering.",
        );
    }

    #[test]
    fn fix_if_false() {
        assert_suggestion_result(
            "IMHO, most use cases would be far better off served if drag-content=false would imply that the user would still be able drag the main content.",
            BetterOffServed::default(),
            "IMHO, most use cases would be far better off if drag-content=false would imply that the user would still be able drag the main content.",
        );
    }

    #[test]
    fn fix_over_tls() {
        assert_suggestion_result(
            "This website is serving a curl|sh command and probably better off served over TLS with HSTS/cert pinning.",
            BetterOffServed::default(),
            "This website is serving a curl|sh command and probably better served over TLS with HSTS/cert pinning.",
        );
    }

    #[test]
    fn fix_contributing() {
        assert_suggestion_result(
            "Don't start it if your code would be better off served contributing to another project where you could ...",
            BetterOffServed::default(),
            "Don't start it if your code would be better off contributing to another project where you could ...",
        );
    }

    #[test]
    fn fix_not_using() {
        assert_suggestion_result(
            "IMHO you are also better off served NOT using a single frame and trying to shove everything into it.",
            BetterOffServed::default(),
            "IMHO you are also better off NOT using a single frame and trying to shove everything into it.",
        );
    }

    #[test]
    fn fix_just_doing() {
        assert_suggestion_result(
            "SSR is neat but I feel like you're better off served just doing it with something else",
            BetterOffServed::default(),
            "SSR is neat but I feel like you're better off just doing it with something else",
        );
    }

    #[test]
    fn fix_as_a_comeback_mechanic() {
        assert_suggestion_result(
            "Personally I think the bonus recruitments would be better off served as a comeback mechanic that triggers when ...",
            BetterOffServed::default(),
            "Personally I think the bonus recruitments would be better served as a comeback mechanic that triggers when ...",
        );
    }

    #[test]
    fn fix_with_a_different_tool() {
        assert_suggestion_result(
            "The second thing you are asking about is likely better off served with a different tool.",
            BetterOffServed::default(),
            "The second thing you are asking about is likely better served with a different tool.",
        );
    }

    #[test]
    fn fix_planning_and_executing() {
        assert_suggestion_result(
            "You are better off served planning and executing some meaningful training.",
            BetterOffServed::default(),
            "You are better off planning and executing some meaningful training.",
        );
    }

    #[test]
    fn ignore_served_cold() {
        assert_no_lints(
            "All movies are better off served cold.",
            BetterOffServed::default(),
        );
    }

    // You might be better off served by making Automod hold all posts and/or comments.
    #[test]
    fn fix_making_automod() {
        assert_suggestion_result(
            "You might be better off served by making Automod hold all posts and/or comments.",
            BetterOffServed::default(),
            "You might be better served by making Automod hold all posts and/or comments.",
        );
    }

    // Given the high consistency of the negative feedback, I'd say you're better off served looking into putting that cash into another game ...
    #[test]
    fn fix_looking_into() {
        assert_suggestion_result(
            "Given the high consistency of the negative feedback, I'd say you're better off served looking into putting that cash into another game ...",
            BetterOffServed::default(),
            "Given the high consistency of the negative feedback, I'd say you're better off looking into putting that cash into another game ...",
        );
    }

    // Otherwise, you're better off served with specialist devices.
    #[test]
    fn fix_with_devices() {
        assert_suggestion_result(
            "Otherwise, you're better off served with specialist devices.",
            BetterOffServed::default(),
            "Otherwise, you're better served with specialist devices.",
        );
    }

    // However, once they are learned, we are better off served by turning our focus forward.
    #[test]
    fn fix_by_turning() {
        assert_suggestion_result(
            "However, once they are learned, we are better off served by turning our focus forward.",
            BetterOffServed::default(),
            "However, once they are learned, we are better served by turning our focus forward.",
        );
    }

    // This company is better off/served without you.
    #[test]
    fn ignore_slash() {
        assert_no_lints(
            "This company is better off/served without you.",
            BetterOffServed::default(),
        );
    }

    // If so that doesn't belong in the table, and would be better off served in a VIEW .
    #[test]
    fn fix_in_a_view() {
        assert_suggestion_result(
            "If so that doesn't belong in the table, and would be better off served in a VIEW .",
            BetterOffServed::default(),
            "If so that doesn't belong in the table, and would be better served in a VIEW .",
        );
    }
}
