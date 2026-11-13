use crate::{
    Lint, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, LintKind, Suggestion, expr_linter::Chunk},
};

const INVARIANT_NOUNS: &[&str] = &[
    "athletics",
    "biceps",
    "billiards",
    "chassis",
    "corps",
    "crossroads",
    "debris",
    "diabetes",
    "economics",
    "ethics",
    "gymnastics",
    "headquarters",
    "linguistics",
    "mathematics",
    "means",
    "measles",
    "mumps",
    "news",
    "physics",
    "politics",
    "rabies",
    "series",
    "species",
    "statistics",
];

pub struct PluralSubjectSeems {
    expr: SequenceExpr,
}

impl Default for PluralSubjectSeems {
    fn default() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_kind_where(|kind| kind.is_plural_noun() && !kind.is_singular_noun())
                .t_ws()
                .t_aco("seems"),
        }
    }
}

impl ExprLinter for PluralSubjectSeems {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let verb = toks.last()?;
        let subject = toks.first()?.get_str(src);

        if INVARIANT_NOUNS
            .iter()
            .any(|noun| subject.eq_ignore_ascii_case(noun))
        {
            return None;
        }

        Some(Lint {
            span: verb.span,
            lint_kind: LintKind::Agreement,
            suggestions: vec![Suggestion::replace_with_match_case(
                "seem".chars().collect(),
                verb.get_ch(src),
            )],
            message: format!(
                "The plural subject `{subject}` requires `seem`; `seems` is singular."
            ),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Ensures plural subjects agree with the verb `seem`."
    }
}

#[cfg(test)]
mod tests {
    use super::PluralSubjectSeems;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_reported_example() {
        assert_suggestion_result(
            "Web pages seems to work pretty well so far.",
            PluralSubjectSeems::default(),
            "Web pages seem to work pretty well so far.",
        );
    }

    #[test]
    fn corrects_simple_plural_subject() {
        assert_suggestion_result(
            "The pages seems ready.",
            PluralSubjectSeems::default(),
            "The pages seem ready.",
        );
    }

    #[test]
    fn corrects_irregular_plural_subject() {
        assert_suggestion_result(
            "The children seems ready.",
            PluralSubjectSeems::default(),
            "The children seem ready.",
        );
    }

    #[test]
    fn allows_singular_subject() {
        assert_lint_count("The page seems ready.", PluralSubjectSeems::default(), 0);
    }

    #[test]
    fn allows_plural_subject_with_plural_verb() {
        assert_lint_count("The pages seem ready.", PluralSubjectSeems::default(), 0);
    }

    #[test]
    fn allows_singular_pronoun_subject() {
        assert_lint_count("It seems ready.", PluralSubjectSeems::default(), 0);
    }

    #[test]
    fn allows_invariant_nouns() {
        assert_lint_count("The news seems good.", PluralSubjectSeems::default(), 0);
        assert_lint_count("The series seems good.", PluralSubjectSeems::default(), 0);
        assert_lint_count(
            "The species seems stable.",
            PluralSubjectSeems::default(),
            0,
        );
    }
}
