use std::sync::Arc;

use crate::{
    Token, TokenKind, TokenStringExt,
    expr::{Expr, ExprMap, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
};

pub struct DoubleClick {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<usize>>,
}

impl DoubleClick {
    fn double_click_sequence() -> SequenceExpr {
        SequenceExpr::default()
            .t_aco("double")
            .t_ws()
            .then_word_set(&["click", "clicked", "clicking", "clicks"])
    }
}

impl Default for DoubleClick {
    fn default() -> Self {
        let mut map = ExprMap::default();

        map.insert(
            SequenceExpr::default()
                .then_seq(Self::double_click_sequence())
                .t_ws()
                .then_any_word(),
            0,
        );

        map.insert(
            SequenceExpr::default()
                .then_seq(Self::double_click_sequence())
                .then_punctuation(),
            0,
        );

        map.insert(
            SequenceExpr::default()
                .then_seq(Self::double_click_sequence())
                .t_ws()
                .then_kind_is_but_is_not(TokenKind::is_word, TokenKind::is_verb),
            0,
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl ExprLinter for DoubleClick {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        let double_idx = *self.map.lookup(0, matched_tokens, source)?;
        let click_idx = 2;
        let span = matched_tokens.get(double_idx..=click_idx)?.span()?;
        let template = span.get_content(source);

        let double_word = matched_tokens.get(double_idx)?.span.get_content(source);
        let click_word = matched_tokens.get(click_idx)?.span.get_content(source);

        let replacement: Vec<char> = double_word
            .iter()
            .copied()
            .chain(['-'])
            .chain(click_word.iter().copied())
            .collect();

        Some(Lint {
            span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![Suggestion::replace_with_match_case(replacement, template)],
            message: "Add a hyphen to this command.".to_owned(),
            priority: 40,
        })
    }

    fn description(&self) -> &'static str {
        "Encourages hyphenating `double-click` and its inflections."
    }
}

#[cfg(test)]
mod tests {
    use super::DoubleClick;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_basic_command() {
        assert_suggestion_result(
            "Double click the icon.",
            DoubleClick::default(),
            "Double-click the icon.",
        );
    }

    #[test]
    fn corrects_with_preposition() {
        assert_suggestion_result(
            "Please double click on the link.",
            DoubleClick::default(),
            "Please double-click on the link.",
        );
    }

    #[test]
    fn corrects_with_pronoun() {
        assert_suggestion_result(
            "You should double click it to open.",
            DoubleClick::default(),
            "You should double-click it to open.",
        );
    }

    #[test]
    fn corrects_plural_form() {
        assert_suggestion_result(
            "Double clicks are recorded in the log.",
            DoubleClick::default(),
            "Double-clicks are recorded in the log.",
        );
    }

    #[test]
    fn corrects_past_tense() {
        assert_suggestion_result(
            "They double clicked the submit button.",
            DoubleClick::default(),
            "They double-clicked the submit button.",
        );
    }

    #[test]
    fn corrects_gerund() {
        assert_suggestion_result(
            "Double clicking the item highlights it.",
            DoubleClick::default(),
            "Double-clicking the item highlights it.",
        );
    }

    #[test]
    fn corrects_with_caps() {
        assert_suggestion_result(
            "He DOUBLE CLICKED the file.",
            DoubleClick::default(),
            "He DOUBLE-CLICKED the file.",
        );
    }

    #[test]
    fn corrects_multiline() {
        assert_suggestion_result(
            "Double\nclick the checkbox.",
            DoubleClick::default(),
            "Double-click the checkbox.",
        );
    }

    #[test]
    fn corrects_at_sentence_end() {
        assert_suggestion_result(
            "Just double click.",
            DoubleClick::default(),
            "Just double-click.",
        );
    }

    #[test]
    fn allows_hyphenated_form() {
        assert_lint_count("Double-click the icon.", DoubleClick::default(), 0);
    }

    #[test]
    fn ignores_other_double_words() {
        assert_lint_count(
            "She said the double rainbow was beautiful.",
            DoubleClick::default(),
            0,
        );
    }
}
