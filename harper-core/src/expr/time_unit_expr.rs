use crate::expr::LongestMatchOf;
use crate::patterns::WordSet;
use crate::{Span, Token};

use super::Expr;

/// Matches a time unit.
///
/// Matches standard units from microsecond to decade.
/// Matches other 'units' such as moment, night, weekend.
/// Matches singular and plural forms.
/// Matches possessive forms (which are also common misspellings for the plurals).
/// Matches abbreviations.
#[derive(Default)]
pub struct TimeUnitExpr {
    include_plurals_only: bool,
}

impl TimeUnitExpr {
    /// Creates a TimeUnitExpr that only matches plural time units
    pub fn plurals_only() -> Self {
        Self {
            include_plurals_only: true,
        }
    }
}

impl Expr for TimeUnitExpr {
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        if tokens.is_empty() {
            return None;
        }

        let units_definite_singular = WordSet::new(&[
            "microsecond",
            "millisecond",
            "second",
            "minute",
            "hour",
            "day",
            "week",
            "month",
            "year",
            "decade",
        ]);

        let units_definite_plural = WordSet::new(&[
            "microseconds",
            "milliseconds",
            "seconds",
            "minutes",
            "hours",
            "days",
            "weeks",
            "months",
            "years",
            "decades",
        ]);

        let units_definite_apos = WordSet::new(&[
            "microsecond's",
            "millisecond's",
            "second's",
            "minute's",
            "hour's",
            "day's",
            "week's",
            "month's",
            "year's",
            "decade's",
        ]);

        // ms
        let units_definite_abbrev = WordSet::new(&["ms"]);

        let units_other_singular = WordSet::new(&["moment", "night", "weekend"]);
        let units_other_plural = WordSet::new(&["moments", "nights", "weekends"]);
        let units_other_apos = WordSet::new(&["moment's", "night's", "weekend's"]);

        let units = if self.include_plurals_only {
            LongestMatchOf::new(vec![
                Box::new(units_definite_plural),
                Box::new(units_other_plural),
            ])
        } else {
            LongestMatchOf::new(vec![
                Box::new(units_definite_singular),
                Box::new(units_definite_plural),
                Box::new(units_other_singular),
                Box::new(units_other_plural),
                Box::new(units_definite_abbrev),
                Box::new(units_definite_apos),
                Box::new(units_other_apos),
            ])
        };

        units.run(cursor, tokens, source)
    }
}
