/// Corrects typewriter-style symbols used in measurements to their proper Unicode equivalents.
///
/// Handles patterns like:
/// - Feet/inches: `6'7"` → `6′7″`
/// - Degrees/minutes: `48°51'` → `48°51′`
///
/// Corrects both ASCII and smart quotes/apostrophes when there's strong context
/// they represent measurements, since these symbols have many other legitimate
/// uses in English text (quotes, contractions, etc.).
use crate::{
    Lint, Token, TokenKind, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct FootInchMinuteSecondSymbols {
    expr: SequenceExpr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolPattern {
    /// Pattern: number + apostrophe/quote + number + quote
    /// Examples: `6'7"`, `5' 11"`, `6'7"`
    SinglePrimeThenDoublePrime,

    /// Pattern: number + degree + number + apostrophe
    /// Examples: `48°51'`, `40° 41'`
    DegreeThenSinglePrime,
}
use SymbolPattern::*;

impl SymbolPattern {
    /// Check if a token is an actual single prime symbol or a character often used as one.
    fn is_single_prime_equiv(token: &Token) -> bool {
        token.kind.is_apostrophe() || token.kind.is_open_single() || token.kind.is_single_prime()
    }
    /// Check if a token is an actual double prime symbol or a character often used as one.
    fn is_double_prime_equiv(token: &Token) -> bool {
        token.kind.is_quote() || token.kind.is_double_prime()
    }
    /// Check if a token is already one of the correct Unicode degree or prime symbols.
    fn is_a_correct_symbol(token: &Token) -> bool {
        token.kind.is_degree() || token.kind.is_single_prime() || token.kind.is_double_prime()
    }
    /// Determine the pattern by examining the last token in the sequence.
    ///
    /// The last token tells us the pattern type:
    /// - Quote (`"`) or double prime (`″`) → feet/inches or minutes/seconds pattern
    /// - Apostrophe (`'`) or smart apostrophe (`'`) → degrees/minutes pattern
    fn from_last_token(token: &Token) -> Option<Self> {
        if Self::is_double_prime_equiv(token) {
            Some(SinglePrimeThenDoublePrime)
        } else if Self::is_single_prime_equiv(token) {
            Some(DegreeThenSinglePrime)
        } else {
            None
        }
    }

    /// Get the Unicode replacement symbols for this pattern.
    ///
    /// Returns (first_symbol, second_symbol) where:
    /// - For feet/inches: (prime, double_prime)
    /// - For degrees/minutes: (degree, prime)
    fn replacement_symbols(self) -> (char, char) {
        match self {
            SinglePrimeThenDoublePrime => ('′', '″'),
            DegreeThenSinglePrime => ('°', '′'),
        }
    }

    /// Check if a token matches the expected first symbol for this pattern.
    ///
    /// For feet/inches patterns, we accept apostrophe (regular or smart) or prime.
    /// For degrees/minutes patterns, we expect a degree symbol.
    fn matches_first_symbol(self, token: &Token) -> bool {
        match self {
            SinglePrimeThenDoublePrime => Self::is_single_prime_equiv(token),
            DegreeThenSinglePrime => token.kind.is_degree(),
        }
    }
}

impl Default for FootInchMinuteSecondSymbols {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                // Match a pair of numbers followed by symbols in a correct sequence (e.g., 5'6" or 45°30')
                // When all three are used, each half will match separately to keep the code simple.
                .then_cardinal_number()
                .t_ows()
                .then_any_of(vec![
                    // Either: feet then inches / minutes then seconds, wrong symbols or primes.
                    Box::new(
                        SequenceExpr::default()
                            .then_kind_any(&[
                                TokenKind::is_apostrophe,
                                TokenKind::is_open_single,
                                TokenKind::is_single_prime,
                            ] as &[_])
                            .t_ows()
                            .then_cardinal_number()
                            .t_ows()
                            .then_kind_either(TokenKind::is_quote, TokenKind::is_double_prime),
                    ),
                    // Or: degrees then minutes. Since degree is always the right symbol, we only need to check for wrong minute symbols.
                    Box::new(
                        SequenceExpr::default()
                            .then_degree()
                            .t_ows()
                            .then_cardinal_number()
                            .t_ows()
                            .then_kind_either(TokenKind::is_apostrophe, TokenKind::is_open_single),
                    ),
                ]),
        }
    }
}

impl ExprLinter for FootInchMinuteSecondSymbols {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // We need at least 4 tokens: number + symbol + number + symbol
        if toks.len() < 4 {
            return None;
        }

        // The last token determines which pattern we're dealing with
        let minor_symbol_idx = toks.len() - 1;
        let pattern = SymbolPattern::from_last_token(&toks[minor_symbol_idx])?;

        // Find the first symbol (token 1 or 2, depending on whether there's whitespace)
        let major_symbol_idx = if pattern.matches_first_symbol(&toks[1]) {
            1
        } else {
            2
        };

        // Skip if both symbols are already correct Unicode symbols
        if [major_symbol_idx, minor_symbol_idx]
            .iter()
            .all(|&idx| SymbolPattern::is_a_correct_symbol(&toks[idx]))
        {
            return None;
        }

        // Extract the span from first symbol to last symbol
        let symbol_span = &toks[major_symbol_idx..=minor_symbol_idx];

        // Preserve any whitespace between the symbols for natural replacement
        let inner_whitespace = &symbol_span[1..symbol_span.len() - 1];

        let (span, between) = (symbol_span.span()?, inner_whitespace.span()?);

        let (major, minor) = pattern.replacement_symbols();

        // Build the replacement: first symbol + preserved whitespace + second symbol
        let replacement = std::iter::once(major)
            .chain(between.get_content(src).iter().copied())
            .chain(std::iter::once(minor))
            .collect::<Vec<char>>();

        let message = match pattern {
            SinglePrimeThenDoublePrime => "For feet and inches or minutes and seconds, use the correct prime and double prime symbols (′ and ″) rather than apostrophes or quotes.",
            DegreeThenSinglePrime => "For minutes, use the correct prime symbol (′) rather than an apostrophe or single quote.",
        }.to_string();

        Some(Lint {
            span,
            lint_kind: LintKind::Formatting,
            suggestions: vec![Suggestion::ReplaceWith(replacement)],
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects the use of typewriter-style apostrophes and quotes for measurements to Unicode prime and double prime symbols."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::FootInchMinuteSecondSymbols;

    #[test]
    fn without_spaces_after() {
        assert_suggestion_result(
            "i'm 5'7\", 140 pounds and the small was a perfect fit for me.",
            FootInchMinuteSecondSymbols::default(),
            "i'm 5′7″, 140 pounds and the small was a perfect fit for me.",
        );
    }

    #[test]
    fn with_spaces_after() {
        assert_suggestion_result(
            "i'm a tall gal, 5' 11\" with broad shoulders and i weigh about 165",
            FootInchMinuteSecondSymbols::default(),
            "i'm a tall gal, 5′ 11″ with broad shoulders and i weigh about 165",
        );
    }

    #[test]
    fn with_spaces_before() {
        assert_suggestion_result(
            "I'm about 6 ' 1 \" tall.",
            FootInchMinuteSecondSymbols::default(),
            "I'm about 6 ′ 1 ″ tall.",
        );
    }

    #[test]
    fn with_open_smart_feet() {
        assert_suggestion_result(
            "LeBron James at approximately 6‘7\"",
            FootInchMinuteSecondSymbols::default(),
            "LeBron James at approximately 6′7″",
        );
    }

    #[test]
    fn with_close_smart_feet() {
        assert_suggestion_result(
            "Michael Jordan: 6’6\"",
            FootInchMinuteSecondSymbols::default(),
            "Michael Jordan: 6′6″",
        );
    }

    #[test]
    fn with_open_smart_inches() {
        assert_suggestion_result(
            "Shaquille O'Neal: 7' 1“",
            FootInchMinuteSecondSymbols::default(),
            "Shaquille O'Neal: 7′ 1″",
        );
    }

    #[test]
    fn with_close_smart_inches() {
        assert_suggestion_result(
            "Magic Johnson: 6' 9”",
            FootInchMinuteSecondSymbols::default(),
            "Magic Johnson: 6′ 9″",
        );
    }

    #[test]
    fn degrees_and_minutes_no_spaces_ascii_apostrophe() {
        assert_suggestion_result(
            "Eiffel Tower: 48°51'N, 2°17'E",
            FootInchMinuteSecondSymbols::default(),
            "Eiffel Tower: 48°51′N, 2°17′E",
        );
    }

    #[test]
    fn degrees_and_minutes_with_spaces_smart_apostrophe() {
        assert_suggestion_result(
            "Statue of Liberty: 40° 41’ N, 74° 02’ W",
            FootInchMinuteSecondSymbols::default(),
            "Statue of Liberty: 40° 41′ N, 74° 02′ W",
        );
    }

    #[test]
    fn fix_prime_feet_ascii_quote() {
        assert_suggestion_result("6′7\"", FootInchMinuteSecondSymbols::default(), "6′7″");
    }

    #[test]
    fn fix_ascii_feet_prime_seconds() {
        assert_suggestion_result("6'7″", FootInchMinuteSecondSymbols::default(), "6′7″");
    }

    #[test]
    fn fix_prime_minute_ascii_quote() {
        assert_suggestion_result("30′15\"", FootInchMinuteSecondSymbols::default(), "30′15″");
    }

    #[test]
    fn fix_ascii_minute_prime_second() {
        assert_suggestion_result("30'15″", FootInchMinuteSecondSymbols::default(), "30′15″");
    }

    #[test]
    fn fix_degrees_apostrophe_quote() {
        assert_suggestion_result(
            "Sydney Opera House: 33°51'25\"S, 151° 12' 55\" East",
            FootInchMinuteSecondSymbols::default(),
            "Sydney Opera House: 33°51′25″S, 151° 12′ 55″ East",
        );
    }

    #[test]
    fn fix_degrees_prime_quote() {
        assert_suggestion_result(
            "The Taj Mahal: 27°10′30\"N & 78° 02’ 32” E",
            FootInchMinuteSecondSymbols::default(),
            "The Taj Mahal: 27°10′30″N & 78° 02′ 32″ E",
        );
    }

    #[test]
    fn fix_degrees_apostrophe_prime() {
        assert_suggestion_result(
            "Colosseum: 41°53'25″North; 12°29'32″East",
            FootInchMinuteSecondSymbols::default(),
            "Colosseum: 41°53′25″North; 12°29′32″East",
        );
    }

    #[test]
    fn dont_flag_degrees_prime_double_prime() {
        assert_no_lints(
            "There are pyramids at 29°58′45″ north and 31°08′03″ east",
            FootInchMinuteSecondSymbols::default(),
        );
    }
}
