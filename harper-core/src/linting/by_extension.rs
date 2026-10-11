use crate::{
    Lint, Token,
    expr::{AnchorStart, Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

pub struct ByExtension {
    expr: SequenceExpr,
}

impl Default for ByExtension {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::any_of([
                Box::new(SequenceExpr::word_set(["and", "so", "which", "thus"]).t_ws())
                    as Box<dyn Expr>,
                Box::new(AnchorStart),
            ])
            .then_word_seq(&["by", "extent"]),
        }
    }
}

impl ExprLinter for ByExtension {
    type Unit = Chunk;

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.last()?.span;

        Some(Lint {
            span,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "extension",
                span.get_content(src),
            )],
            message: "The idiom is `by extension` not `by extent`.".to_owned(),
            ..Default::default()
        })
    }

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn description(&self) -> &str {
        "Corrects `by extent` to `by extension`."
    }
}

#[cfg(test)]
mod tests {
    use crate::linting::tests::{assert_no_lints, assert_suggestion_result};

    use super::ByExtension;

    #[test]
    fn and() {
        assert_suggestion_result(
            "Well it's mostly arcane magic, as TTYs (and by extent PTYs) are an ancient technology used in teletypes",
            ByExtension::default(),
            "Well it's mostly arcane magic, as TTYs (and by extension PTYs) are an ancient technology used in teletypes",
        );
    }

    #[test]
    fn and_comma() {
        assert_suggestion_result(
            "Since pushState does nothing to inform the browser--and, by extent, the screen reader--that new content is present",
            ByExtension::default(),
            "Since pushState does nothing to inform the browser--and, by extension, the screen reader--that new content is present",
        );
    }

    #[test]
    fn so() {
        assert_suggestion_result(
            "Discord activities are only available on the Discord desktop client, so by extent the Stremio presence only works on desktop.",
            ByExtension::default(),
            "Discord activities are only available on the Discord desktop client, so by extension the Stremio presence only works on desktop.",
        );
    }

    #[test]
    fn thus() {
        assert_suggestion_result(
            "This means it can emulate every other Turing complete language and thus by extent Java",
            ByExtension::default(),
            "This means it can emulate every other Turing complete language and thus by extension Java",
        );
    }

    #[test]
    fn which_comma() {
        assert_suggestion_result(
            "which, by extent, includes 100% match with your OS file paths handling.",
            ByExtension::default(),
            "which, by extension, includes 100% match with your OS file paths handling.",
        );
    }

    #[test]
    fn dont_flag_raster_context() {
        assert_no_lints(
            "Mask raster by extent in Python using rasterio",
            ByExtension::default(),
        );
    }

    #[test]
    fn dont_flag_fact_unit_context() {
        assert_no_lints(
            "Group facts by extent. The unit is the extent, not the fact.",
            ByExtension::default(),
        );
    }

    #[test]
    fn dont_flag_geometry_tile_context() {
        assert_no_lints(
            "Geometries that extend past the tile's area as defined by extent are often used as a buffer for rendering features that overlap multiple adjacent tiles.",
            ByExtension::default(),
        );
    }
}
