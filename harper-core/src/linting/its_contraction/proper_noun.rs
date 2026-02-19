use std::ops::Range;
use std::sync::Arc;

use harper_brill::UPOS;

use crate::{
    Document, Token, TokenStringExt,
    expr::{Expr, ExprExt, ExprMap, OwnedExprExt, SequenceExpr},
    linting::{Lint, LintKind, Linter, Suggestion},
    patterns::{DerivedFrom, UPOSSet},
};

pub struct ProperNoun {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<Range<usize>>>,
}

impl Default for ProperNoun {
    fn default() -> Self {
        let mut map = ExprMap::default();

        let opinion_verbs = DerivedFrom::new_from_str("think")
            .or(DerivedFrom::new_from_str("hope"))
            .or(DerivedFrom::new_from_str("assume"))
            .or(DerivedFrom::new_from_str("doubt"))
            .or(DerivedFrom::new_from_str("guess"));

        let capitalized_word = |tok: &Token, src: &[char]| {
            tok.kind.is_word()
                && tok
                    .span
                    .get_content(src)
                    .first()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
        };

        let name_head = UPOSSet::new(&[UPOS::PROPN]).or(capitalized_word);

        let lookahead_word = SequenceExpr::default().t_ws().then_any_word();

        map.insert(
            SequenceExpr::with(opinion_verbs)
                .t_ws()
                .t_aco("its")
                .t_ws()
                .then(name_head)
                .then_optional(lookahead_word),
            2..3,
        );

        let map = Arc::new(map);

        Self {
            expr: Box::new(map.clone()),
            map,
        }
    }
}

impl Linter for ProperNoun {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();
        let source = document.get_source();

        for chunk in document.iter_chunks() {
            lints.extend(
                self.expr
                    .iter_matches(chunk, source)
                    .filter_map(|match_span| {
                        let matched = &chunk[match_span.start..match_span.end];
                        self.match_to_lint(matched, source)
                    }),
            );
        }

        lints
    }

    fn description(&self) -> &str {
        "Suggests the contraction `it's` after opinion verbs when it introduces a proper noun."
    }
}

impl ProperNoun {
    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        if matched_tokens.len() >= 7
            && let Some(next_word) = matched_tokens.get(6)
        {
            let is_lowercase = next_word
                .span
                .get_content(source)
                .first()
                .map(|c| c.is_lowercase())
                .unwrap_or(false);

            if is_lowercase
                && (next_word.kind.is_upos(UPOS::NOUN) || next_word.kind.is_upos(UPOS::ADJ))
            {
                return None;
            }
        }

        let range = self.map.lookup(0, matched_tokens, source)?.clone();
        let offending = matched_tokens.get(range.start)?;
        let offender_text = offending.span.get_content(source);

        Some(Lint {
            span: offending.span,
            lint_kind: LintKind::Punctuation,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "it's",
                offender_text,
            )],
            message: "Use `it's` (short for \"it is\") before a proper noun in this construction."
                .to_owned(),
            priority: 31,
        })
    }
}
