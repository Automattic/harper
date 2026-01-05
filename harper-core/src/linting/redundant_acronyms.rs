use crate::{
    CharStringExt, Token,
    expr::{Expr, FirstMatchOf, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
    token_string_ext::TokenStringExt,
};

// (acronym, first_words, last_word)
const ACRONYMS: &[(&str, &[&str], &str)] = &[
    ("ATM", &["automated teller", "automatic teller"], "machine"),
    ("GUI", &["graphical user"], "interface"),
    ("LCD", &["liquid crystal"], "display"),
    // Note: "pin number" (not capitalized) is used to refer to GPIO pins etc.
    ("PIN", &["personal identification"], "number"),
    ("TUI", &["text-based user", "terminal user"], "interface"),
    ("UI", &["user"], "interface"),
    ("VIN", &["vehicle identification"], "number"),
];

pub struct RedundantAcronyms {
    expr: Box<dyn Expr>,
}

impl Default for RedundantAcronyms {
    fn default() -> Self {
        let exprs: Vec<Box<dyn Expr>> = ACRONYMS
            .iter()
            .map(|&(acronym, _, last_str)| {
                let last_string = last_str.to_string();
                Box::new(SequenceExpr::aco(acronym).t_ws().then_any_of(vec![
                    Box::new(Word::new(last_str)),
                    Box::new(move |t: &Token, src: &[char]| {
                        t.span
                            .get_content(src)
                            .eq_ignore_ascii_case_str(&format!("{last_string}s"))
                    }),
                ])) as Box<dyn Expr>
            })
            .collect();

        Self {
            expr: Box::new(FirstMatchOf::new(exprs)),
        }
    }
}

impl ExprLinter for RedundantAcronyms {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let last_word_span = toks.last()?.span;
        let last_word_chars = last_word_span.get_content(src);
        let acronym_str = toks.first()?.span.get_content_string(src);

        // "pin number" (lowercase) is used to refer to the pins on microchips, etc.
        if acronym_str.eq_ignore_ascii_case("PIN") && acronym_str != "PIN" {
            return None;
        }

        let (_, middle_words, _) = ACRONYMS
            .iter()
            .find(|(a, _, _)| (*a).eq_ignore_ascii_case(&acronym_str))?;

        let is_all_caps = last_word_chars
            .iter()
            .all(|c| c.is_ascii_alphabetic() && c.is_ascii_uppercase());

        let plural_ending = last_word_chars
            .last()
            .filter(|&&c| c.eq_ignore_ascii_case(&'s'))
            .map(|c| c.to_string())
            .unwrap_or_default();

        let suggestions: Vec<Suggestion> = std::iter::once(Suggestion::ReplaceWith(
            format!("{acronym_str}{plural_ending}").chars().collect(),
        ))
        .chain(middle_words.iter().map(|mw| {
            let middle_words = if is_all_caps {
                mw.to_ascii_uppercase()
            } else {
                mw.to_string()
            };
            Suggestion::ReplaceWith(
                format!("{middle_words} {}", last_word_span.get_content_string(src))
                    .chars()
                    .collect(),
            )
        }))
        .collect();

        Some(Lint {
        span: toks.span()?,
        lint_kind: LintKind::Redundancy,
        suggestions,
        message: "The acronym's last letter already stands for the last word. Use just the acronym or the full phrase.".to_string(),
        ..Default::default()
    })
    }

    fn description(&self) -> &str {
        "Identifies redundant acronyms where the last word repeats the last letter's meaning (e.g., `ATM machine` → `ATM` or `automated teller machine`)."
    }
}

#[cfg(test)]
mod tests {
    use super::RedundantAcronyms;
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_no_lints};

    #[test]
    fn test_made_up() {
        assert_good_and_bad_suggestions(
            "I forgot my PIN number!",
            RedundantAcronyms::default(),
            &[
                "I forgot my PIN!",
                "I forgot my personal identification number!",
            ],
            &[],
        );
    }

    #[test]
    fn test_all_caps_singular() {
        assert_good_and_bad_suggestions(
            "CAN TWO CARS HAVE THE SAME VIN NUMBER?",
            RedundantAcronyms::default(),
            &[
                "CAN TWO CARS HAVE THE SAME VIN?",
                "CAN TWO CARS HAVE THE SAME VEHICLE IDENTIFICATION NUMBER?",
            ],
            &[],
        );
    }

    #[test]
    fn test_all_caps_plural() {
        assert_good_and_bad_suggestions(
            "THESE ATM MACHINES ALL HAVE HIGH FEES!",
            RedundantAcronyms::default(),
            &[
                "THESE ATMS ALL HAVE HIGH FEES!",
                "THESE AUTOMATED TELLER MACHINES ALL HAVE HIGH FEES!",
            ],
            &[],
        );
    }

    #[test]
    fn test_all_lowercase_singular() {
        assert_good_and_bad_suggestions(
            "the atm machine at my card",
            RedundantAcronyms::default(),
            &[
                "the atm at my card",
                "the automated teller machine at my card",
            ],
            &[],
        );
    }

    #[test]
    fn test_all_lowercase_plural() {
        assert_good_and_bad_suggestions(
            "gui interfaces were sooo trendy in 1984!",
            RedundantAcronyms::default(),
            &[
                "guis were sooo trendy in 1984!",
                "graphical user interfaces were sooo trendy in 1984!",
            ],
            &[],
        );
    }

    #[test]
    fn correct_atm_machine() {
        assert_good_and_bad_suggestions(
            "Developed an ATM machine application for Raspberry Pi",
            RedundantAcronyms::default(),
            &[
                "Developed an ATM application for Raspberry Pi",
                "Developed an automatic teller machine application for Raspberry Pi",
                "Developed an automated teller machine application for Raspberry Pi",
            ],
            &[],
        );
    }

    #[test]
    fn correct_atm_machines() {
        assert_good_and_bad_suggestions(
            "ATM machines allow 4 or 6 digit PIN codes",
            RedundantAcronyms::default(),
            &[
                "ATMs allow 4 or 6 digit PIN codes",
                "automated teller machines allow 4 or 6 digit PIN codes",
                "automatic teller machines allow 4 or 6 digit PIN codes",
            ],
            &[],
        );
    }

    #[test]
    fn correct_gui_interface() {
        assert_good_and_bad_suggestions(
            "This project develops using java language with GUI interface.",
            RedundantAcronyms::default(),
            &[
                "This project develops using java language with GUI.",
                "This project develops using java language with graphical user interface.",
            ],
            &[],
        );
    }

    #[test]
    fn correct_gui_interfaces() {
        assert_good_and_bad_suggestions(
            "In non-crafting GUI interfaces, such as a mod's own recipe tree, the shortcut key cannot be used to view item usage or crafting methods.",
            RedundantAcronyms::default(),
            &[
                "In non-crafting GUIs, such as a mod's own recipe tree, the shortcut key cannot be used to view item usage or crafting methods.",
                "In non-crafting graphical user interfaces, such as a mod's own recipe tree, the shortcut key cannot be used to view item usage or crafting methods.",
            ],
            &[],
        );
    }

    #[test]
    fn correct_lcd_display() {
        assert_good_and_bad_suggestions(
            "This function accepts I2C shield address for LCD display, number of columns, rows and dot size",
            RedundantAcronyms::default(),
            &[
                "This function accepts I2C shield address for LCD, number of columns, rows and dot size",
                "This function accepts I2C shield address for liquid crystal display, number of columns, rows and dot size",
            ],
            &[],
        );
    }

    #[test]
    fn correct_lcd_displays() {
        assert_good_and_bad_suggestions(
            "ScreenUi makes it easy to build simple or complex character based user interfaces on small LCD displays like those commonly used with Arduinos.",
            RedundantAcronyms::default(),
            &[
                "ScreenUi makes it easy to build simple or complex character based user interfaces on small LCDs like those commonly used with Arduinos.",
                "ScreenUi makes it easy to build simple or complex character based user interfaces on small liquid crystal displays like those commonly used with Arduinos.",
            ],
            &[],
        );
    }

    #[test]
    fn correct_pin_numbers_caps() {
        assert_good_and_bad_suggestions(
            "Randomly generating PIN numbers for ATM access.",
            RedundantAcronyms::default(),
            &[
                "Randomly generating PINs for ATM access.",
                "Randomly generating personal identification numbers for ATM access.",
            ],
            &[],
        );
    }

    #[test]
    fn correct_pin_number_all_caps() {
        assert_good_and_bad_suggestions(
            "DON'T LET ANYONE SEE YOUR PIN NUMBER",
            RedundantAcronyms::default(),
            &[
                "DON'T LET ANYONE SEE YOUR PIN",
                "DON'T LET ANYONE SEE YOUR PERSONAL IDENTIFICATION NUMBER",
            ],
            &[],
        );
    }

    #[test]
    fn dont_correct_pin_number_lowercase() {
        assert_no_lints(
            "GPIO 26 (pin 37) on the Pi4 is mapped to pin nummer GPIO 425 on the pi5",
            RedundantAcronyms::default(),
        );
    }

    #[test]
    fn dont_correct_pin_number_titlecase() {
        assert_no_lints(
            "Pin Number Match Project in Javascript.",
            RedundantAcronyms::default(),
        )
    }

    #[test]
    fn correct_tui_interface() {
        assert_good_and_bad_suggestions(
            "Could a history search TUI interface be added for xonsh?",
            RedundantAcronyms::default(),
            &[
                "Could a history search TUI be added for xonsh?",
                "Could a history search text-based user interface be added for xonsh?",
                "Could a history search terminal user interface be added for xonsh?",
            ],
            &[],
        );
    }

    #[test]
    fn correct_ui_interface() {
        assert_good_and_bad_suggestions(
            "call ESPUI.begin(\"Some Title\"); to start the UI interface",
            RedundantAcronyms::default(),
            &[
                "call ESPUI.begin(\"Some Title\"); to start the UI",
                "call ESPUI.begin(\"Some Title\"); to start the user interface",
            ],
            &[],
        );
    }

    #[test]
    fn correct_vin_numbers() {
        assert_good_and_bad_suggestions(
            "That was actually accurate in decoding the VIN numbers but it costed me a 1000 USD.",
            RedundantAcronyms::default(),
            &[
                "That was actually accurate in decoding the VINs but it costed me a 1000 USD.",
                "That was actually accurate in decoding the vehicle identification numbers but it costed me a 1000 USD.",
            ],
            &[],
        );
    }

    #[test]
    fn correct_vin_number() {
        assert_good_and_bad_suggestions(
            "we have implemented verification algorithms, which ensure that VIN number received is correct",
            RedundantAcronyms::default(),
            &[
                "we have implemented verification algorithms, which ensure that VIN received is correct",
                "we have implemented verification algorithms, which ensure that vehicle identification number received is correct",
            ],
            &[],
        );
    }
}
