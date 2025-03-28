use super::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, Span, TokenStringExt};

/// Detect sequences of words of the form "adjective of a".
#[derive(Debug, Clone, Copy, Default)]
pub struct AdjectiveOfA;

impl Linter for AdjectiveOfA {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for i in document.iter_adjective_indices() {
            let adjective = document.get_token(i).unwrap();
            let space_1 = document.get_token(i + 1);
            let word_of = document.get_token(i + 2);
            let space_2 = document.get_token(i + 3);
            let a_or_an = document.get_token(i + 4);
            let adj_chars: &[char] = document.get_span_content(&adjective.span);

            // Rule out false positives

            if match adj_chars {
                // "much of a" is a different and valid construction.
                ['m', 'u', 'c', 'h'] | ['M', 'u', 'c', 'h'] => true,
                // The word is used more as a noun in this context.
                // (using .kind.is_likely_homograph() here is too strict)
                ['f', 'r', 'o', 'n', 't'] | ['F', 'r', 'o', 'n', 't'] => true,
                ['k', 'i', 'n', 'd'] | ['K', 'i', 'n', 'd'] => true,
                ['m', 'e', 'a', 'n', 'i', 'n', 'g'] | ['M', 'e', 'a', 'n', 'i', 'n', 'g'] => true,
                ['p', 'a', 'r', 't'] | ['P', 'a', 'r', 't'] => true,
                // TODO: consider "more of a" and "less of a"
                _ => false,
            } {
                continue;
            }
            // Rule out comparatives and superlatives.

            // Pros:
            // "for the better of a day"
            // "might not be the best of a given run"
            // "Which brings me to my best of a bad situation."
            //
            // Cons:
            // "see if you can give us a little better of an answer"
            // "hopefully it won't be too much worse of a problem"
            // "seems far worse of a result to me"
            let len = adj_chars.len();
            if len > 2 {
                let ending = &adj_chars[len - 2..len];
                if ending == ['e', 'r'] || ending == ['s', 't'] {
                    continue;
                }
            }
            if space_1.is_none() || word_of.is_none() || space_2.is_none() || a_or_an.is_none() {
                continue;
            }
            let space_1 = space_1.unwrap();
            if !space_1.kind.is_whitespace() {
                continue;
            }
            let word_of = word_of.unwrap();
            if !word_of.kind.is_word() {
                continue;
            }
            let word_of = document.get_span_content(&word_of.span);
            if word_of != ['o', 'f'] {
                continue;
            }
            let space_2 = space_2.unwrap();
            if !space_2.kind.is_whitespace() {
                continue;
            }
            let a_or_an = a_or_an.unwrap();
            if !a_or_an.kind.is_word() {
                continue;
            }
            let a_or_an_chars = document.get_span_content(&a_or_an.span);
            if a_or_an_chars != ['a'] && a_or_an_chars != ['a', 'n'] {
                continue;
            }

            // Whitespace may differ, add the other replacement if so
            let mut sugg_1 = Vec::new();
            sugg_1.extend_from_slice(document.get_span_content(&adjective.span));
            sugg_1.extend_from_slice(document.get_span_content(&space_1.span));
            sugg_1.extend_from_slice(document.get_span_content(&a_or_an.span));

            let mut sugg_2 = Vec::new();
            sugg_2.extend_from_slice(document.get_span_content(&adjective.span));
            sugg_2.extend_from_slice(document.get_span_content(&space_2.span));
            sugg_2.extend_from_slice(document.get_span_content(&a_or_an.span));

            let mut suggestions = vec![Suggestion::ReplaceWith(sugg_1.clone())];
            if sugg_1 != sugg_2 {
                suggestions.push(Suggestion::ReplaceWith(sugg_2));
            }

            lints.push(Lint {
                span: Span::new(adjective.span.start, a_or_an.span.end),
                lint_kind: LintKind::Style,
                suggestions,
                message: "The word `of` is not needed here.".to_string(),
                priority: 63,
            });
        }

        lints
    }

    fn description(&self) -> &str {
        "This rule looks for sequences of words of the form `adjective of a`."
    }
}

#[cfg(test)]
mod tests {
    use super::AdjectiveOfA;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn correct_large_of_a() {
        assert_suggestion_result(
            "Yeah I'm using as large of a batch size as I can on this machine",
            AdjectiveOfA,
            "Yeah I'm using as large a batch size as I can on this machine",
        )
    }

    #[test]
    fn correct_bad_of_an() {
        assert_suggestion_result(
            "- If forking is really that bad of an option, let's first decide where to put this.",
            AdjectiveOfA,
            "- If forking is really that bad an option, let's first decide where to put this.",
        );
    }

    #[test]
    fn dont_flag_comparative() {
        assert_lint_count(
            "I only worked with custom composer installers for the better of a day, so please excuse me if I missed a thing.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_superlative() {
        assert_lint_count(
            "I am trying to use composites to visualize the worst of a set of metrics.",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_kind() {
        assert_lint_count(
            "Log.txt file automatic creation in PWD is kind of an anti-feature",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_part() {
        assert_lint_count(
            "cannot delete a food that is no longer part of a recipe",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_much() {
        assert_lint_count(
            "How much of a performance impact when switching from rails to rails-api ?",
            AdjectiveOfA,
            0,
        );
    }

    #[test]
    fn dont_flag_false_positive_upper() {
        assert_lint_count(
            "Quarkus Extension as Part of a Project inside a Monorepo?",
            AdjectiveOfA,
            0,
        );
    }
}
