use harper_brill::UPOS;

use crate::{
    CharStringExt, Document, Token, TokenStringExt,
    expr::{All, Expr, ExprExt, OwnedExprExt, SequenceExpr},
    linting::{Lint, LintKind, Linter, Suggestion},
    patterns::{NominalPhrase, Pattern, UPOSSet, WordSet},
};

pub struct General {
    expr: Box<dyn Expr>,
}

impl Default for General {
    fn default() -> Self {
        let positive = SequenceExpr::default().t_aco("its").then_whitespace().then(
            UPOSSet::new(&[UPOS::VERB, UPOS::AUX, UPOS::DET, UPOS::PRON])
                .or(WordSet::new(&["because"])),
        );

        let exceptions = SequenceExpr::anything()
            .then_anything()
            .then(WordSet::new(&["own", "intended"]));

        let inverted = SequenceExpr::default().then_unless(exceptions);

        let expr = All::new(vec![Box::new(positive), Box::new(inverted)]).or_longest(
            SequenceExpr::aco("its")
                .t_ws()
                .then(UPOSSet::new(&[UPOS::ADJ]))
                .t_ws()
                .then(UPOSSet::new(&[UPOS::SCONJ, UPOS::PART])),
        );

        Self {
            expr: Box::new(expr),
        }
    }
}

impl Linter for General {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for chunk in document.iter_chunks() {
            lints.extend(
                self.expr
                    .iter_matches(chunk, source)
                    .filter_map(|match_span| self.match_to_lint(chunk, match_span.start, source)),
            );
        }

        lints
    }

    fn description(&self) -> &str {
        "Detects the possessive `its` before `had`, `been`, or `got` and offers `it's` or `it has`."
    }
}

impl General {
    fn match_to_lint(&self, chunk: &[Token], match_start: usize, source: &[char]) -> Option<Lint> {
        let toks = &chunk[match_start..];
        let offender = toks.first()?;
        let offender_chars = offender.span.get_content(source);

        if toks.get(2)?.kind.is_upos(UPOS::VERB)
            && NominalPhrase.matches(&toks[2..], source).is_some()
        {
            return None;
        }

        // "best" can function as a noun (as in "does its best"), so allow it
        // when "its" is not at the start of the chunk (i.e., there's context before it)
        // This distinguishes "does its best to help" (possessive, valid) from
        // "Its best to leave" (should be "It's best to leave")
        if let Some(adj_tok) = toks.get(2) {
            if adj_tok
                .span
                .get_content(source)
                .eq_ignore_ascii_case_str("best")
                && match_start > 0
            {
                return None;
            }
        }

        Some(Lint {
            span: offender.span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![
                Suggestion::replace_with_match_case_str("it's", offender_chars),
                Suggestion::replace_with_match_case_str("it has", offender_chars),
            ],
            message: "Use `it's` (short for `it has` or `it is`) here, not the possessive `its`."
                .to_owned(),
            priority: 54,
        })
    }
}
