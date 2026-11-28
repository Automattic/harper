use crate::{
    Lint, Token, TokenStringExt,
    case::copy_casing,
    expr::{Expr, FixedPhrase},
    linting::{
        ExprLinter, LintKind, Suggestion,
        expr_linter::{Chunk, followed_by_hyphen},
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

/// Skip non-idiomatic uses where `own` modifies a following noun phrase
/// (e.g. "returned each to their own home" or "give to each their own key").
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

    let Some(next) = after.iter().find(|t| !t.kind.is_whitespace()) else {
        return false;
    };

    if next.kind.is_number() || next.kind.is_unlintable() || next.kind.is_oov() {
        return true;
    }

    if !next.kind.is_word() {
        return false;
    }

    let is_clause_continuation = next.kind.is_conjunction()
        || next.kind.is_preposition()
        || next.kind.is_pronoun()
        || next.kind.is_auxiliary_verb()
        || next.kind.is_linking_verb()
        || (next.kind.is_adverb() && !next.kind.is_adjective() && !next.kind.is_noun());

    !is_clause_continuation && (next.kind.is_noun() || next.kind.is_adjective())
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
    let between_span = toks[first_idx + 1..second_idx].span()?;
    let rest_span = toks[second_idx + 1..].span()?;

    let (first_word, second_word, target_phrase, source_phrase) = match pref {
        Prefer::ToEachTheirOwn => ("to", "each", TO_EACH_THEIR_OWN, EACH_TO_THEIR_OWN),
        Prefer::EachToTheirOwn => ("each", "to", EACH_TO_THEIR_OWN, TO_EACH_THEIR_OWN),
    };

    // Match casing word-by-word so Title Case ("Each To Their Own" -> "To Each Their Own")
    // does not shift uppercase letters across words of different lengths.
    let replacement: Vec<char> = copy_casing(first_tok.get_ch(src), first_word.chars())
        .into_iter()
        .chain(between_span.get_content(src).iter().copied())
        .chain(copy_casing(second_tok.get_ch(src), second_word.chars()))
        .chain(rest_span.get_content(src).iter().copied())
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
            type Unit = Chunk;

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
        assert_lint_count, assert_lint_message, assert_no_lints, assert_suggestion_result,
    };

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
