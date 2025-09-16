use std::sync::Arc;

use harper_brill::UPOS;

use crate::{
    CharStringExt, Token, TokenKind,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{UPOSSet, WhitespacePattern},
};

pub struct EffectAffect {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for EffectAffect {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let context = SequenceExpr::default()
            .then(UPOSSet::new(&[UPOS::PART, UPOS::PUNCT, UPOS::NOUN]))
            .t_ws()
            .then(|tok: &Token, source: &[char]| is_effect_word(tok, source))
            .t_ws()
            .then(UPOSSet::new(&[
                UPOS::ADV,
                UPOS::AUX,
                UPOS::PRON,
                UPOS::PROPN,
                UPOS::VERB,
                UPOS::NUM,
                UPOS::NOUN,
                UPOS::INTJ,
                UPOS::SCONJ,
            ]))
            .then_optional(WhitespacePattern)
            .then_optional(UPOSSet::new(&[UPOS::NOUN, UPOS::PUNCT]))
            .then_optional(WhitespacePattern);

        map.insert(context, 2);

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for EffectAffect {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let offending_idx = *self.map.lookup(0, matched_tokens, source)?;
        let target = &matched_tokens[offending_idx];

        let preceding = matched_tokens[..offending_idx]
            .iter()
            .rfind(|tok| !tok.kind.is_whitespace());

        let mut following = matched_tokens[offending_idx + 1..]
            .iter()
            .filter(|tok| !tok.kind.is_whitespace());

        let first_following = following.next()?;
        let second_following = following.next();

        // Avoid "to effect change", which uses the legitimate verb "effect".
        if let Some(prev) = preceding {
            if is_token_to(prev, source) && is_change_like(first_following, source) {
                return None;
            }
        }

        // Skip when the context already shows a clear noun usage (e.g., "the effect your idea had").
        if let Some(prev) = preceding {
            if prev.kind.is_determiner() || prev.kind.is_adjective() {
                return None;
            }
        }

        // Do not flag when the following noun is clearly the result of "effect" in the idiomatic sense.
        if let Some(next) = second_following {
            if next.kind.is_noun() && is_change_like(next, source) {
                return None;
            }
        }

        let token_text = target.span.get_content_string(source);
        let lower = token_text.to_lowercase();
        let replacement = match lower.as_str() {
            "effect" => "affect",
            "effects" => "affects",
            _ => return None,
        };

        Some(Lint {
            span: target.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                replacement,
                target.span.get_content(source),
            )],
            message:
                "Use `affect` for the verb meaning to influence; `effect` usually names the result."
                    .into(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects `effect` to `affect` when the context shows the verb meaning `influence`."
    }
}

fn is_effect_word(token: &Token, source: &[char]) -> bool {
    if !matches!(token.kind, TokenKind::Word(_)) {
        return false;
    }

    const EFFECT: &[char] = &['e', 'f', 'f', 'e', 'c', 't'];
    const EFFECTS: &[char] = &['e', 'f', 'f', 'e', 'c', 't', 's'];

    let text = token.span.get_content(source);
    text.eq_ignore_ascii_case_chars(EFFECT) || text.eq_ignore_ascii_case_chars(EFFECTS)
}

fn is_token_to(token: &Token, source: &[char]) -> bool {
    token
        .span
        .get_content(source)
        .eq_ignore_ascii_case_chars(&['t', 'o'])
}

fn is_change_like(token: &Token, source: &[char]) -> bool {
    if !token.kind.is_noun() {
        return false;
    }

    matches!(
        token
            .span
            .get_content_string(source)
            .to_lowercase()
            .as_str(),
        "change" | "changes"
    )
}

#[cfg(test)]
mod tests {
    use super::EffectAffect;
    use crate::{
        Document,
        linting::{
            expr_linter::ExprLinter,
            tests::{assert_lint_count, assert_suggestion_result},
        },
    };

    #[test]
    fn corrects_noun_subject_effects_object() {
        assert_suggestion_result(
            "System outages effect our customers.",
            EffectAffect::default(),
            "System outages affect our customers.",
        );
    }

    #[test]
    fn corrects_effects_variant() {
        assert_suggestion_result(
            "This policy effects employee morale.",
            EffectAffect::default(),
            "This policy affects employee morale.",
        );
    }

    #[test]
    fn ignores_effect_change_idiom() {
        assert_lint_count(
            "Leaders work to effect change in their communities.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn ignores_effect_noun_phrase() {
        assert_lint_count(
            "The effect your plan had was dramatic.",
            EffectAffect::default(),
            0,
        );
    }

    #[test]
    fn corrects_tariff_effect_import_prices() {
        assert_suggestion_result(
            "The new tariff will effect import prices next quarter.",
            EffectAffect::default(),
            "The new tariff will affect import prices next quarter.",
        );
    }

    #[test]
    fn corrects_droughts_effect_crop_yields() {
        assert_suggestion_result(
            "Prolonged droughts severely effect crop yields across the valley.",
            EffectAffect::default(),
            "Prolonged droughts severely affect crop yields across the valley.",
        );
    }

    #[test]
    fn corrects_caffeine_effect_sleep() {
        assert_suggestion_result(
            "Caffeine can effect your sleep architecture.",
            EffectAffect::default(),
            "Caffeine can affect your sleep architecture.",
        );
    }

    #[test]
    fn corrects_bug_effect_devices() {
        assert_suggestion_result(
            "The firmware bug doesn't effect older devices.",
            EffectAffect::default(),
            "The firmware bug doesn't affect older devices.",
        );
    }

    #[test]
    fn corrects_sarcasm_effect_morale() {
        assert_suggestion_result(
            "Her sarcasm seemed to effect the team's morale.",
            EffectAffect::default(),
            "Her sarcasm seemed to affect the team's morale.",
        );
    }

    #[test]
    fn corrects_outage_effect_timeline() {
        assert_suggestion_result(
            "How will this outage effect our deployment timeline?",
            EffectAffect::default(),
            "How will this outage affect our deployment timeline?",
        );
    }

    #[test]
    fn corrects_temperatures_effect_battery() {
        assert_suggestion_result(
            "Cold temperatures drastically effect lithium-ion battery performance.",
            EffectAffect::default(),
            "Cold temperatures drastically affect lithium-ion battery performance.",
        );
    }

    #[test]
    fn corrects_policy_effect_eligibility() {
        assert_suggestion_result(
            "The policy change could effect your eligibility for benefits.",
            EffectAffect::default(),
            "The policy change could affect your eligibility for benefits.",
        );
    }

    #[test]
    fn corrects_variables_effect_results() {
        assert_suggestion_result(
            "These confounding variables may effect the study's results.",
            EffectAffect::default(),
            "These confounding variables may affect the study's results.",
        );
    }

    #[test]
    fn corrects_fans_effect_concentration() {
        assert_suggestion_result(
            "The noisy HVAC fans constantly effect concentration in the lab.",
            EffectAffect::default(),
            "The noisy HVAC fans constantly affect concentration in the lab.",
        );
    }

    #[test]
    fn corrects_hormones_effect_immunity() {
        assert_suggestion_result(
            "Stress hormones can effect immune response during recovery.",
            EffectAffect::default(),
            "Stress hormones can affect immune response during recovery.",
        );
    }

    #[test]
    fn corrects_pacing_effect_engagement() {
        assert_suggestion_result(
            "The instructor's pacing tended to effect student engagement.",
            EffectAffect::default(),
            "The instructor's pacing tended to affect student engagement.",
        );
    }

    #[test]
    fn corrects_humidity_effect_paint() {
        assert_suggestion_result(
            "Humidity levels directly effect paint curing time.",
            EffectAffect::default(),
            "Humidity levels directly affect paint curing time.",
        );
    }

    #[test]
    fn corrects_exchange_effect_invoice() {
        assert_suggestion_result(
            "The exchange rate will surely effect the final invoice.",
            EffectAffect::default(),
            "The exchange rate will surely affect the final invoice.",
        );
    }

    #[test]
    fn corrects_brightness_effect_contrast() {
        assert_suggestion_result(
            "Screen brightness settings can effect perceived contrast.",
            EffectAffect::default(),
            "Screen brightness settings can affect perceived contrast.",
        );
    }

    #[test]
    fn corrects_medication_effect_him() {
        assert_suggestion_result(
            "The medication didn't effect him the way the doctor expected.",
            EffectAffect::default(),
            "The medication didn't affect him the way the doctor expected.",
        );
    }

    #[test]
    fn corrects_payments_effect_credit() {
        assert_suggestion_result(
            "Late payments will negatively effect your credit score.",
            EffectAffect::default(),
            "Late payments will negatively affect your credit score.",
        );
    }

    #[test]
    fn corrects_wording_effect_interpretation() {
        assert_suggestion_result(
            "Minor wording tweaks shouldn't effect the legal interpretation.",
            EffectAffect::default(),
            "Minor wording tweaks shouldn't affect the legal interpretation.",
        );
    }

    #[test]
    fn corrects_traffic_effect_delivery() {
        assert_suggestion_result(
            "Traffic patterns often effect delivery windows downtown.",
            EffectAffect::default(),
            "Traffic patterns often affect delivery windows downtown.",
        );
    }

    #[test]
    fn corrects_rumor_effect_confidence() {
        assert_suggestion_result(
            "The rumor started to effect investor confidence by noon.",
            EffectAffect::default(),
            "The rumor started to affect investor confidence by noon.",
        );
    }

    #[test]
    fn corrects_allergies_effect_productivity() {
        assert_suggestion_result(
            "Seasonal allergies badly effect her productivity each April.",
            EffectAffect::default(),
            "Seasonal allergies badly affect her productivity each April.",
        );
    }

    #[test]
    fn corrects_feedback_effect_roadmap() {
        assert_suggestion_result(
            "Your feedback won't immediately effect the roadmap.",
            EffectAffect::default(),
            "Your feedback won't immediately affect the roadmap.",
        );
    }

    #[test]
    fn corrects_bandwidth_effect_video() {
        assert_suggestion_result(
            "Fluctuating bandwidth can effect video call quality.",
            EffectAffect::default(),
            "Fluctuating bandwidth can affect video call quality.",
        );
    }

    #[test]
    fn corrects_gradient_effect_sensor() {
        assert_suggestion_result(
            "The temperature gradient might effect the sensor's calibration.",
            EffectAffect::default(),
            "The temperature gradient might affect the sensor's calibration.",
        );
    }

    #[test]
    fn corrects_delays_effect_satisfaction() {
        assert_suggestion_result(
            "Even tiny delays can effect user satisfaction metrics.",
            EffectAffect::default(),
            "Even tiny delays can affect user satisfaction metrics.",
        );
    }

    #[test]
    fn corrects_architecture_effect_gps() {
        assert_suggestion_result(
            "The surrounding architecture can effect GPS accuracy.",
            EffectAffect::default(),
            "The surrounding architecture can affect GPS accuracy.",
        );
    }

    #[test]
    fn corrects_lighting_effect_color() {
        assert_suggestion_result(
            "Lighting conditions strongly effect color perception.",
            EffectAffect::default(),
            "Lighting conditions strongly affect color perception.",
        );
    }

    #[test]
    fn corrects_coach_effect_roles() {
        assert_suggestion_result(
            "The new coach's strategy will effect players' roles.",
            EffectAffect::default(),
            "The new coach's strategy will affect players' roles.",
        );
    }

    #[test]
    fn corrects_overtraining_effect_reaction() {
        assert_suggestion_result(
            "Overtraining can effect reaction time and coordination.",
            EffectAffect::default(),
            "Overtraining can affect reaction time and coordination.",
        );
    }

    #[test]
    fn corrects_label_effect_behavior() {
        assert_suggestion_result(
            "The warning label may effect how consumers use the product.",
            EffectAffect::default(),
            "The warning label may affect how consumers use the product.",
        );
    }
}
