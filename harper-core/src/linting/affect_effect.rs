use std::sync::Arc;

use harper_brill::UPOS;

use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::UPOSSet,
};

pub struct AffectEffect {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for AffectEffect {
    fn default() -> Self {
        let mut map = ExprMap::default();

        // Common case: "<context> affect <word>"
        let word_follow = SequenceExpr::default()
            .then(|tok: &Token, _source: &[char]| is_preceding_context(tok))
            .t_ws()
            .then(|tok: &Token, source: &[char]| is_affect_word(tok, source))
            .t_ws()
            .then(UPOSSet::new(&[
                UPOS::AUX,
                UPOS::PROPN,
                UPOS::VERB,
                UPOS::INTJ,
                UPOS::ADP,
                UPOS::SCONJ,
                UPOS::ADJ,
            ]));

        map.insert(word_follow, 2);

        // Handle punctuation immediately after the misspelling (e.g., "affect.")
        let punctuation_follow = SequenceExpr::default()
            .then(|tok: &Token, _source: &[char]| is_preceding_context(tok))
            .t_ws()
            .then(|tok: &Token, source: &[char]| is_affect_word(tok, source))
            .then(|tok: &Token, _source: &[char]| matches!(tok.kind, TokenKind::Punctuation(_)));

        map.insert(punctuation_follow, 2);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for AffectEffect {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_idx = *self.map.lookup(0, matched_tokens, source)?;
        let target = &matched_tokens[offending_idx];

        // Avoid flagging legitimate psychological usage like "patient's affect"
        if let Some(preceding) = matched_tokens.first() {
            if preceding.kind.is_possessive_nominal() {
                return None;
            }
        }

        let token_text = target.span.get_content_string(source);
        let lower = token_text.to_lowercase();
        let replacement = match lower.as_str() {
            "affect" => "effect",
            "affects" => "effects",
            _ => return None,
        };

        Some(Lint {
            span: target.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                replacement,
                target.span.get_content(source),
            )],
            message: "`affect` is usually a verb; use `effect` here for the result or outcome."
                .into(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `affect` to `effect` when the context shows the noun meaning `result`."
    }
}

fn is_affect_word(token: &Token, source: &[char]) -> bool {
    const AFFECT: &[char] = &['a', 'f', 'f', 'e', 'c', 't'];
    const AFFECTS: &[char] = &['a', 'f', 'f', 'e', 'c', 't', 's'];

    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    let text = token.span.get_content(source);
    text.eq_ignore_ascii_case_chars(AFFECT) || text.eq_ignore_ascii_case_chars(AFFECTS)
}

fn is_preceding_context(token: &Token) -> bool {
    if token.kind.is_adverb() {
        return false;
    }

    matches!(token.kind, TokenKind::Punctuation(_))
        || token.kind.is_preposition()
        || token.kind.is_conjunction()
        || token.kind.is_proper_noun()
        || token.kind.is_verb()
        || token.kind.is_adjective()
        || token.kind.is_determiner()
        || token.kind.is_noun()
}

#[cfg(test)]
mod tests {
    use super::AffectEffect;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_because_affect_is() {
        assert_suggestion_result(
            "I worry because affect is hidden.",
            AffectEffect::default(),
            "I worry because effect is hidden.",
        );
    }

    #[test]
    fn ignores_psychology_usage() {
        assert_lint_count("The patient's affect is flat.", AffectEffect::default(), 0);
    }

    #[test]
    fn corrects_positive_affect_on() {
        assert_suggestion_result(
            "The new law had a positive affect on small businesses.",
            AffectEffect::default(),
            "The new law had a positive effect on small businesses.",
        );
    }

    #[test]
    fn corrects_affect_of() {
        assert_suggestion_result(
            "We measured the affect of caffeine on reaction time.",
            AffectEffect::default(),
            "We measured the effect of caffeine on reaction time.",
        );
    }

    #[test]
    fn corrects_side_affects() {
        assert_suggestion_result(
            "The side affects included nausea and fatigue.",
            AffectEffect::default(),
            "The side effects included nausea and fatigue.",
        );
    }

    #[test]
    fn corrects_cause_and_affect() {
        assert_suggestion_result(
            "Cause and affect are not the same thing.",
            AffectEffect::default(),
            "Cause and effect are not the same thing.",
        );
    }

    #[test]
    fn corrects_have_an_affect_on() {
        assert_suggestion_result(
            "The change will have an affect on our revenue.",
            AffectEffect::default(),
            "The change will have an effect on our revenue.",
        );
    }

    #[test]
    fn corrects_took_affect() {
        assert_suggestion_result(
            "The medicine took affect within minutes.",
            AffectEffect::default(),
            "The medicine took effect within minutes.",
        );
    }

    #[test]
    fn corrects_come_into_affect() {
        assert_suggestion_result(
            "The policy will come into affect on October 1.",
            AffectEffect::default(),
            "The policy will come into effect on October 1.",
        );
    }

    #[test]
    fn corrects_in_affect_sentence() {
        assert_suggestion_result(
            "The rules are now in affect.",
            AffectEffect::default(),
            "The rules are now in effect.",
        );
    }

    #[test]
    fn corrects_with_immediate_affect() {
        assert_suggestion_result(
            "With immediate affect, the office is closed.",
            AffectEffect::default(),
            "With immediate effect, the office is closed.",
        );
    }

    #[test]
    fn corrects_special_affects() {
        assert_suggestion_result(
            "The director used stunning special affects.",
            AffectEffect::default(),
            "The director used stunning special effects.",
        );
    }

    #[test]
    fn corrects_placebo_affect() {
        assert_suggestion_result(
            "The placebo affect can be powerful.",
            AffectEffect::default(),
            "The placebo effect can be powerful.",
        );
    }

    #[test]
    fn corrects_ripple_affect() {
        assert_suggestion_result(
            "We felt the ripple affect across the entire market.",
            AffectEffect::default(),
            "We felt the ripple effect across the entire market.",
        );
    }

    #[test]
    fn corrects_snowball_affect() {
        assert_suggestion_result(
            "The snowball affect amplified the problem.",
            AffectEffect::default(),
            "The snowball effect amplified the problem.",
        );
    }

    #[test]
    fn corrects_knock_on_affect() {
        assert_suggestion_result(
            "That decision had a knock-on affect throughout the team.",
            AffectEffect::default(),
            "That decision had a knock-on effect throughout the team.",
        );
    }

    #[test]
    fn corrects_greenhouse_affect() {
        assert_suggestion_result(
            "The greenhouse affect warms the planet.",
            AffectEffect::default(),
            "The greenhouse effect warms the planet.",
        );
    }

    #[test]
    fn corrects_little_affect() {
        assert_suggestion_result(
            "Her apology had little affect.",
            AffectEffect::default(),
            "Her apology had little effect.",
        );
    }

    #[test]
    fn corrects_go_into_affect() {
        assert_suggestion_result(
            "The new settings go into affect after a restart.",
            AffectEffect::default(),
            "The new settings go into effect after a restart.",
        );
    }

    #[test]
    fn corrects_put_plan_into_affect() {
        assert_suggestion_result(
            "They put the new plan into affect last week.",
            AffectEffect::default(),
            "They put the new plan into effect last week.",
        );
    }

    #[test]
    fn corrects_contract_into_affect() {
        assert_suggestion_result(
            "The contract comes into affect at midnight.",
            AffectEffect::default(),
            "The contract comes into effect at midnight.",
        );
    }

    #[test]
    fn corrects_no_affect_on_behavior() {
        assert_suggestion_result(
            "The warning had no affect on his behavior.",
            AffectEffect::default(),
            "The warning had no effect on his behavior.",
        );
    }

    #[test]
    fn corrects_opposite_affect() {
        assert_suggestion_result(
            "Inflation had the opposite affect than expected.",
            AffectEffect::default(),
            "Inflation had the opposite effect than expected.",
        );
    }

    #[test]
    fn corrects_remains_in_affect() {
        assert_suggestion_result(
            "The regulation remains in affect until further notice.",
            AffectEffect::default(),
            "The regulation remains in effect until further notice.",
        );
    }

    #[test]
    fn corrects_take_affect_next_week() {
        assert_suggestion_result(
            "The app changes take affect next week.",
            AffectEffect::default(),
            "The app changes take effect next week.",
        );
    }

    #[test]
    fn corrects_sound_affects() {
        assert_suggestion_result(
            "Sound affects were added in post.",
            AffectEffect::default(),
            "Sound effects were added in post.",
        );
    }

    #[test]
    fn does_not_flag_affect_as_verb() {
        assert_lint_count(
            "The change will affect our revenue significantly.",
            AffectEffect::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_affects_as_verb() {
        assert_lint_count(
            "This policy directly affects remote workers.",
            AffectEffect::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_correct_effect_noun() {
        assert_lint_count(
            "The placebo effect can be powerful.",
            AffectEffect::default(),
            0,
        );
    }

    #[test]
    fn does_not_flag_sound_effects() {
        assert_lint_count(
            "Sound effects were added in post.",
            AffectEffect::default(),
            0,
        );
    }
}
