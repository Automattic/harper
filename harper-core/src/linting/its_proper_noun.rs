use std::ops::Range;
use std::sync::Arc;

use harper_brill::UPOS;

use crate::{
    Token,
    expr::{Expr, ExprMap, OwnedExprExt, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{DerivedFrom, UPOSSet},
};

pub struct ItsProperNoun {
    expr: Box<dyn Expr>,
    map: Arc<ExprMap<Range<usize>>>,
}

impl Default for ItsProperNoun {
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
            SequenceExpr::default()
                .then(opinion_verbs)
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

impl ExprLinter for ItsProperNoun {
    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, matched_tokens: &[Token], source: &[char]) -> Option<Lint> {
        if matched_tokens.len() == 7 {
            let next_word = matched_tokens.get(6)?;
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
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "it's",
                offender_text,
            )],
            message: "Use `it's` (short for \"it is\") before a proper noun in this construction."
                .to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Suggests the contraction `it's` after opinion verbs when it introduces a proper noun."
    }
}

#[cfg(test)]
mod tests {
    use super::ItsProperNoun;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn corrects_think_google() {
        assert_suggestion_result(
            "I think its Google, not Microsoft.",
            ItsProperNoun::default(),
            "I think it's Google, not Microsoft.",
        );
    }

    #[test]
    fn corrects_hope_katie() {
        assert_suggestion_result(
            "I hope its Katie.",
            ItsProperNoun::default(),
            "I hope it's Katie.",
        );
    }

    #[test]
    fn corrects_guess_date() {
        assert_suggestion_result(
            "I guess its March 6.",
            ItsProperNoun::default(),
            "I guess it's March 6.",
        );
    }

    #[test]
    fn corrects_assume_john() {
        assert_suggestion_result(
            "We assume its John.",
            ItsProperNoun::default(),
            "We assume it's John.",
        );
    }

    #[test]
    fn corrects_doubt_tesla() {
        assert_suggestion_result(
            "They doubt its Tesla this year.",
            ItsProperNoun::default(),
            "They doubt it's Tesla this year.",
        );
    }

    #[test]
    fn handles_two_word_name() {
        assert_suggestion_result(
            "She thinks its New York.",
            ItsProperNoun::default(),
            "She thinks it's New York.",
        );
    }

    #[test]
    fn ignores_existing_contraction() {
        assert_lint_count("I think it's Google.", ItsProperNoun::default(), 0);
    }

    #[test]
    fn ignores_possessive_noun_after_name() {
        assert_lint_count(
            "I think its Google product launch.",
            ItsProperNoun::default(),
            0,
        );
    }

    #[test]
    fn ignores_without_opinion_verb() {
        assert_lint_count(
            "Its Google Pixel lineup is impressive.",
            ItsProperNoun::default(),
            0,
        );
    }

    #[test]
    fn ignores_common_noun_target() {
        assert_lint_count(
            "We hope its accuracy improves.",
            ItsProperNoun::default(),
            0,
        );
    }
}
