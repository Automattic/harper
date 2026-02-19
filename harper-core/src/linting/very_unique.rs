use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct VeryUnique {
    expr: Box<dyn Expr>,
}

impl Default for VeryUnique {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&[
                    "fairly", "pretty", "rather", "quite", "somewhat", "very",
                ])
                .t_ws()
                .t_aco("unique"),
            ),
        }
    }
}

impl ExprLinter for VeryUnique {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let very_unique_span = toks.span()?;
        let very_unique_chars = very_unique_span.get_content(src);
        let qualifier_tok = &toks.first()?;
        let qualifier_str = qualifier_tok.span.get_content_string(src);

        let adjectives = ["special", "rare", "unusual"];

        let suggestions = adjectives
            .iter()
            .map(|adj| {
                Suggestion::replace_with_match_case(
                    format!("{qualifier_str} {adj}").chars().collect(),
                    very_unique_chars,
                )
            })
            .chain(std::iter::once(Suggestion::replace_with_match_case(
                "unique".chars().collect(),
                very_unique_chars,
            )))
            .collect::<Vec<_>>();

        Some(Lint {
            span: very_unique_span,
            lint_kind: LintKind::WordChoice,
            suggestions,
            message: "`Unique` is absolute, so consider using `unique` alone or a more precise adjective such as `special`, `rare`, or `unusual`.".to_string(),
            priority: 57,
        })
    }

    fn description(&self) -> &str {
        "Flags phrases like `very unique`, `pretty unique`, etc., and suggests using `unique` alone or a more precise adjective such as `special`, `rare`, or `unusual`."
    }
}

#[cfg(test)]
mod tests {
    use super::VeryUnique;
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_top3_suggestion_result};

    #[test]
    fn fix_very_unique() {
        assert_good_and_bad_suggestions(
            "I'm not sure whether Llama Stack or ollama are generating the chat completion ids, but they are not very unique.",
            VeryUnique::default(),
            &[
                "I'm not sure whether Llama Stack or ollama are generating the chat completion ids, but they are not unique.",
            ],
            &[],
        );
    }

    #[test]
    fn fix_pretty_unique() {
        assert_top3_suggestion_result(
            "Numerous accounts with my exact full name/surname (which is pretty unique) has been created (most recently).",
            VeryUnique::default(),
            "Numerous accounts with my exact full name/surname (which is pretty rare) has been created (most recently).",
        );
    }

    #[test]
    fn fix_fairly_unique() {
        assert_good_and_bad_suggestions(
            "In browsers, the first chars are obtained from the user agent string (which is fairly unique), and the supported mimeTypes",
            VeryUnique::default(),
            &[
                "In browsers, the first chars are obtained from the user agent string (which is unique), and the supported mimeTypes",
            ],
            &[],
        );
    }

    #[test]
    fn fix_somewhat_unique() {
        assert_top3_suggestion_result(
            "A new pack of somewhat unique upgrades for R.E.P.O.!",
            VeryUnique::default(),
            "A new pack of somewhat unusual upgrades for R.E.P.O.!",
        );
    }

    #[test]
    fn fix_quite_unique() {
        assert_good_and_bad_suggestions(
            "Now I understand that this is quite unique to insta and if it's not useful I am also going to investigate alternatives",
            VeryUnique::default(),
            &[
                "Now I understand that this is unique to insta and if it's not useful I am also going to investigate alternatives",
            ],
            &[],
        );
    }

    #[test]
    fn fix_rather_unique() {
        assert_top3_suggestion_result(
            "I regret using the Vue compiler because the resulting AST is rather unique.",
            VeryUnique::default(),
            "I regret using the Vue compiler because the resulting AST is rather unusual.",
        );
    }
}
