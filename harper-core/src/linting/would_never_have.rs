use crate::{
    Token,
    expr::{Expr, FixedPhrase, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    token_string_ext::TokenStringExt,
};

pub struct WouldNeverHave {
    expr: Box<dyn Expr>,
}

impl Default for WouldNeverHave {
    fn default() -> Self {
        let phrases = [
            "could have never",
            "could never have",
            "could've never",
            "couldve never",
            "would have never",
            "would never have",
            "would've never",
            "wouldve never",
        ];

        let expr: Vec<Box<dyn Expr>> = phrases
            .iter()
            .map(|&phrase| Box::new(FixedPhrase::from_phrase(phrase)) as Box<dyn Expr>)
            .collect();

        // TODO: verb should be perfect form ("done", "happened", etc.) when verb property changes are merged
        let expr = SequenceExpr::any_of(expr).then_whitespace().then_verb();

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for WouldNeverHave {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let modal_have_toks = toks.first()?;
        let modal_have_chars = modal_have_toks.span.get_content(src);
        let modal_have_str = modal_have_toks.span.get_content_string(src).to_lowercase();

        let modal = if modal_have_str.starts_with("could") {
            "could"
        } else if modal_have_str.starts_with("would") {
            "would"
        } else {
            return None;
        };

        let is_contraction = modal_have_str.ends_with("ve");

        let new_phrasing = format!(
            "never {modal}{}",
            if is_contraction { "'ve" } else { " have" }
        );

        let suggestions = vec![Suggestion::replace_with_match_case(
            new_phrasing.chars().collect(),
            modal_have_chars,
        )];

        let message = format!("For a more standard style, consider using `{new_phrasing}`.");

        Some(Lint {
            span: toks[..toks.len() - 2].span()?,
            lint_kind: LintKind::Style,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects `would/could have never` to `never would/could have`."
    }
}

#[cfg(test)]
mod tests {
    use super::WouldNeverHave;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn fix_could_have_never_been() {
        assert_suggestion_result(
            "Having a conversation would have never been easier with Ramen!",
            WouldNeverHave::default(),
            "Having a conversation never would have been easier with Ramen!",
        );
    }

    #[test]
    fn fix_would_have_never_come() {
        assert_suggestion_result(
            "This would have never come about without the help and encouragement of many people, too numerous to mention here.",
            WouldNeverHave::default(),
            "This never would have come about without the help and encouragement of many people, too numerous to mention here.",
        );
    }

    #[test]
    fn fix_would_have_never_find() {
        assert_suggestion_result(
            "Thanks for the help, think I would have never find it out alone.",
            WouldNeverHave::default(),
            "Thanks for the help, think I never would have find it out alone.",
        );
    }

    #[test]
    fn fix_all_caps() {
        assert_suggestion_result(
            "I WOULD'VE NEVER THOUGHT TO TEST ALL CAPS.",
            WouldNeverHave::default(),
            "I NEVER WOULD'VE THOUGHT TO TEST ALL CAPS.",
        );
    }

    #[test]
    #[ignore = "Fails due to the strange way replace_with_match_case works"]
    fn fix_title_case() {
        assert_suggestion_result(
            "I Would Never Have Thought To Test Title Case English.",
            WouldNeverHave::default(),
            "I Never Would Have Thought To Test Title Case English.",
        );
    }

    #[test]
    fn fix_could_never_have_worked() {
        assert_suggestion_result(
            "ft_quantile_discretizer could never have worked",
            WouldNeverHave::default(),
            "ft_quantile_discretizer never could have worked",
        );
    }

    #[test]
    fn fix_would_never_have_thought_of() {
        assert_suggestion_result(
            "We discover security flaws that your team would never have thought of.",
            WouldNeverHave::default(),
            "We discover security flaws that your team never would have thought of.",
        );
    }

    #[test]
    fn fix_wouldve_never_known_missing_apostrophe() {
        assert_suggestion_result(
            "We wouldve never known from the current api docs",
            WouldNeverHave::default(),
            "We never would've known from the current api docs",
        );
    }

    #[test]
    fn fix_wouldve_never_grokked() {
        assert_suggestion_result(
            "I would've never grokked that it's an issue in rollup.",
            WouldNeverHave::default(),
            "I never would've grokked that it's an issue in rollup.",
        );
    }

    #[test]
    fn fix_couldve_never_designed() {
        assert_suggestion_result(
            "Without my subscription I could've never designed this in such little time without it.",
            WouldNeverHave::default(),
            "Without my subscription I never could've designed this in such little time without it.",
        );
    }
}
