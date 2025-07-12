use crate::{
    CharStringExt, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct AdjectiveDoubleDegree {
    expr: Box<dyn Expr>,
}

impl Default for AdjectiveDoubleDegree {
    fn default() -> Self {
        Self {
            expr: Box::new(SequenceExpr::word_set(&["more", "most"]).t_ws().then(
                |tok: &Token, _src: &[char]| {
                    tok.kind.is_comparative_adjective() || tok.kind.is_superlative_adjective()
                },
            )),
        }
    }
}

impl ExprLinter for AdjectiveDoubleDegree {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let phrase_span = toks.span()?;
        let phrase_chars = phrase_span.get_content(src);

        let adj_chars = toks.last()?.span.get_content(src);

        let (lint_kind, message, suggestions) = match (
            &toks.first()?.span.get_content(src).to_lower().as_ref(),
            toks.last()?.kind.is_comparative_adjective(),
            toks.last()?.kind.is_superlative_adjective(),
        ) {
            (['m', 'o', 'r', 'e'], true, false) => (
                LintKind::Redundancy,
                "Using `more` and the comparative form of the adjective together is redundant."
                    .to_string(),
                vec![Suggestion::replace_with_match_case(
                    adj_chars.to_vec(),
                    phrase_chars,
                )],
            ),
            (['m', 'o', 's', 't'], false, true) => (
                LintKind::Redundancy,
                "Using `most` and the superlative form of the adjective together is redundant."
                    .to_string(),
                vec![Suggestion::replace_with_match_case(
                    adj_chars.to_vec(),
                    phrase_chars,
                )],
            ),
            _ => {
                let other_adj_degree = match adj_chars {
                    &['b', 'e', 't', 't', 'e', 'r'] => vec!['b', 'e', 's', 't'],
                    &['b', 'e', 's', 't'] => vec!['b', 'e', 't', 't', 'e', 'r'],
                    &['w', 'o', 'r', 's', 'e'] => vec!['w', 'o', 'r', 's', 't'],
                    &['w', 'o', 'r', 's', 't'] => vec!['w', 'o', 'r', 's', 'e'],
                    adj_chars if adj_chars.ends_with(&['r']) => {
                        let len = adj_chars.len() + 1;
                        let mut other = vec!['\0'; len];
                        other[..len - 2].copy_from_slice(&adj_chars[..len - 2]);
                        other[len - 2] = 's';
                        other[len - 1] = 't';
                        other
                    }
                    adj_chars if adj_chars.ends_with(&['s', 't']) => {
                        let len = adj_chars.len() - 1;
                        let mut other = vec!['\0'; len];
                        other[..len - 1].copy_from_slice(&adj_chars[..len - 1]);
                        other[len - 1] = 'r';
                        other
                    }
                    _ => return None,
                };

                (
                    LintKind::WordChoice,
                    "The degree of the adverb conflicts with the degree of the adjective."
                        .to_string(),
                    vec![
                        Suggestion::replace_with_match_case(adj_chars.to_vec(), phrase_chars),
                        Suggestion::replace_with_match_case(other_adj_degree, phrase_chars),
                    ],
                )
            }
        };

        Some(Lint {
            span: phrase_span,
            lint_kind,
            message,
            suggestions,
            priority: 126,
        })
    }

    fn description(&self) -> &'static str {
        "Finds adjectives that are used as double degrees (e.g. `more prettier`)."
    }
}

#[cfg(test)]
mod tests {
    use super::AdjectiveDoubleDegree;
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_suggestion_result};

    #[test]
    fn fix_double_regular_superlative() {
        assert_suggestion_result(
            "The most easiest to use, self-service open BI reporting and BI dashboard and BI monitor screen platform.",
            AdjectiveDoubleDegree::default(),
            "The easiest to use, self-service open BI reporting and BI dashboard and BI monitor screen platform.",
        );
    }

    #[test]
    fn fix_double_regular_comparative() {
        assert_suggestion_result(
            "how can make docx gennerate more faster?",
            AdjectiveDoubleDegree::default(),
            "how can make docx gennerate faster?",
        );
    }

    #[test]
    fn fix_double_irregular_comparative() {
        assert_suggestion_result(
            "Find alternative product name more better than age .",
            AdjectiveDoubleDegree::default(),
            "Find alternative product name better than age .",
        );
    }

    #[test]
    fn fix_double_irregular_superlative() {
        assert_suggestion_result(
            "how can i get a most best quality file",
            AdjectiveDoubleDegree::default(),
            "how can i get a best quality file",
        );
    }

    #[test]
    fn conflicting_moster_offers_two_suggestions() {
        assert_good_and_bad_suggestions(
            "application which students to learn most faster in efficient way.",
            AdjectiveDoubleDegree::default(),
            &[
                "application which students to learn faster in efficient way.",
                "application which students to learn fastest in efficient way.",
            ],
            &[],
        );
    }

    #[test]
    fn conflicting_morest_offers_two_suggestions() {
        assert_good_and_bad_suggestions(
            "I suggest migrating to vite that more flexible and more fastest.",
            AdjectiveDoubleDegree::default(),
            &[
                "I suggest migrating to vite that more flexible and faster.",
                "I suggest migrating to vite that more flexible and fastest.",
            ],
            &[],
        );
    }

    #[test]
    fn conflicting_most_better_offers_two_suggestions() {
        assert_good_and_bad_suggestions(
            "But first logo is most better for me.",
            AdjectiveDoubleDegree::default(),
            &[
                "But first logo is better for me.",
                "But first logo is best for me.",
            ],
            &[],
        );
    }

    #[test]
    fn conflicting_most_worse_offers_two_suggestions() {
        assert_good_and_bad_suggestions(
            "We also see the need of a generic solution built-in in Pimcore, but currently it's probably the most worse time to implement a new solution.",
            AdjectiveDoubleDegree::default(),
            &[
                // TODO: special-case after "the" since that implies a superlative
                "We also see the need of a generic solution built-in in Pimcore, but currently it's probably the worse time to implement a new solution.",
                "We also see the need of a generic solution built-in in Pimcore, but currently it's probably the worst time to implement a new solution.",
            ],
            &[],
        );
    }
}
