use crate::{
    Dialect, Lint, Token, TokenStringExt,
    expr::{Expr, FirstMatchOf},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
    patterns::Word,
};

pub struct ExpandFavourite {
    expr: FirstMatchOf,
    dialect: Dialect,
}

impl ExpandFavourite {
    pub fn new(dialect: Dialect) -> Self {
        Self {
            expr: FirstMatchOf::new(vec![
                Box::new(Word::new_standard_case("fav")),
                Box::new(Word::new_standard_case("favs")),
                Box::new(Word::new_standard_case("fave")),
                Box::new(Word::new_standard_case("faves")),
            ]),
            dialect,
        }
    }
}

impl ExprLinter for ExpandFavourite {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;

        let abbr = span.get_content_string(src).to_ascii_lowercase();

        let expanded = match (self.dialect, abbr.ends_with('s')) {
            (Dialect::American, false) => "favorite",
            (Dialect::American, true) => "favorites",
            (_, false) => "favourite",
            (_, true) => "favourites",
        };

        let suggestions = vec![Suggestion::replace_with_match_case_str(
            expanded,
            span.get_content(src),
        )];
        let message = format!("Use `{expanded}` instead of `{abbr}`.");

        Some(Lint {
            span,
            lint_kind: LintKind::Style,
            suggestions,
            message,
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Expands the abbreviations `fav` and `fave` to the full word `favorite` or `favourite` for clarity."
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Dialect,
        linting::tests::{assert_no_lints, assert_suggestion_result},
    };

    use super::ExpandFavourite;

    // Basic case tests

    #[test]
    fn fix_lowercase() {
        assert_suggestion_result(
            "It's my fav",
            ExpandFavourite::new(Dialect::American),
            "It's my favorite",
        );
    }

    #[test]
    fn fix_all_caps() {
        assert_suggestion_result(
            "IT'S MY FAV",
            ExpandFavourite::new(Dialect::American),
            "IT'S MY FAVORITE",
        );
    }

    #[test]
    fn fix_title_case() {
        assert_suggestion_result(
            "It's My Fav",
            ExpandFavourite::new(Dialect::American),
            "It's My Favorite",
        );
    }

    #[test]
    fn dont_flag_pascal_case() {
        assert_no_lints(
            "It's a project called FaVe",
            ExpandFavourite::new(Dialect::American),
        );
    }

    // Real-world tests

    #[test]
    fn fix_favs_lower_us() {
        assert_suggestion_result(
            "Each single favorite group - no matter if it contains 1 or 1000 favs - in here is displayed as 11MB",
            ExpandFavourite::new(Dialect::American),
            "Each single favorite group - no matter if it contains 1 or 1000 favorites - in here is displayed as 11MB",
        );
    }

    #[test]
    fn fix_favs_title_au() {
        assert_suggestion_result(
            "Lost all my Favs again on reinstalling mac - got angry - then used claude.ai to help make a tool",
            ExpandFavourite::new(Dialect::Australian),
            "Lost all my Favourites again on reinstalling mac - got angry - then used claude.ai to help make a tool",
        );
    }

    #[test]
    fn dont_flag_fave_mixed_case_uk() {
        assert_no_lints(
            "FaVe employs end-to-end encryption and leverages the inherent security features of FairOS and Swarm to ensure data integrity and confidentiality.",
            ExpandFavourite::new(Dialect::British),
        );
    }

    #[test]
    fn fix_fave_and_faves_ca() {
        assert_suggestion_result(
            "Once faves are created, a user's fave posts can be queried",
            ExpandFavourite::new(Dialect::Canadian),
            "Once favourites are created, a user's favourite posts can be queried",
        );
    }

    #[test]
    fn fix_fav_title_in() {
        assert_suggestion_result(
            "Save your Data ,by not Streaming your Fav Songs Online again & again(Just Download Them!)",
            ExpandFavourite::new(Dialect::Indian),
            "Save your Data ,by not Streaming your Favourite Songs Online again & again(Just Download Them!)",
        );
    }
}
