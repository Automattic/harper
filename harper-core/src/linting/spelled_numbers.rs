use crate::linting::{LintKind, Linter, Suggestion};
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
                    message: "Try to spell out numbers less than ten.".to_string(),
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

    Some(match num {
        0 => "zero".to_string(),
        1 => "one".to_string(),
        2 => "two".to_string(),
        3 => "three".to_string(),
        4 => "four".to_string(),
        5 => "five".to_string(),
        6 => "six".to_string(),
        7 => "seven".to_string(),
        8 => "eight".to_string(),
        9 => "nine".to_string(),
        10 => "ten".to_string(),
        11 => "eleven".to_string(),
        12 => "twelve".to_string(),
        13 => "thirteen".to_string(),
        14 => "fourteen".to_string(),
        15 => "fifteen".to_string(),
        16 => "sixteen".to_string(),
        17 => "seventeen".to_string(),
        18 => "eighteen".to_string(),
        19 => "nineteen".to_string(),
        20 => "twenty".to_string(),
        30 => "thirty".to_string(),
        40 => "forty".to_string(),
        50 => "fifty".to_string(),
        60 => "sixty".to_string(),
        70 => "seventy".to_string(),
        80 => "eighty".to_string(),
        90 => "ninety".to_string(),
        hundred if hundred % 100 == 0 => {
            format!("{} hundred", spell_out_number(hundred / 100).unwrap())
        }
        _ => {
            let n = 10u64.pow((num as f32).log10() as u32);
            let parent = (num / n) * n; // truncate
            let child = num % n;

            format!(
                "{}{}{}",
                spell_out_number(parent).unwrap(),
                if num <= 99 { '-' } else { ' ' },
                spell_out_number(child).unwrap()
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::{SpelledNumbers, spell_out_number};

    #[test]
    fn produces_zero() {
        assert_eq!(spell_out_number(0), Some("zero".to_string()))
    }

    #[test]
    fn produces_eighty_two() {
        assert_eq!(spell_out_number(82), Some("eighty-two".to_string()))
    }

    #[test]
    fn produces_nine_hundred_ninety_nine() {
        assert_eq!(
            spell_out_number(999),
            Some("nine hundred ninety-nine".to_string())
        )
    }

    #[test]
    fn corrects_nine() {
        assert_suggestion_result("There are 9 pigs.", SpelledNumbers, "There are nine pigs.");
    }

    #[test]
    fn does_not_correct_ten() {
        assert_suggestion_result("There are 10 pigs.", SpelledNumbers, "There are 10 pigs.");
    }

    /// Check that the algorithm won't stack overflow or return `None` for any numbers within the specified range.
    #[test]
    fn services_range() {
        for i in 0..1000 {
            spell_out_number(i).unwrap();
        }
    }
}
