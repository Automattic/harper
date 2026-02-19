use std::sync::Arc;

use crate::{
    Token, TokenStringExt,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::expr_linter::Chunk,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::DerivedFrom,
};

pub struct RightClick {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl Default for RightClick {
    fn default() -> Self {
        let mut map = ExprMap::default();

        map.insert(
            SequenceExpr::word_set(&["right", "left", "middle"])
                .t_ws()
                .then(DerivedFrom::new_from_str("click")),
            0,
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for RightClick {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let start_idx = *self.map.lookup(0, matched_tokens, source)?;
        let click_idx = matched_tokens.len().checked_sub(1)?;
        let span = matched_tokens.get(start_idx..=click_idx)?.span()?;
        let template = span.get_content(source);

        let direction = matched_tokens.get(start_idx)?.span.get_content(source);
        let click = matched_tokens.get(click_idx)?.span.get_content(source);

        let replacement: Vec<char> = direction
            .iter()
            .copied()
            .chain(['-'])
            .chain(click.iter().copied())
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![Suggestion::replace_with_match_case(replacement, template)],
            message: "Hyphenate this mouse command.".to_owned(),
            priority: 40,
        })
    }

    fn description(&self) -> &'static str {
        "Hyphenates right-click style mouse commands."
    }
}

#[cfg(test)]
mod tests {
    use super::RightClick;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn hyphenates_basic_command() {
        assert_suggestion_result(
            "Right click the icon.",
            RightClick::default(),
            "Right-click the icon.",
        );
    }

    #[test]
    fn hyphenates_with_preposition() {
        assert_suggestion_result(
            "Please right click on the link.",
            RightClick::default(),
            "Please right-click on the link.",
        );
    }

    #[test]
    fn hyphenates_past_tense() {
        assert_suggestion_result(
            "They right clicked the submit button.",
            RightClick::default(),
            "They right-clicked the submit button.",
        );
    }

    #[test]
    fn hyphenates_gerund() {
        assert_suggestion_result(
            "Right clicking the item highlights it.",
            RightClick::default(),
            "Right-clicking the item highlights it.",
        );
    }

    #[test]
    fn hyphenates_plural_noun() {
        assert_suggestion_result(
            "Right clicks are tracked in the log.",
            RightClick::default(),
            "Right-clicks are tracked in the log.",
        );
    }

    #[test]
    fn hyphenates_all_caps() {
        assert_suggestion_result(
            "He RIGHT CLICKED the file.",
            RightClick::default(),
            "He RIGHT-CLICKED the file.",
        );
    }

    #[test]
    fn hyphenates_left_click() {
        assert_suggestion_result(
            "Left click the checkbox.",
            RightClick::default(),
            "Left-click the checkbox.",
        );
    }

    #[test]
    fn hyphenates_middle_click() {
        assert_suggestion_result(
            "Middle click to open in a new tab.",
            RightClick::default(),
            "Middle-click to open in a new tab.",
        );
    }

    #[test]
    fn allows_hyphenated_form() {
        assert_lint_count("Right-click the icon.", RightClick::default(), 0);
    }

    #[test]
    fn ignores_unrelated_right_and_click() {
        assert_lint_count(
            "Click the right button to continue.",
            RightClick::default(),
            0,
        );
    }
}
