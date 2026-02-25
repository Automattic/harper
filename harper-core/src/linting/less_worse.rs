use crate::expr::{Expr, SequenceExpr};
use crate::{CharStringExt, Token, TokenStringExt};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

pub struct LessWorse {
    expr: Box<dyn Expr>,
}

impl Default for LessWorse {
    fn default() -> Self {
        Self {
            expr: Box::new(
                SequenceExpr::word_set(&["less", "least"])
                    .t_ws_h()
                    .then_word_set(&["worse", "worst"]),
            ),
        }
    }
}

impl ExprLinter for LessWorse {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() != 3 {
            return None;
        }

        let span = toks.span()?;

        let how_little = toks[0].span.get_content(src).to_lower();
        let space_or_hyphen = &toks[1];
        let how_bad = toks[2].span.get_content(src).to_lower();

        let (suggestions, message): (&[&[char]], &str) = match (
            how_little.as_ref(),
            space_or_hyphen.kind.is_hyphen(),
            how_bad.as_ref(),
        ) {
            // "Least worst": Not standard or grammatical but idiomatic and popular.
            (['l', 'e', 'a', 's', 't'], false, ['w', 'o', 'r', 's', 't']) => (
                &[&['l', 'e', 'a', 's', 't', ' ', 'b', 'a', 'd']],
                "Though `least worst` is a common idiom, `least bad` is the standard way to compare bad options.",
            ),
            // "Least-worst": As above but also the hyphen is incorrect.
            (['l', 'e', 'a', 's', 't'], true, ['w', 'o', 'r', 's', 't']) => (
                &[
                    &['l', 'e', 'a', 's', 't', ' ', 'w', 'o', 'r', 's', 't'],
                    &['l', 'e', 'a', 's', 't', ' ', 'b', 'a', 'd'],
                ],
                "`Least worst` (without the hyphen) is a common idiom, but `least bad` is the standard way to compare bad options.",
            ),
            // About 1/3 as common as "least worst" so less acceptable as an idiom.
            (['l', 'e', 's', 's'], _, ['w', 'o', 'r', 's', 'e']) => (
                &[&['l', 'e', 's', 's', ' ', 'b', 'a', 'd']],
                "The standard way to compare bad options is `less bad`.",
            ),
            // Ambiguous. Is it supposed to be comparative or superlative?
            (['l', 'e', 's', 's'], _, ['w', 'o', 'r', 's', 't']) => (
                &[
                    &['l', 'e', 's', 's', ' ', 'b', 'a', 'd'],
                    &['l', 'e', 'a', 's', 't', ' ', 'b', 'a', 'd'],
                ],
                "These words conflict with each other. Choose `less bad` or `least bad`.",
            ),
            // Ambiguous. Probably a non-native speaker that means "least worst", but offer all three options.
            (['l', 'e', 'a', 's', 't'], _, ['w', 'o', 'r', 's', 'e']) => (
                &[
                    &['l', 'e', 'a', 's', 't', ' ', 'w', 'o', 'r', 's', 't'],
                    &['l', 'e', 'a', 's', 't', ' ', 'b', 'a', 'd'],
                    &['l', 'e', 's', 's', ' ', 'b', 'a', 'd'],
                ],
                "These words conflict with each other. Choose `less bad` or `least bad` for more standard English, or `least worst` for more idiomatic English.",
            ),
            _ => return None,
        };

        let template = span.get_content(src);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: suggestions
                .iter()
                .map(|s| Suggestion::replace_with_match_case(s.to_vec(), template))
                .collect::<Vec<_>>(),
            message: message.to_string(),
            priority: 126,
        })
    }

    fn description(&self) -> &'static str {
        "Suggests alternatives to `less/least worse/worst` for more standard, clearer comparisons."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_top3_suggestion_result};

    use super::LessWorse;

    #[test]
    fn correct_least_worse() {
        assert_good_and_bad_suggestions(
            "Maybe downstream packaging folks could advice what would be least worse option.",
            LessWorse::default(),
            &[
                "Maybe downstream packaging folks could advice what would be least worst option.",
                "Maybe downstream packaging folks could advice what would be least bad option.",
                "Maybe downstream packaging folks could advice what would be less bad option.",
            ],
            &[],
        );
    }

    #[test]
    fn correct_least_worst_hyphen() {
        assert_good_and_bad_suggestions(
            "async-dropper is probably the least-worst ad-hoc AsyncDrop implementation you've seen.",
            LessWorse::default(),
            &[
                "async-dropper is probably the least worst ad-hoc AsyncDrop implementation you've seen.",
                "async-dropper is probably the least bad ad-hoc AsyncDrop implementation you've seen.",
            ],
            &[],
        );
    }

    #[test]
    fn correct_less_worse() {
        assert_top3_suggestion_result(
            "Professionally I've convinced the team at @Roave to pay me for making their PHP code marginally less worse.",
            LessWorse::default(),
            "Professionally I've convinced the team at @Roave to pay me for making their PHP code marginally less bad.",
        );
    }

    #[test]
    fn correct_less_worst() {
        assert_good_and_bad_suggestions(
            "May be the less worst choice for some little playlists.",
            LessWorse::default(),
            &[
                "May be the less bad choice for some little playlists.",
                "May be the least bad choice for some little playlists.",
            ],
            &[],
        );
    }
}
