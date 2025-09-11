use crate::{
    Document,
    expr::{ExprExt, SequenceExpr},
    linting::Linter,
};

use super::{Lint, LintKind, Suggestion};

pub struct ToTooEos {
    expr: SequenceExpr,
}

impl ToTooEos {
    pub fn new() -> Self {
        Self {
            expr: SequenceExpr::default()
                .then_comma()
                .t_ws()
                .t_aco("to")
                .then_sentence_terminator(),
        }
    }
}

impl Default for ToTooEos {
    fn default() -> Self {
        Self::new()
    }
}

impl Linter for ToTooEos {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let matches = self.expr.iter_matches_in_doc(document);

        matches
            .map(|m| {
                let tok = &m.get_content(document.get_tokens())[2];

                Lint {
                    span: tok.span,
                    lint_kind: LintKind::Typo,
                    suggestions: vec![Suggestion::replace_with_match_case_str(
                        "too",
                        tok.span.get_content(document.get_source()),
                    )],
                    message: "Use `too` when expressing similarity.".to_owned(),
                    priority: 63,
                }
            })
            .collect()
    }

    fn description(&self) -> &str {
        "Identifies incorrect usage of the term `to` at the end of a sentence."
    }
}
