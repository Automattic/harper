use crate::{
    Lint, Token, TokenKind, TokenStringExt,
    expr::{All, Expr, SequenceExpr, SpelledNumberExpr},
    linting::{ExprLinter, LintKind, debug::format_lint_match, expr_linter::Chunk},
    patterns::WordSet,
};

pub struct NumericModifier {
    expr: SequenceExpr,
}

const PLURAL_UNITS: &str = "acres|amperes|amps|bars|bits|bowls|bucks|bytes|\
                            cents|cms|centimeters|centmetres|centuries|colors|\
                            colours|columns|cups|days|decades|decibels|\
                            deciliters|decilitres|decimeters|decimetres|degs|\
                            degrees|dollars|drops|ems|euros|feet|frames|gals|\
                            gallons|gigabits|gigabytes|gigs|glasses|handfuls|\
                            hectares|hrs|hours|inches|joules|keys|kilobits|\
                            kilobytes|kgs|kilograms|kilojoules|kilometers|\
                            kilometres|kilopascals|lightyears|lbs|megs|lines|\
                            liters|megabits|megabytes|meters|metres|mis|miles|\
                            mins|minutes|milligrams|mls|milliliters|\
                            millilitres|millimeters|millimetres|mos|months|\
                            megapascals|nanometers|nanometres|ohms|ounces|\
                            pages|parsecs|paragraphs|pascals|pence|petabits|\
                            petabytes|pixels|pounds|quarters|qts|quarts|rads|\
                            radians|rows|seasons|secs|seconds|slices|stages|\
                            strings|terabits|terabytes|ticks|tonnes|tons|volts|\
                            watts|wks|weeks|words|yds|yards|yrs|years";
impl Default for NumericModifier {
    fn default() -> Self {
        let number = SequenceExpr::any_of(vec![
            Box::new(SpelledNumberExpr),
            Box::new(SequenceExpr::default().then_cardinal_number()),
        ]);

        // e.g. "10 ‹YEARS› old laptop"
        let unit = All::new(vec![
            Box::new(WordSet::new(PLURAL_UNITS.split('|'))),
            Box::new(|t: &Token, s: &[char]| t.kind.is_plural_noun() && s.len() > 1),
        ]);

        // e.g. "10 years ‹OLD› laptop"
        let dimensional_adjective = SequenceExpr::default().then_kind_is_but_isnt_any_of_except(
            TokenKind::is_adjective,
            &[
                TokenKind::is_superlative_adjective,
                TokenKind::is_preposition,
                TokenKind::is_verb_progressive_form,
            ],
            &["it", "left", "max", "now", "paid", "per", "spent"],
        );

        // e.g. "10 years old ‹LAPTOP›"
        let noun = SequenceExpr::default().then_kind_is_but_isnt_any_of_except(
            TokenKind::is_noun,
            &[TokenKind::is_conjunction, TokenKind::is_preposition],
            &["uh"],
        );

        Self {
            expr: number
                .t_ws_h()
                .then(unit)
                .then_optional(SequenceExpr::default().t_ws_h().then(dimensional_adjective))
                .t_ws()
                .then(noun),
        }
    }
}

impl ExprLinter for NumericModifier {
    type Unit = Chunk;

    fn match_to_lint_with_context(
        &self,
        matched_tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        eprintln!("🚨 {}", format_lint_match(matched_tokens, context, source));
        let span = matched_tokens.span()?;
        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![],
            message: "👎".to_string(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Flags plural units used in modifiers."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::assert_suggestion_result;

    use super::NumericModifier;

    #[test]
    fn arguments_function() {
        assert_suggestion_result(
            "The Ackerman function is a two arguments function.",
            NumericModifier::default(),
            "The Ackerman function is a two argument function.",
        );
    }

    #[test]
    fn bytes_sequence() {
        assert_suggestion_result(
            "My laptop is a six years old Lenovo Ideapad L340-17IRH Gaming.",
            NumericModifier::default(),
            "My laptop is a six year old Lenovo Ideapad L340-17IRH Gaming.",
        );
    }

    #[test]
    fn columns_dataframe() {
        assert_suggestion_result(
            "Building a tree from a two columns data frame",
            NumericModifier::default(),
            "Building a tree from a two column data frame",
        );
    }

    #[test]
    fn days_event() {
        assert_suggestion_result(
            "That is, is 1/1/2018-2/1/2018 a two days event, or a 1 day event like in ical",
            NumericModifier::default(),
            "That is, is 1/1/2018-2/1/2018 a two day event, or a 1 day event like in ical",
        );
    }

    #[test]
    fn days_window() {
        assert_suggestion_result(
            "how much the customer paid in a seven days window",
            NumericModifier::default(),
            "how much the customer paid in a seven day window",
        );
    }

    #[test]
    fn dollars_tip() {
        assert_suggestion_result(
            "A two-dollars tip each day!",
            NumericModifier::default(),
            "A two-dollar tip each day!",
        );
    }

    #[test]
    fn feet_pole() {
        assert_suggestion_result(
            "I know someone who won't even touch six words with a ten-feet pole",
            NumericModifier::default(),
            "I know someone who won't even touch six words with a ten-foot pole",
        );
    }

    #[test]
    fn lines_program() {
        assert_suggestion_result(
            "Adding a two lines program that triggers the issue here",
            NumericModifier::default(),
            "Adding a two line program that triggers the issue here",
        );
    }

    #[test]
    fn meters_distance() {
        assert_suggestion_result(
            "in front of the sensor (at a three meters distance)",
            NumericModifier::default(),
            "in front of the sensor (at a three meter distance)",
        );
    }

    #[test]
    fn minutes_walk() {
        assert_suggestion_result(
            "Take a Ten Minutes Walk.",
            NumericModifier::default(),
            "Take a Ten Minute Walk.",
        );
    }

    #[test]
    fn parameters_function() {
        assert_suggestion_result(
            "I need to implement a two parameters function with python.",
            NumericModifier::default(),
            "I need to implement a two parameter function with python.",
        );
    }

    #[test]
    fn params_function() {
        assert_suggestion_result(
            "This is an alias over zip and then apply a two params function.",
            NumericModifier::default(),
            "This is an alias over zip and then apply a two param function.",
        );
    }

    #[test]
    fn years_old() {
        assert_suggestion_result(
            "My laptop is a six years old Lenovo Ideapad L340-17IRH Gaming.",
            NumericModifier::default(),
            "My laptop is a six year old Lenovo Ideapad L340-17IRH Gaming.",
        );
    }
}
