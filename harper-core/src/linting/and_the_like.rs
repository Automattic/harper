use crate::expr::{All, Expr, FixedPhrase, SequenceExpr};
use crate::linting::{ExprLinter, LintKind, Suggestion};
use crate::patterns::WordSet;
use crate::token_string_ext::TokenStringExt;
use crate::{Lint, Token};

pub struct AndTheLike {
    expr: Box<dyn Expr>,
}

impl Default for AndTheLike {
    fn default() -> Self {
        Self {
            expr: Box::new(All::new(vec![
                Box::new(
                    // All known variants seen in the wild, good and bad
                    SequenceExpr::word_set(&["and", "or", "an"])
                        .t_ws()
                        .then_optional(SequenceExpr::aco("the").t_ws())
                        .then_word_set(&["alike", "alikes", "like", "likes"]),
                ),
                Box::new(SequenceExpr::unless(
                    SequenceExpr::word_set(&["and", "or"])
                        .t_ws()
                        .then_any_of(vec![
                            // But not the correct variants
                            Box::new(FixedPhrase::from_phrase("the like")),
                            // And not the phrases that were coincidentally caught in the net
                            Box::new(WordSet::new(&["like", "likes"])),
                        ]),
                )),
            ])),
        }
    }
}

impl ExprLinter for AndTheLike {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let (conj, ws) = (&toks[0], &toks[1]);

        let conj = if conj.span.get_content(src)[0] == 'a' {
            "and"
        } else {
            "or"
        };

        let corrected = format!("{}{}the like", conj, ws.span.get_content_string(src));

        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Usage,
            suggestions: vec![Suggestion::replace_with_match_case(
                corrected.chars().collect(),
                toks.span()?.get_content(src),
            )],
            message: "If you intended the idiom meaning `similar things`, the correct form is with `the like`.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Corrects mistakes in `and the like` and `or the like`."
    }
}

#[cfg(test)]
mod tests {
    use super::AndTheLike;
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    #[test]
    fn dont_flag_and_the_like() {
        assert_no_lints(
            "The color of brackets and the like appears to be incorrect ...",
            AndTheLike::default(),
        );
    }

    #[test]
    fn dont_flag_or_the_like() {
        assert_no_lints(
            "Does WCAG apply only to English (or the like), or does it aim to cover all languages?",
            AndTheLike::default(),
        );
    }

    #[test]
    fn flag_an_the_likes() {
        assert_suggestion_result(
            "Allow jsSourceDir (an the likes) to refer to the project root. #5",
            AndTheLike::default(),
            "Allow jsSourceDir (and the like) to refer to the project root. #5",
        );
    }

    #[test]
    fn flag_and_alike() {
        assert_suggestion_result(
            "Latest release breaks FilePicker and alike",
            AndTheLike::default(),
            "Latest release breaks FilePicker and the like",
        );
    }

    #[test]
    fn flag_and_alikes() {
        assert_suggestion_result(
            "Compiled functions (and alikes) need to keep references for their module objects",
            AndTheLike::default(),
            "Compiled functions (and the like) need to keep references for their module objects",
        );
    }

    #[test]
    fn flag_and_the_alike() {
        assert_suggestion_result(
            "Suggestions, comments and the alike are welcome on http://waa.ai/4xtC",
            AndTheLike::default(),
            "Suggestions, comments and the like are welcome on http://waa.ai/4xtC",
        );
    }

    #[test]
    fn flag_and_the_likes() {
        assert_suggestion_result(
            "Don't report \"expected semicolon or line break\", \"expected comma\" and the likes at every token boundary",
            AndTheLike::default(),
            "Don't report \"expected semicolon or line break\", \"expected comma\" and the like at every token boundary",
        );
    }

    #[test]
    fn flag_or_alike() {
        assert_suggestion_result(
            "enable biome extension to \"monitor or alike\" the workspace.",
            AndTheLike::default(),
            "enable biome extension to \"monitor or the like\" the workspace.",
        );
    }

    #[test]
    fn flag_or_alikes() {
        assert_suggestion_result(
            "Persistent Compiler Caching with ccache or alikes",
            AndTheLike::default(),
            "Persistent Compiler Caching with ccache or the like",
        );
    }

    #[test]
    fn flag_or_the_likes() {
        assert_suggestion_result(
            "Description of the problem: Implement aria2c or the likes to resume partial downloads.",
            AndTheLike::default(),
            "Description of the problem: Implement aria2c or the like to resume partial downloads.",
        );
    }
}
