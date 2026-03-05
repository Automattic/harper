use harper_brill::UPOS;

use crate::{
    Document, Token, TokenStringExt,
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
            .then_word_set(&["own", "intended"]);

        let inverted = SequenceExpr::unless(exceptions);

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
                    .filter_map(|match_span| {
                        self.match_to_lint(&chunk[match_span.start..], source)
                    }),
            );
        }

        lints
    }

    fn description(&self) -> &str {
        "Detects the possessive `its` before `had`, `been`, or `got` and offers `it's` or `it has`."
    }
}

impl General {
    fn match_to_lint(&self, toks: &[Token], source: &[char]) -> Option<Lint> {
        let offender = toks.first()?;
        let offender_chars = offender.span.get_content(source);

        let modifier = toks.get(2)?;

        if modifier.kind.is_upos(UPOS::VERB)
            && NominalPhrase.matches(&toks[2..], source).is_some()
            && !Self::is_likely_predicative_participle(modifier, source)
        {
            return None;
        }

        // Past-participle modifiers can be tagged as verbs even in possessive noun phrases:
        // "its abetted parameter", "its associated parameter", etc.
        if self.is_possessive_participle_noun_phrase(toks, source) {
            return None;
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

    fn is_possessive_participle_noun_phrase(&self, toks: &[Token], source: &[char]) -> bool {
        let Some(modifier) = toks.get(2) else {
            return false;
        };
        let Some(gap) = toks.get(3) else {
            return false;
        };
        let Some(head) = toks.get(4) else {
            return false;
        };

        if !modifier.kind.is_verb_past_participle_form() || !gap.kind.is_whitespace() {
            return false;
        }

        if !(head.kind.is_noun() || head.kind.is_proper_noun()) {
            return false;
        }

        if Self::is_likely_predicative_participle(modifier, source) {
            return false;
        }

        let modifier_text = modifier.span.get_content_string(source);

        !["had", "been", "got"]
            .iter()
            .any(|word| modifier_text.eq_ignore_ascii_case(word))
    }

    fn is_likely_predicative_participle(tok: &Token, source: &[char]) -> bool {
        let text = tok.span.get_content_string(source);

        ["called", "named", "known", "termed", "titled"]
            .iter()
            .any(|word| text.eq_ignore_ascii_case(word))
    }
}
