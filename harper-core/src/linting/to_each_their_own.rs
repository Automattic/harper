use pulldown_cmark::{Event, LinkType, Parser, Tag};

use crate::{
    Lint, Token, TokenStringExt,
    case::copy_casing,
    expr::{Expr, FixedPhrase},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Sentence, followed_by_hyphen},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Prefer {
    ToEachTheirOwn,
    EachToTheirOwn,
}

pub struct PreferToEachTheirOwn {
    expr: FixedPhrase,
}

pub struct PreferEachToTheirOwn {
    expr: FixedPhrase,
}

const TO_EACH_THEIR_OWN: &str = "to each their own";
const EACH_TO_THEIR_OWN: &str = "each to their own";

fn build_expr(flag: Prefer) -> FixedPhrase {
    match flag {
        Prefer::ToEachTheirOwn => FixedPhrase::from_phrase(EACH_TO_THEIR_OWN),
        Prefer::EachToTheirOwn => FixedPhrase::from_phrase(TO_EACH_THEIR_OWN),
    }
}

fn is_pure_adverb(tok: &Token) -> bool {
    tok.kind.is_word()
        && tok.kind.is_adverb()
        && !tok.kind.is_adjective()
        && !tok.kind.is_noun()
        && !tok.kind.is_conjunction()
        && !tok.kind.is_preposition()
        && !tok.kind.is_pronoun()
}

fn is_nominal_or_modifier_word(tok: &Token) -> bool {
    if tok.kind.is_number() || tok.kind.is_unlintable() || tok.kind.is_oov() {
        return true;
    }

    if !tok.kind.is_word() {
        return false;
    }

    let is_clause_continuation = tok.kind.is_conjunction()
        || tok.kind.is_preposition()
        || tok.kind.is_pronoun()
        || tok.kind.is_auxiliary_verb()
        || tok.kind.is_linking_verb()
        || (tok.kind.is_adverb() && !tok.kind.is_adjective() && !tok.kind.is_noun());

    !is_clause_continuation && (tok.kind.is_noun() || tok.kind.is_adjective())
}

fn starts_noun_phrase(mut rest: &[Token]) -> bool {
    while rest.first().is_some_and(|t| t.kind.is_whitespace()) {
        rest = &rest[1..];
    }

    let Some(first) = rest.first() else {
        return false;
    };

    if first.kind.is_open_round() || first.kind.is_open_square() {
        let round = first.kind.is_open_round();
        let mut depth = 1;
        let close = rest[1..].iter().position(|tok| {
            let (opens, closes) = if round {
                (tok.kind.is_open_round(), tok.kind.is_close_round())
            } else {
                (tok.kind.is_open_square(), tok.kind.is_close_square())
            };
            if opens {
                depth += 1;
            } else if closes {
                depth -= 1;
            }
            depth == 0
        });
        if let Some(close_rel) = close {
            let after_close = &rest[close_rel + 2..];
            return after_close.first().is_some_and(|t| t.kind.is_whitespace())
                && starts_noun_phrase(after_close);
        }

        return starts_noun_phrase(&rest[1..]);
    }

    if first.kind.is_quote() || first.kind.is_open_single() {
        let close = rest[1..].iter().position(|tok| {
            if first.kind.is_open_single() {
                tok.kind.is_apostrophe()
            } else {
                tok.kind.is_quote()
            }
        });
        return close.is_some_and(|close_rel| {
            let after_close = &rest[close_rel + 2..];
            after_close.first().is_some_and(|t| t.kind.is_whitespace())
                && starts_noun_phrase(after_close)
        });
    }

    if is_pure_adverb(first) {
        let after_adverb = &rest[1..];
        if !after_adverb
            .first()
            .is_some_and(|t| t.kind.is_whitespace() || t.kind.is_hyphen())
        {
            return false;
        }

        let mut next_slice = after_adverb;
        while next_slice
            .first()
            .is_some_and(|t| t.kind.is_whitespace() || t.kind.is_hyphen())
        {
            next_slice = &next_slice[1..];
        }

        if let Some(next_tok) = next_slice.first()
            && (next_tok.kind.is_verb_past_participle_form()
                || next_tok.kind.is_verb_progressive_form())
            && next_slice
                .get(1)
                .is_some_and(|t| t.kind.is_whitespace() || t.kind.is_hyphen())
            && starts_noun_phrase(&next_slice[2..])
        {
            return true;
        }

        return starts_noun_phrase(next_slice);
    }

    is_nominal_or_modifier_word(first)
        || (first.kind.is_adjective()
            && rest.get(1).is_some_and(|tok| tok.kind.is_whitespace())
            && rest.get(2).is_some_and(|tok| tok.kind.is_noun()))
}

/// Skip non-idiomatic uses where `own` modifies a following noun phrase
/// (e.g. "returned each to their own home", "each to their own (private) room",
/// or "each to their own unusually quiet room").
fn followed_by_modifier_or_noun(ctx: Option<(&[Token], &[Token])>) -> bool {
    if followed_by_hyphen(ctx) {
        return true;
    }

    let Some((_, after)) = ctx else {
        return false;
    };

    if !after.first().is_some_and(|t| t.kind.is_whitespace()) {
        return false;
    }

    starts_noun_phrase(after)
}

fn changes_reference_label(src: &[char], first: &Token, second: &Token) -> bool {
    if !src[..second.span.start].contains(&'[') || !src.windows(2).any(|chars| chars == [']', ':'])
    {
        return false;
    }

    // These links use their visible text as the lookup key for the definition.
    let source: String = src.iter().collect();
    let byte_offset = |tok: &Token| src[..tok.span.start].iter().map(|ch| ch.len_utf8()).sum();
    let first_start = byte_offset(first);
    let second_start = byte_offset(second);
    Parser::new(&source)
        .into_offset_iter()
        .any(|(event, span)| {
            matches!(
                event,
                Event::Start(Tag::Link {
                    link_type: LinkType::Shortcut | LinkType::Collapsed,
                    ..
                })
            ) && (span.contains(&first_start) || span.contains(&second_start))
        })
}

fn to_lint(
    toks: &[Token],
    src: &[char],
    ctx: Option<(&[Token], &[Token])>,
    pref: Prefer,
) -> Option<Lint> {
    if followed_by_modifier_or_noun(ctx) {
        return None;
    }

    let span = toks.span()?;

    let mut word_indices = toks
        .iter()
        .enumerate()
        .filter_map(|(i, tok)| tok.kind.is_word().then_some(i));
    let first_idx = word_indices.next()?;
    let second_idx = word_indices.next()?;

    let first_tok = &toks[first_idx];
    let second_tok = &toks[second_idx];
    if changes_reference_label(src, first_tok, second_tok) {
        return None;
    }
    let prefix = src.get(span.start..first_tok.span.start)?;
    let between = src.get(first_tok.span.end..second_tok.span.start)?;
    let rest = src.get(second_tok.span.end..span.end)?;

    let (first_word, second_word, target_phrase, source_phrase) = match pref {
        Prefer::ToEachTheirOwn => ("to", "each", TO_EACH_THEIR_OWN, EACH_TO_THEIR_OWN),
        Prefer::EachToTheirOwn => ("each", "to", EACH_TO_THEIR_OWN, TO_EACH_THEIR_OWN),
    };

    // Match casing word-by-word and preserve raw source gaps between tokens so Title Case
    // and omitted Markdown delimiters (e.g. `**Each** to their own`) stay intact.
    let replacement: Vec<char> = prefix
        .iter()
        .copied()
        .chain(copy_casing(first_tok.get_ch(src), first_word.chars()))
        .chain(between.iter().copied())
        .chain(copy_casing(second_tok.get_ch(src), second_word.chars()))
        .chain(rest.iter().copied())
        .collect();

    Some(Lint {
        span,
        lint_kind: LintKind::Usage,
        suggestions: vec![Suggestion::ReplaceWith(replacement)],
        message: format!("Use `{}` instead of `{}`.", target_phrase, source_phrase),
        ..Default::default()
    })
}

macro_rules! impl_expr_linter {
    ($name:ident, $pref:expr, $desc:expr) => {
        impl Default for $name {
            fn default() -> Self {
                Self {
                    expr: build_expr($pref),
                }
            }
        }

        impl ExprLinter for $name {
            type Unit = Sentence;

            fn description(&self) -> &str {
                $desc
            }

            fn match_to_lint_with_context(
                &self,
                toks: &[Token],
                src: &[char],
                ctx: Option<(&[Token], &[Token])>,
            ) -> Option<Lint> {
                to_lint(toks, src, ctx, $pref)
            }

            fn expr(&self) -> &dyn Expr {
                &self.expr
            }
        }
    };
}

impl_expr_linter!(
    PreferToEachTheirOwn,
    Prefer::ToEachTheirOwn,
    "Prefer `to each their own` over `each to their own`."
);

impl_expr_linter!(
    PreferEachToTheirOwn,
    Prefer::EachToTheirOwn,
    "Prefer `each to their own` over `to each their own`."
);

#[cfg(test)]
mod tests {
    use super::{PreferEachToTheirOwn, PreferToEachTheirOwn};
    use crate::linting::tests::{
        assert_lint_count, assert_lint_message, assert_markdown_suggestion_result, assert_no_lints,
        assert_suggestion_result,
    };
    use crate::{Document, linting::Linter};

    // Prefer "to each their own"

    #[test]
    fn correct_each_to_their_own_lowercase() {
        assert_suggestion_result(
            "Well, each to their own, I suppose.",
            PreferToEachTheirOwn::default(),
            "Well, to each their own, I suppose.",
        );
    }

    #[test]
    fn correct_each_to_their_own_sentence_case() {
        assert_suggestion_result(
            "Each to their own.",
            PreferToEachTheirOwn::default(),
            "To each their own.",
        );
    }

    #[test]
    fn correct_each_to_their_own_title_case() {
        assert_suggestion_result(
            "Each To Their Own",
            PreferToEachTheirOwn::default(),
            "To Each Their Own",
        );
    }

    #[test]
    fn correct_each_to_their_own_all_caps() {
        assert_suggestion_result(
            "EACH TO THEIR OWN!",
            PreferToEachTheirOwn::default(),
            "TO EACH THEIR OWN!",
        );
    }

    #[test]
    fn correct_each_to_their_own_across_newline() {
        assert_suggestion_result(
            "Well, each\nto their own.",
            PreferToEachTheirOwn::default(),
            "Well, to\neach their own.",
        );
    }

    #[test]
    fn correct_each_to_their_own_before_conjunction() {
        assert_suggestion_result(
            "Each to their own but I prefer dark mode.",
            PreferToEachTheirOwn::default(),
            "To each their own but I prefer dark mode.",
        );
    }

    #[test]
    fn correct_each_to_their_own_before_pronoun() {
        assert_suggestion_result(
            "Each to their own I guess.",
            PreferToEachTheirOwn::default(),
            "To each their own I guess.",
        );
    }

    #[test]
    fn correct_each_to_their_own_before_dash() {
        assert_suggestion_result(
            "Each to their own — I still like Vim.",
            PreferToEachTheirOwn::default(),
            "To each their own — I still like Vim.",
        );
    }

    #[test]
    fn dont_flag_to_each_their_own_when_preferred() {
        assert_no_lints(
            "To each their own, I suppose.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_each_to_their_own_before_noun() {
        assert_no_lints(
            "They returned each to their own home.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_each_to_their_own_before_adjective() {
        assert_no_lints(
            "We assigned each to their own separate room.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_each_to_their_own_before_hyphen() {
        assert_no_lints(
            "They prefer each to their own-brand snacks.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_each_to_their_own_before_parenthesized_modifier() {
        assert_no_lints(
            "We assigned each to their own (private) room.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_each_to_their_own_before_adverb_modified_noun() {
        assert_no_lints(
            "We assigned each to their own unusually quiet room.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_each_to_their_own_before_adverb_and_participle() {
        assert_no_lints(
            "We moved each to their own newly created branch.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_quoted_modifiers() {
        for text in [
            "We assigned each to their own \"private\" room.",
            "We assigned each to their own “private” room.",
            "We assigned each to their own ‘private’ room.",
        ] {
            assert_no_lints(text, PreferToEachTheirOwn::default());
        }
    }

    #[test]
    fn dont_flag_nested_parenthetical_modifiers() {
        assert_no_lints(
            "We assigned each to their own (private (but tiny)) room.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_adverb_and_ambiguous_adjective_before_noun() {
        assert_no_lints(
            "We assigned each to their own unusually round table.",
            PreferToEachTheirOwn::default(),
        );
    }

    #[test]
    fn doesnt_rename_markdown_reference_labels() {
        for text in [
            "[Each] to their own.\n\n[Each]: /choices",
            "Each [to] their own.\n\n[to]: /choices",
            "[Each to their own].\n\n[Each to their own]: /choices",
        ] {
            let doc = Document::new_markdown_default_curated(text);
            assert!(PreferToEachTheirOwn::default().lint(&doc).is_empty());
        }
    }

    #[test]
    fn correct_each_to_their_own_before_parenthetical_aside() {
        assert_suggestion_result(
            "Each to their own (I suppose).",
            PreferToEachTheirOwn::default(),
            "To each their own (I suppose).",
        );
    }

    #[test]
    fn correct_each_to_their_own_before_trailing_adverb() {
        assert_suggestion_result(
            "Each to their own indeed.",
            PreferToEachTheirOwn::default(),
            "To each their own indeed.",
        );
    }

    #[test]
    fn correct_each_to_their_own_in_parentheses() {
        assert_suggestion_result(
            "Well (each to their own).",
            PreferToEachTheirOwn::default(),
            "Well (to each their own).",
        );
    }

    #[test]
    fn preserves_markdown_emphasis_on_first_word() {
        assert_markdown_suggestion_result(
            "**Each** to their own.",
            PreferToEachTheirOwn::default(),
            "**To** each their own.",
        );
    }

    #[test]
    fn preserves_markdown_emphasis_on_second_word() {
        assert_markdown_suggestion_result(
            "Each **to** their own.",
            PreferToEachTheirOwn::default(),
            "To **each** their own.",
        );
    }

    // Prefer "each to their own"

    #[test]
    fn correct_to_each_their_own_lowercase() {
        assert_suggestion_result(
            "I guess it is to each their own.",
            PreferEachToTheirOwn::default(),
            "I guess it is each to their own.",
        );
    }

    #[test]
    fn correct_to_each_their_own_sentence_case() {
        assert_suggestion_result(
            "To each their own!",
            PreferEachToTheirOwn::default(),
            "Each to their own!",
        );
    }

    #[test]
    fn correct_to_each_their_own_title_case() {
        assert_suggestion_result(
            "To Each Their Own",
            PreferEachToTheirOwn::default(),
            "Each To Their Own",
        );
    }

    #[test]
    fn correct_to_each_their_own_all_caps() {
        assert_suggestion_result(
            "TO EACH THEIR OWN.",
            PreferEachToTheirOwn::default(),
            "EACH TO THEIR OWN.",
        );
    }

    #[test]
    fn correct_to_each_their_own_before_verb() {
        assert_suggestion_result(
            "To each their own is my motto.",
            PreferEachToTheirOwn::default(),
            "Each to their own is my motto.",
        );
    }

    #[test]
    fn dont_flag_each_to_their_own_when_preferred() {
        assert_no_lints("Each to their own, right?", PreferEachToTheirOwn::default());
    }

    #[test]
    fn dont_flag_to_each_their_own_before_noun() {
        assert_no_lints(
            "Give to each their own key.",
            PreferEachToTheirOwn::default(),
        );
    }

    #[test]
    fn dont_flag_to_each_their_own_before_parenthesized_and_adverb_modifiers() {
        assert_no_lints(
            "Give to each their own (private) key.",
            PreferEachToTheirOwn::default(),
        );
        assert_no_lints(
            "Give to each their own unusually quiet room.",
            PreferEachToTheirOwn::default(),
        );
    }

    #[test]
    fn preserves_markdown_emphasis_prefer_each_to_their_own() {
        assert_markdown_suggestion_result(
            "*To* each their own.",
            PreferEachToTheirOwn::default(),
            "*Each* to their own.",
        );
    }

    // Messages and OneOfMany / Mutual exclusivity

    #[test]
    fn lint_messages_match_expected_format() {
        assert_lint_message(
            "Each to their own.",
            PreferToEachTheirOwn::default(),
            "Use `to each their own` instead of `each to their own`.",
        );
        assert_lint_message(
            "To each their own.",
            PreferEachToTheirOwn::default(),
            "Use `each to their own` instead of `to each their own`.",
        );
    }

    #[test]
    fn by_default_one_must_be_enabled_and_one_must_be_disabled() {
        use crate::Dialect;
        use crate::spell::FstDictionary;

        assert_lint_count(
            "I say 'to each their own' and you say 'each to their own'.",
            crate::linting::LintGroup::new_curated(FstDictionary::curated(), Dialect::American),
            1,
        );
    }
}
