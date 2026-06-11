use crate::linting::english::{LintKind, Linter, Suggestion};
use crate::{Document, Lint, Number, TokenStringExt};

/// Linter that checks to make sure small integers (< 10) are spelled
/// out.
#[derive(Default, Clone, Copy)]
pub struct SpelledNumbers;

impl Linter for SpelledNumbers {
    fn lint(&mut self, document: &Document) -> Vec<crate::Lint> {
        let mut lints = Vec::new();

        for number_tok in document.iter_numbers() {
            let Number {
                value,
                suffix: None,
                ..
            } = number_tok.kind.as_number().unwrap()
            else {
                continue;
            };
            let value: f64 = (*value).into();

            if (value - value.floor()).abs() < f64::EPSILON && value < 10. {
                lints.push(Lint {
                    span: number_tok.span,
                    lint_kind: LintKind::Readability,
                    suggestions: vec![Suggestion::ReplaceWith(
                        spell_out_number(value as u64).unwrap().chars().collect(),
                    )],
                    message: "Try to spell out numbers less than ten.".to_owned(),
                    priority: 63,
                })
            }
        }

        lints
    }

    fn description(&self) -> &'static str {
        "Most style guides recommend that you spell out numbers less than ten."
    }
}

/// Converts a number to its spelled-out variant.
///
/// For example: 100 -> one hundred.
///
/// Works for numbers up to 999, but can be expanded to include more powers of 10.
fn spell_out_number(num: u64) -> Option<String> {
    if num > 999 {
        return None;
    }

    match num {
        hundred if hundred % 100 == 0 && hundred > 0 => Some(format!(
            "{} hundred",
            spell_out_number(hundred / 100).unwrap()
        )),
        // Match numbers above 100 (like 110), OR two-digit numbers that don't end in 0 (like 21)
        num if num > 100 || (num > 20 && num % 10 != 0) => {
            let n = 10u64.pow((num as f32).log10() as u32);
            let parent = (num / n) * n; // truncate
            let child = num % n;

            Some(format!(
                "{}{}{}",
                spell_out_number(parent).unwrap(),
                if num <= 99 { '-' } else { ' ' },
                spell_out_number(child).unwrap()
            ))
        }
        base_num => Some(
            match base_num {
                0 => "zero",
                1 => "one",
                2 => "two",
                3 => "three",
                4 => "four",
                5 => "five",
                6 => "six",
                7 => "seven",
                8 => "eight",
                9 => "nine",
                10 => "ten",
                11 => "eleven",
                12 => "twelve",
                13 => "thirteen",
                14 => "fourteen",
                15 => "fifteen",
                16 => "sixteen",
                17 => "seventeen",
                18 => "eighteen",
                19 => "nineteen",
                20 => "twenty",
                30 => "thirty",
                40 => "forty",
                50 => "fifty",
                60 => "sixty",
                70 => "seventy",
                80 => "eighty",
                90 => "ninety",
                _ => return None,
            }
            .to_owned(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::{SpelledNumbers, spell_out_number};

    #[test]
    fn produces_zero() {
        assert_eq!(spell_out_number(0), Some("zero".to_owned()))
    }

    #[test]
    fn produces_eighty_two() {
        assert_eq!(spell_out_number(82), Some("eighty-two".to_owned()))
    }

    #[test]
    fn produces_nine_hundred_ninety_nine() {
        assert_eq!(
            spell_out_number(999),
            Some("nine hundred ninety-nine".to_owned())
        )
    }

    #[test]
    fn corrects_nine() {
        assert_suggestion_result(
            "There are 9 pigs.",
            SpelledNumbers,
            "There are nine pigs.",
            crate::languages::LanguageFamily::English,
        );
    }

    #[test]
    fn does_not_correct_ten() {
        assert_suggestion_result(
            "There are 10 pigs.",
            SpelledNumbers,
            "There are 10 pigs.",
            crate::languages::LanguageFamily::English,
        );
    }

    /// Check that the algorithm won't stack overflow or return `None` for any numbers within the specified range.
    #[test]
    fn services_range() {
        for i in 0..1000 {
            spell_out_number(i).unwrap();
        }
    }
}
