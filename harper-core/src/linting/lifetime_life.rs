use crate::Token;
use crate::expr::{Expr, SequenceExpr};
use crate::linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Sentence};
use crate::token_string_ext::TokenStringExt;

/// Detects `lifetime` used where `life` is meant.
///
/// After superlative adjectives (best, worst, biggest, etc.) or certain
/// intensifiers, "in my/your/their/etc. lifetime" should typically be
/// "in my/your/their/etc. life" because the sentence refers to lived
/// experience up to now, not the entire span of years.
///
/// Also catches bare "lifetime" after a superlative where "life" is
/// the conventional idiom, e.g. "best goal of my lifetime" → "best goal of my life".
///
/// Does NOT flag the well-established idioms:
/// - "once in a lifetime" / "once-in-a-lifetime"
/// - "lifetime achievement"
/// - "lifetime warranty/guarantee/access"
pub struct LifetimeLife {
    expr: SequenceExpr,
}

/// Helper: build the superlative-or-intensifier prefix.
fn superlative_prefix() -> SequenceExpr {
    SequenceExpr::any_of(vec![
        Box::new(SequenceExpr::default().then_superlative_adjective()),
        Box::new(
            SequenceExpr::default()
                .t_aco("most")
                .t_ws()
                .then_positive_adjective(),
        ),
        Box::new(SequenceExpr::word_set(&["favorite", "favourite", "top"])),
    ])
}

/// Helper: noun gap — skip non-noun tokens, then match a noun (or OOV),
/// then optional compound nouns.
fn noun_gap() -> SequenceExpr {
    SequenceExpr::default()
        .then_zero_or_more(|tok: &Token, _: &[char]| !tok.kind.is_noun() && !tok.kind.is_oov())
        .then_kind_where(|kind| (kind.is_noun() || kind.is_oov()) && !kind.is_preposition())
        .then_zero_or_more(
            SequenceExpr::default().t_ws().then_kind_where(|kind| {
                (kind.is_noun() || kind.is_oov()) && !kind.is_preposition()
            }),
        )
}

/// Build one pattern variant: superlative + noun gap + " PREP POSS lifetime"
fn build_variant(phrase: &'static str) -> SequenceExpr {
    superlative_prefix()
        .then(noun_gap())
        .then_fixed_phrase(phrase)
}

/// All the fixed-phrase suffixes we want to match after the superlative + noun gap.
const VARIANTS: &[&str] = &[
    " of my lifetime",
    " of your lifetime",
    " of his lifetime",
    " of her lifetime",
    " of its lifetime",
    " of our lifetime",
    " of their lifetime",
    " in my lifetime",
    " in your lifetime",
    " in his lifetime",
    " in her lifetime",
    " in its lifetime",
    " in our lifetime",
    " in their lifetime",
    " of lifetime",
    " in lifetime",
];

impl Default for LifetimeLife {
    fn default() -> Self {
        let patterns: Vec<Box<dyn Expr>> = VARIANTS
            .iter()
            .map(|&phrase| Box::new(build_variant(phrase)) as Box<dyn Expr>)
            .collect();

        let expr = SequenceExpr::any_of(patterns);
        Self { expr }
    }
}

impl ExprLinter for LifetimeLife {
    type Unit = Sentence;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let last = toks.last()?;
        let lifetime_span = last.span;

        // Check for exception phrases — if the matched region includes
        // "once in a lifetime" or "lifetime achievement/warranty/guarantee/access",
        // skip it.
        let matched_text = toks.span()?.get_content_string(src).to_lowercase();

        // "once in a lifetime" is a well-established idiom — don't flag it
        if matched_text.contains("once in a lifetime")
            || matched_text.contains("once-in-a-lifetime")
        {
            return None;
        }

        // "lifetime achievement/warranty/guarantee/access" are compound terms
        if matched_text.contains("lifetime achievement")
            || matched_text.contains("lifetime warranty")
            || matched_text.contains("lifetime guarantee")
            || matched_text.contains("lifetime access")
            || matched_text.contains("lifetime membership")
            || matched_text.contains("lifetime subscription")
        {
            return None;
        }

        Some(Lint {
            span: lifetime_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                "life".chars().collect(),
                lifetime_span.get_content(src),
            )],
            message: "`lifetime` refers to a span of years, but after superlatives the idiom uses `life` (lived experience).".to_string(),
            priority: 45,
        })
    }

    fn description(&self) -> &'static str {
        "Flags `lifetime` after superlatives where `life` (lived experience) is the conventional word choice."
    }
}

#[cfg(test)]
mod tests {
    use super::LifetimeLife;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn fix_best_of_my_lifetime() {
        assert_suggestion_result(
            "That was the best day of my lifetime.",
            LifetimeLife::default(),
            "That was the best day of my life.",
        );
    }

    #[test]
    fn fix_worst_of_my_lifetime() {
        assert_suggestion_result(
            "It was the worst experience of my lifetime.",
            LifetimeLife::default(),
            "It was the worst experience of my life.",
        );
    }

    #[test]
    fn fix_biggest_in_your_lifetime() {
        assert_suggestion_result(
            "This is the biggest change in your lifetime.",
            LifetimeLife::default(),
            "This is the biggest change in your life.",
        );
    }

    #[test]
    fn fix_greatest_of_his_lifetime() {
        assert_suggestion_result(
            "It was the greatest achievement of his lifetime.",
            LifetimeLife::default(),
            "It was the greatest achievement of his life.",
        );
    }

    #[test]
    fn fix_most_influential_in_our_lifetime() {
        assert_suggestion_result(
            "She is the most influential person in our lifetime.",
            LifetimeLife::default(),
            "She is the most influential person in our life.",
        );
    }

    #[test]
    fn fix_favorite_of_my_lifetime() {
        assert_suggestion_result(
            "This is my favorite book of my lifetime.",
            LifetimeLife::default(),
            "This is my favorite book of my life.",
        );
    }

    #[test]
    fn fix_top_of_her_lifetime() {
        assert_suggestion_result(
            "This ranks among the top moments of her lifetime.",
            LifetimeLife::default(),
            "This ranks among the top moments of her life.",
        );
    }

    #[test]
    fn fix_best_game_of_lifetime() {
        assert_suggestion_result(
            "This is the best game of lifetime.",
            LifetimeLife::default(),
            "This is the best game of life.",
        );
    }

    #[test]
    fn dont_flag_once_in_a_lifetime() {
        assert_lint_count(
            "It's a once in a lifetime opportunity.",
            LifetimeLife::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_lifetime_achievement() {
        assert_lint_count(
            "She won the lifetime achievement award.",
            LifetimeLife::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_lifetime_warranty() {
        assert_lint_count(
            "This product comes with a lifetime warranty.",
            LifetimeLife::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_lifetime_access() {
        assert_lint_count(
            "You get lifetime access to the course.",
            LifetimeLife::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_lifetime_in_isolation() {
        assert_lint_count(
            "The lifetime of this component is about 10 years.",
            LifetimeLife::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_lifetime_guarantee() {
        assert_lint_count(
            "We offer a lifetime guarantee on all products.",
            LifetimeLife::default(),
            0,
        );
    }

    #[test]
    fn fix_best_video_game_of_my_lifetime() {
        assert_suggestion_result(
            "Ocarina of Time is the best video game of my lifetime.",
            LifetimeLife::default(),
            "Ocarina of Time is the best video game of my life.",
        );
    }

    #[test]
    fn fix_worst_day_in_their_lifetime() {
        assert_suggestion_result(
            "It was the worst day in their lifetime.",
            LifetimeLife::default(),
            "It was the worst day in their life.",
        );
    }

    #[test]
    fn dont_flag_lifetime_membership() {
        assert_lint_count(
            "I purchased a lifetime membership.",
            LifetimeLife::default(),
            0,
        );
    }

    #[test]
    fn fix_favourite_movie_of_my_lifetime() {
        assert_suggestion_result(
            "It's my favourite movie of my lifetime.",
            LifetimeLife::default(),
            "It's my favourite movie of my life.",
        );
    }
}
