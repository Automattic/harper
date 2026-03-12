//! Corrects common mistaken forms of "of course" while ignoring valid phrases like
//! "kind of curse".

use crate::expr::{Expr, OwnedExprExt, SequenceExpr};
use crate::linting::expr_linter::Chunk;
use crate::{
    Token, TokenStringExt,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct OfCourse {
    expr: Box<dyn Expr>,
}

impl Default for OfCourse {
    fn default() -> Self {
        let curse_or_corse = SequenceExpr::default()
            .t_aco("of")
            .then_whitespace()
            .then(WordSet::new(&["curse", "corse"]));

        let off_course_or_coarse = SequenceExpr::default()
            .t_aco("off")
            .then_whitespace()
            .then(WordSet::new(&["course", "coarse"]));

        let expr = curse_or_corse
            .or_longest(off_course_or_coarse)
            .or_longest(
                SequenceExpr::default()
                    .t_aco("o")
                    .then_whitespace()
                    .t_aco("course"),
            )
            .or_longest(WordSet::new(&["ofcourse"]));

        Self {
            expr: Box::new(expr),
        }
    }
}

impl ExprLinter for OfCourse {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched: &[Token], source: &[char]) -> Option<Lint> {
        let phrase_span = matched.span()?;
        let phrase = phrase_span.get_content_string(source);

        if (phrase.eq_ignore_ascii_case("of curse") || phrase.eq_ignore_ascii_case("of corse"))
            && preceding_word(source, matched.first()?.span.start)
                .is_some_and(|prev| matches!(prev.as_str(), "kind" | "sort"))
        {
            return None;
        }

        if phrase.eq_ignore_ascii_case("off course")
            && preceding_non_whitespace_char(source, matched.first()?.span.start)
                .is_some_and(|ch| ch.is_alphanumeric())
        {
            return None;
        }

        Some(Lint {
            span: phrase_span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "of course",
                phrase_span.get_content(source),
            )],
            message: "Did you mean `of course`?".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Corrects common mistaken forms of `of course`, including `of curse`, `off course`, and `ofcourse`, while ignoring valid phrases like `kind of curse`."
    }
}

fn preceding_word(source: &[char], offset: usize) -> Option<String> {
    let prefix = source.get(..offset)?;
    let mut i = prefix.len().checked_sub(1)?;

    while prefix[i].is_whitespace() {
        i = i.checked_sub(1)?;
    }

    let start = prefix[..=i]
        .iter()
        .rposition(|c| c.is_whitespace())
        .map(|pos| pos + 1)
        .unwrap_or(0);

    Some(
        prefix[start..=i]
            .iter()
            .collect::<String>()
            .to_ascii_lowercase(),
    )
}

fn preceding_non_whitespace_char(source: &[char], offset: usize) -> Option<char> {
    let prefix = source.get(..offset)?;
    prefix.iter().rev().find(|c| !c.is_whitespace()).copied()
}

#[cfg(test)]
mod tests {
    use super::OfCourse;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn flags_of_curse() {
        assert_suggestion_result("Yes, of curse!", OfCourse::default(), "Yes, of course!");
    }

    #[test]
    fn flags_of_corse() {
        assert_suggestion_result(
            "Well, of corse we can.",
            OfCourse::default(),
            "Well, of course we can.",
        );
    }

    #[test]
    fn ignores_kind_of_curse() {
        assert_lint_count("This kind of curse is dangerous.", OfCourse::default(), 0);
    }

    #[test]
    fn ignores_sort_of_curse() {
        assert_lint_count("It's a sort of curse that lingers.", OfCourse::default(), 0);
    }

    #[test]
    fn ignores_curse_of_title() {
        assert_lint_count(
            "The Curse of Strahd is a famous module.",
            OfCourse::default(),
            0,
        );
    }

    #[test]
    fn flags_off_course() {
        assert_suggestion_result(
            "Yes, off course we should do that.",
            OfCourse::default(),
            "Yes, of course we should do that.",
        );
    }

    #[test]
    fn flags_o_course() {
        assert_suggestion_result(
            "Yes, o course we should do that.",
            OfCourse::default(),
            "Yes, of course we should do that.",
        );
    }

    #[test]
    fn flags_ofcourse() {
        assert_suggestion_result(
            "Ofcourse, I like other languages.",
            OfCourse::default(),
            "Of course, I like other languages.",
        );
    }

    #[test]
    fn flags_off_coarse() {
        assert_suggestion_result(
            "Off coarse, the web service will still be operational.",
            OfCourse::default(),
            "Of course, the web service will still be operational.",
        );
    }

    #[test]
    fn ignores_literal_off_course() {
        assert_lint_count(
            "Her sailboat had been driven off course by the storm.",
            OfCourse::default(),
            0,
        );
    }
}
