use super::{Lint, LintKind, Linter, Suggestion};
use crate::{Span, Token, TokenKind, TokenStringExt};

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

            let comma_kind = *toks.2.span.get_content(source).first().unwrap();

            let span;
            let suggestion;
            let mut remove_space_before = false;
            let mut fix_comma = false;
            let mut add_space_after = false;

            match (toks.0, toks.1, comma_kind, toks.3, toks.4) {
                (None | Some(_), Some(t1_w), _, Some(t3_s), Some(t4_w))
                    if matches!(t1_w.kind, TokenKind::Word(_))
                        && comma_kind != ','
                        && matches!(t3_s.kind, TokenKind::Space(_))
                        && matches!(t4_w.kind, TokenKind::Word(_)) =>
                {
                    span = toks.2.span;
                    suggestion = Suggestion::ReplaceWith(vec![',']);
                    fix_comma = true;
                }

                (Some(t0_w), Some(t1_s), ',', Some(t3_s), Some(t4_w))
                    if matches!(t0_w.kind, TokenKind::Word(_))
                        && matches!(t1_s.kind, TokenKind::Space(_))
                        && matches!(t3_s.kind, TokenKind::Space(_))
                        && matches!(t4_w.kind, TokenKind::Word(_)) =>
                {
                    span = toks.1.unwrap().span;
                    suggestion = Suggestion::Remove;
                    remove_space_before = true;
                }

                (Some(t0_w), Some(t1_s), _, Some(t3_s), Some(t4_w))
                    if matches!(t0_w.kind, TokenKind::Word(_))
                        && matches!(t1_s.kind, TokenKind::Space(_))
                        && comma_kind != ','
                        && matches!(t3_s.kind, TokenKind::Space(_))
                        && matches!(t4_w.kind, TokenKind::Word(_)) =>
                {
                    span = Span::new(toks.1.unwrap().span.start, toks.2.span.end);
                    suggestion = Suggestion::ReplaceWith(vec![',']);
                    remove_space_before = true;
                    fix_comma = true;
                }

                (None | Some(_), Some(t1_w), ',', Some(t3_w), None | Some(_))
                    if matches!(t1_w.kind, TokenKind::Word(_))
                        && matches!(t3_w.kind, TokenKind::Word(_)) =>
                {
                    span = toks.2.span;
                    suggestion = Suggestion::InsertAfter(vec![' ']);
                    add_space_after = true;
                }

                (None | Some(_), Some(t1_w), _, Some(t3_w), None | Some(_))
                    if matches!(t1_w.kind, TokenKind::Word(_))
                        && comma_kind != ','
                        && matches!(t3_w.kind, TokenKind::Word(_)) =>
                {
                    span = toks.2.span;
                    suggestion = Suggestion::ReplaceWith(vec![',', ' ']);
                    fix_comma = true;
                    add_space_after = true;
                }

                (Some(t0_w), Some(t1_s), ',', Some(t3_w), None | Some(_))
                    if matches!(t0_w.kind, TokenKind::Word(_))
                        && matches!(t1_s.kind, TokenKind::Space(_))
                        && matches!(t3_w.kind, TokenKind::Word(_)) =>
                {
                    span = Span::new(toks.1.unwrap().span.start, toks.2.span.end);
                    suggestion = Suggestion::ReplaceWith(vec![',', ' ']);
                    remove_space_before = true;
                    add_space_after = true;
                }

                (Some(t0_w), Some(t1_s), _, Some(t3_w), None | Some(_))
                    if matches!(t0_w.kind, TokenKind::Word(_))
                        && matches!(t1_s.kind, TokenKind::Space(_))
                        && comma_kind != ','
                        && matches!(t3_w.kind, TokenKind::Word(_)) =>
                {
                    span = Span::new(toks.1.unwrap().span.start, toks.2.span.end);
                    suggestion = Suggestion::ReplaceWith(vec![',', ' ']);
                    remove_space_before = true;
                    fix_comma = true;
                    add_space_after = true;
                }

                // Handles Asian commas in all other contexts
                // TokenKind::Unlintable is used for non-English tokens
                //  to prevent changing commas within CJK text
                (None | Some(_), None | Some(_), _, None | Some(_), None | Some(_))
                    if comma_kind != ','
                        && !matches!(
                            toks.1,
                            Some(Token {
                                kind: TokenKind::Unlintable,
                                ..
                            })
                        )
                        && !matches!(
                            toks.3,
                            Some(Token {
                                kind: TokenKind::Unlintable,
                                ..
                            })
                        ) =>
                {
                    span = toks.2.span;
                    suggestion = Suggestion::ReplaceWith(vec![',']);
                    fix_comma = true;
                }

                _ => {
                    continue;
                }
            }

            let mut message = Vec::new();
            if remove_space_before {
                message.push(MSG_SPACE_BEFORE);
            }
            if fix_comma {
                message.push(MSG_AVOID_ASIAN);
            }
            if add_space_after {
                message.push(MSG_SPACE_AFTER);
            }

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
        "Fix common comma errors such as no space after, erroneous space before, etc, Asian commas instead of English commas, etc."
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
