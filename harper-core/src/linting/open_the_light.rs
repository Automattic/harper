use crate::expr::Expr;
use crate::expr::LongestMatchOf;
use crate::expr::SequenceExpr;
use crate::{
    Lrc, Token, TokenStringExt,
    linting::{LintKind, Suggestion},
};

use super::{ExprLinter, Lint};
use crate::linting::expr_linter::Chunk;

pub struct OpenTheLight {
    expr: Box<dyn Expr>,
}

impl Default for OpenTheLight {
    fn default() -> Self {
        const TO_OPEN: &[&str] = &["open", "opens", "opened", "opening"];
        const DEVICES: &[&str] = &[
            "air conditioner",
            "air conditioning",
            "aircon",
            "cellphone",
            "fan",
            "handphone",
            "heater",
            "heating",
            "lamp",
            "light",
            "lights",
            "radio",
            "telephone",
            "television",
            "TV",
        ];

        let open_the_device = Lrc::new(
            SequenceExpr::word_set(TO_OPEN)
                .t_ws()
                .then_determiner()
                .t_ws()
                .then_word_set(DEVICES),
        );

        let open_the_device_then_noun = SequenceExpr::with(open_the_device.clone())
            .t_ws()
            .then_noun();

        let expr = Box::new(LongestMatchOf::new(vec![
            Box::new(open_the_device),
            Box::new(open_the_device_then_noun),
        ]));

        Self { expr }
    }
}

impl ExprLinter for OpenTheLight {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // If I try to do this in the Pattern, the shorter pattern matches, without the context token.
        if toks.len() == 7 {
            let (device_tok, context_tok) = (toks.get_rel(-3)?, toks.get_rel(-1)?);
            // The device word is part of compound noun if it's singular and followed by another noun
            if !device_tok.kind.is_plural_noun() || !context_tok.kind.is_noun() {
                return None;
            }
        }

        const ING: &[char] = &['i', 'n', 'g'];
        const ED: &[char] = &['e', 'd'];
        const ES: &[char] = &['e', 's'];
        const LEMMA: &[char] = &[];

        let verb: &[char] = toks.first()?.span.get_content(src);

        let (e, n, d) = (
            verb[verb.len() - 3],
            verb[verb.len() - 2],
            verb[verb.len() - 1],
        );

        let (turn_ending, switch_ending) = match (e, n, d) {
            ('i', 'n', 'g') => (ING, ING),
            (_, 'e', 'd') => (ED, ED),
            (_, _, 's') => (&ES[1..], ES),
            _ => (LEMMA, LEMMA),
        };

        let mut turn_end_on: [char; 7 + 3] =
            ['t', 'u', 'r', 'n', '\0', '\0', '\0', '\0', '\0', '\0'];
        let mut switch_end_on: [char; 9 + 3] = [
            's', 'w', 'i', 't', 'c', 'h', '\0', '\0', '\0', '\0', '\0', '\0',
        ];

        // paste in the inflected ending
        turn_end_on[4..4 + turn_ending.len()].copy_from_slice(turn_ending);
        switch_end_on[6..6 + switch_ending.len()].copy_from_slice(switch_ending);

        turn_end_on[4 + turn_ending.len()..4 + turn_ending.len() + 3]
            .copy_from_slice(&[' ', 'o', 'n']);
        switch_end_on[6 + switch_ending.len()..6 + switch_ending.len() + 3]
            .copy_from_slice(&[' ', 'o', 'n']);

        let turn = &turn_end_on[..4 + turn_ending.len() + 3];
        let switch = &switch_end_on[..6 + switch_ending.len() + 3];

        Some(Lint {
            span: toks.first()?.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![
                Suggestion::replace_with_match_case(turn.to_vec(), toks.span()?.get_content(src)),
                Suggestion::replace_with_match_case(switch.to_vec(), toks.span()?.get_content(src)),
            ],
            message: "Are you accessing the device's internals or `turning` it `on`?".to_owned(),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects using `open` instead of `turn on` or `switch on`"
    }
}

#[cfg(test)]
mod tests {
    use super::OpenTheLight;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    // made-up unit tests

    #[test]
    fn fix_open_the_tv() {
        assert_suggestion_result("open the TV", OpenTheLight::default(), "turn on the TV");
    }

    #[test]
    fn fix_he_opens_the_tv() {
        assert_suggestion_result(
            "he opens the TV",
            OpenTheLight::default(),
            "he turns on the TV",
        );
    }

    #[test]
    fn fix_she_opened_the_tv() {
        assert_suggestion_result(
            "she opened the TV",
            OpenTheLight::default(),
            "she turned on the TV",
        );
    }

    #[test]
    fn opening_the_tv() {
        assert_suggestion_result(
            "opening the TV",
            OpenTheLight::default(),
            "turning on the TV",
        );
    }

    #[test]
    fn dont_flag_open_the_tv_app() {
        assert_lint_count("open the TV app", OpenTheLight::default(), 0);
    }

    #[test]
    fn fix_open_the_tv_to_watch_the_news() {
        assert_suggestion_result(
            "open the TV to watch the news",
            OpenTheLight::default(),
            "turn on the TV to watch the news",
        );
    }

    #[test]
    fn fix_dont_forget_to_open_the_lights() {
        assert_suggestion_result(
            "Don't forget to open the lights when you enter the room.",
            OpenTheLight::default(),
            "Don't forget to turn on the lights when you enter the room.",
        );
    }

    #[test]
    fn fix_can_you_open_the_fan() {
        assert_suggestion_result(
            "Can you open the fan? It's quite stuffy.",
            OpenTheLight::default(),
            "Can you turn on the fan? It's quite stuffy.",
        );
    }

    #[test]
    fn fix_opened_the_radio() {
        assert_suggestion_result(
            "I opened the radio to listen to the morning show.",
            OpenTheLight::default(),
            "I turned on the radio to listen to the morning show.",
        );
    }

    #[test]
    fn fix_open_the_aircon() {
        assert_suggestion_result(
            "Can you open the aircon? It's hot.",
            OpenTheLight::default(),
            "Can you turn on the aircon? It's hot.",
        );
    }

    #[test]
    fn dont_flag_open_the_tv_mode() {
        assert_lint_count("open the TV mode", OpenTheLight::default(), 0);
    }

    // real world examples

    #[test]
    fn dont_flag_radio_configuration() {
        assert_lint_count(
            "To open the Radio Configuration click on the three dots on the top right side.",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    #[ignore = "Requires much more complex context parsing"]
    fn dont_flag_open_the_lamp() {
        assert_lint_count(
            "Now you will need to open your lamp and solder everything together according to schematics.",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_open_tv_up_to() {
        assert_lint_count(
            "it opens the TV up to a massive library of software",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    #[ignore = "Requires more complex context parsing"]
    fn dont_flag_open_the_light_slash_sound() {
        assert_lint_count(
            "To do so, open the light/sound configuration.",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    #[ignore = "Not common enough"]
    fn dont_flag_cutting_open() {
        assert_lint_count(
            "However, instead of cutting open the lights, I opted to 3D print the Minecraft Torch Nightlight",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_open_the_light_source() {
        assert_lint_count(
            "open the light source light and regulate it to the suitable luminance",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    #[ignore = "Requires more complex context parsing"]
    fn dont_flag_opening_lamp() {
        assert_lint_count(
            "After opening the lamp, you need to solder 4 wires to the board in order to connect the USB-to-Serial adapter.",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_fan_control() {
        assert_lint_count(
            "It seems like it opens the fan control? ",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    #[ignore = "Requires more complex context parsing"]
    fn dont_flag_open_tv_to_access_eeprom() {
        assert_lint_count(
            "Involves opening your TV and directly accessing the an EEPROM IC.",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    fn dont_flag_open_tv_viewing_application() {
        assert_lint_count(
            "Open your TV viewing application or platform.",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    #[ignore = "Requires more complex context parsing"]
    fn dont_flag_open_as_noun() {
        assert_lint_count(
            "when we press open the lamp will be on",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    #[ignore = "Requires more complex context parsing"]
    fn dont_flag_opening_as_noun() {
        assert_lint_count(
            "and through that opening the light was streaming in",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    fn fix_opening_fan() {
        assert_suggestion_result(
            "If the CO2 passed a set point, it would open the fan, and close it once CO2 dropped enough.",
            OpenTheLight::default(),
            "If the CO2 passed a set point, it would turn on the fan, and close it once CO2 dropped enough.",
        );
    }

    #[test]
    fn fix_opening_tv() {
        assert_suggestion_result(
            "This was to prevent me from falling back into the temptation of opening the TV and breaking up the rule I wanted to implement.",
            OpenTheLight::default(),
            "This was to prevent me from falling back into the temptation of turning on the TV and breaking up the rule I wanted to implement.",
        );
    }

    #[test]
    #[ignore = "We don't yet handle hyphenated words"]
    fn dont_flag_opens_fan_like() {
        assert_lint_count(
            "Out by the garden fence the high ice plant opens its fan-like petals to the sun.",
            OpenTheLight::default(),
            0,
        );
    }

    #[test]
    fn fix_opening_lights() {
        assert_suggestion_result(
            "Steering wheel remains blocked until I open my lights.",
            OpenTheLight::default(),
            "Steering wheel remains blocked until I turn on my lights.",
        );
    }
}
