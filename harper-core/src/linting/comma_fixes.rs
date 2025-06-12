use super::{Lint, LintKind, Linter, Suggestion};
use crate::{
    Span,
    TokenKind::{Space, Unlintable, Word},
    TokenStringExt,
};

const MSG_SPACE_BEFORE: &str = "Don't use a space before a comma.";
const MSG_AVOID_ASIAN: &str = "Avoid East Asian commas in English contexts.";
const MSG_SPACE_AFTER: &str = "Use a space after a comma.";

/// A linter that fixes common comma errors:
/// No space after.
/// Inappropriate space before.
/// Asian commas instead of English commas.
/// This linter only Asian commas anywhere, and wrong spacing of commas between words.
/// Commas between numbers are used differently in different contexts and these are not checked:
/// Lists of numbers: 1, 2, 3
/// Thousands separators: 1,000,000
/// Decimal points used mistakenly by Europeans: 3,14159
#[derive(Debug, Default)]
pub struct CommaFixes;

impl Linter for CommaFixes {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for ci in document.iter_comma_indices() {
            let mut toks = (None, None, document.get_token(ci).unwrap(), None, None);
            toks.0 = (ci >= 2).then(|| document.get_token(ci - 2).unwrap());
            toks.1 = (ci >= 1).then(|| document.get_token(ci - 1).unwrap());
            toks.3 = document.get_token(ci + 1);
            toks.4 = document.get_token(ci + 2);

            let kinds = (
                toks.0.map(|t| &t.kind),
                toks.1.map(|t| &t.kind),
                *toks.2.span.get_content(source).first().unwrap(),
                toks.3.map(|t| &t.kind),
                toks.4.map(|t| &t.kind),
            );

            let (span, suggestion, message) = match kinds {
                (_, Some(Word(_)), '、' | '，', Some(Space(_)), Some(Word(_))) => (
                    toks.2.span,
                    Suggestion::ReplaceWith(vec![',']),
                    vec![MSG_AVOID_ASIAN],
                ),

                (Some(Word(_)), Some(Space(_)), ',', Some(Space(_)), Some(Word(_))) => (
                    toks.1.unwrap().span,
                    Suggestion::Remove,
                    vec![MSG_SPACE_BEFORE],
                ),

                (Some(Word(_)), Some(Space(_)), '、' | '，', Some(Space(_)), Some(Word(_))) => (
                    Span::new(toks.1.unwrap().span.start, toks.2.span.end),
                    Suggestion::ReplaceWith(vec![',']),
                    vec![MSG_SPACE_BEFORE, MSG_AVOID_ASIAN],
                ),

                (_, Some(Word(_)), ',', Some(Word(_)), _) => (
                    toks.2.span,
                    Suggestion::InsertAfter(vec![' ']),
                    vec![MSG_SPACE_AFTER],
                ),

                (_, Some(Word(_)), '、' | '，', Some(Word(_)), _) => (
                    toks.2.span,
                    Suggestion::ReplaceWith(vec![',', ' ']),
                    vec![MSG_AVOID_ASIAN, MSG_SPACE_AFTER],
                ),

                (Some(Word(_)), Some(Space(_)), ',', Some(Word(_)), _) => (
                    Span::new(toks.1.unwrap().span.start, toks.2.span.end),
                    Suggestion::ReplaceWith(vec![',', ' ']),
                    vec![MSG_SPACE_BEFORE, MSG_SPACE_AFTER],
                ),

                (Some(Word(_)), Some(Space(_)), '、' | '，', Some(Word(_)), _) => (
                    Span::new(toks.1.unwrap().span.start, toks.2.span.end),
                    Suggestion::ReplaceWith(vec![',', ' ']),
                    vec![MSG_SPACE_BEFORE, MSG_AVOID_ASIAN, MSG_SPACE_AFTER],
                ),

                // Handles Asian commas in all other contexts
                // Unlintable is used for non-English tokens to prevent changing commas in CJK text
                (_, Some(Unlintable), '、' | '，', _, _) => continue,
                (_, _, '、' | '，', Some(Unlintable), _) => continue,

                (_, _, '、' | '，', _, _) => (
                    toks.2.span,
                    Suggestion::ReplaceWith(vec![',']),
                    vec![MSG_AVOID_ASIAN],
                ),

                _ => continue,
            };

            lints.push(Lint {
                span,
                lint_kind: LintKind::Punctuation,
                suggestions: vec![suggestion],
                message: message.join(" "),
                priority: 32,
            });
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Fix common comma errors such as no space after, erroneous space before, etc., Asian commas instead of English commas, etc."
    }
}

#[cfg(test)]
mod tests {
    use super::CommaFixes;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn allows_english_comma_atomic() {
        assert_lint_count(",", CommaFixes, 0);
    }

    #[test]
    fn flags_fullwidth_comma_atomic() {
        assert_lint_count("，", CommaFixes, 1);
    }

    #[test]
    fn flags_ideographic_comma_atomic() {
        assert_lint_count("、", CommaFixes, 1);
    }

    #[test]
    fn corrects_fullwidth_comma_real_world() {
        assert_suggestion_result(
            "higher 2 bits of the number of nodes， whether abandoned or not decided by .index section",
            CommaFixes,
            "higher 2 bits of the number of nodes, whether abandoned or not decided by .index section",
        );
    }

    #[test]
    fn corrects_ideographic_comma_real_world() {
        assert_suggestion_result("cout、endl、string", CommaFixes, "cout, endl, string")
    }

    #[test]
    fn doesnt_flag_comma_space_between_words() {
        assert_lint_count("foo, bar", CommaFixes, 0);
    }

    #[test]
    fn flags_fullwidth_comma_space_between_words() {
        assert_lint_count("foo， bar", CommaFixes, 1);
    }

    #[test]
    fn flags_ideographic_comma_space_between_words() {
        assert_lint_count("foo、 bar", CommaFixes, 1);
    }

    #[test]
    fn doesnt_flag_semicolon_space_between_words() {
        assert_lint_count("foo; bar", CommaFixes, 0);
    }

    #[test]
    fn corrects_comma_between_words_with_no_space() {
        assert_suggestion_result("foo,bar", CommaFixes, "foo, bar")
    }

    #[test]
    fn corrects_asian_comma_between_words_with_no_space() {
        assert_suggestion_result("foo，bar", CommaFixes, "foo, bar")
    }

    #[test]
    fn corrects_space_on_wrong_side_of_comma_between_words() {
        assert_suggestion_result("foo ,bar", CommaFixes, "foo, bar")
    }

    #[test]
    fn corrects_comma_on_wrong_side_of_asian_comma_between_words() {
        assert_suggestion_result("foo ，bar", CommaFixes, "foo, bar")
    }

    #[test]
    fn corrects_comma_between_words_with_space_on_both_sides() {
        assert_suggestion_result("foo , bar", CommaFixes, "foo, bar")
    }

    #[test]
    fn corrects_asian_comma_between_words_with_space_on_both_sides() {
        assert_suggestion_result("foo 、 bar", CommaFixes, "foo, bar")
    }

    #[test]
    fn doesnt_correct_comma_between_non_english_tokens() {
        assert_lint_count("严禁采摘花、 果、叶，挖掘树根、草药!", CommaFixes, 0);
    }
}
