use crate::{
    Lint, Token, char_string::CharStringExt, expr::{AnchorStart, Expr, FirstMatchOf, SequenceExpr}, linting::{
        ExprLinter, LintKind, Suggestion, debug::format_lint_match, expr_linter::{Sentence, at_start_of_sentence, find_the_only_token_index_matching},
    },
};

pub struct Atm {
    expr: FirstMatchOf,
}

impl Default for Atm {
    fn default() -> Self {
        Self {
            expr: FirstMatchOf::new([Box::new(
                SequenceExpr::with(AnchorStart)
                    .t_aco("atm")
                    .then_optional_whitespace()
                    .then_comma(),
            )]),
        }
    }
}

impl ExprLinter for Atm {
    type Unit = Sentence;

    fn match_to_lint_with_context(
        &self,
        toks: &[Token],
        src: &[char],
        ctx: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(toks, ctx, src));

        // For when 'atm' means 'at the moment' not at the start of a sentence
        let mut include_all_lowercase = false;
        // For when 'atm' means 'at the moment' at the start of a sentence
        let mut include_sentence_case = false;

        let atm_i =
            find_the_only_token_index_matching(toks, src, |t, s| t.get_ch(s).eq_str("atm"))?;

        eprintln!(" ℹ️ ATM is at #{}", atm_i);

        // TODO: What happens if there's whitespace before the 'atm'?
        include_sentence_case = atm_i == 0;

        // Check if atm is followed by a comma (with optional whitespace in between)
        let is_followed_by_comma = match toks.get(atm_i + 1) {
            Some(tok) if tok.kind.is_comma() => true,
            Some(tok) if tok.kind.is_whitespace() => {
                toks.get(atm_i + 2).is_some_and(|next| next.kind.is_comma())
            }
            _ => false,
        };

        // TODO: when we match more patterns, we'll want to verify that atm is anchored at the start
        // TODO: if that's even possible
        if !is_followed_by_comma {
            return None;
        }

        let mut suggs = Vec::new();
        if include_all_lowercase {
            suggs.push("at the moment");
        }
        if include_sentence_case {
            suggs.push("At the moment");
        }

        let atm_span = toks[atm_i].span;

        // let suggestions = vec![Suggestion::ReplaceWith("at the moment".chars().collect())];
        let suggestions = suggs.iter().map(|s| Suggestion::ReplaceWith(s.chars().collect())).collect();

        Some(Lint {
            span: atm_span,
            lint_kind: LintKind::Style,
            suggestions,
            message: "Use `at the moment` instead of `atm`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Expands the abbreviation `atm` to the full phrase `at the moment` for clarity."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::Atm;

    #[test]
    fn at_start_followed_by_comma() {
        assert_suggestion_result(
            "ATM, everything is OK",
            Atm::default(),
            "At the moment, everything is OK",
        );
    }

    #[test]
    fn at_start_followed_by_comma_with_extra_space() {
        assert_suggestion_result(
            "ATM , everything is OK",
            Atm::default(),
            "At the moment , everything is OK",
        );
    }

    #[test]
    fn at_start_followed_by_comma_with_wrong_space() {
        assert_suggestion_result(
            "ATM ,everything is OK",
            Atm::default(),
            "At the moment ,everything is OK",
        );
    }

    #[test]
    fn at_start_after_whitespace() {
        assert_suggestion_result(
            " ATM, everything is OK",
            Atm::default(),
            " At the moment, everything is OK",
        );
    }

    #[test]
    fn at_start_of_2nd_sentence() {
        assert_suggestion_result(
            "Hello. ATM, everything is OK",
            Atm::default(),
            "Hello. At the moment, everything is OK",
        );
    }
}
