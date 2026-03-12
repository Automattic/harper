use crate::linting::expr_linter::Chunk;
use crate::{
    Lrc, Token,
    expr::{Expr, SequenceExpr},
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::{UPOSSet, WordSet},
};
use harper_brill::UPOS;

use super::NOUN_VERB_PAIRS;

pub struct VerbInsteadOfNoun {
    expr: Box<dyn Expr>,
}

impl Default for VerbInsteadOfNoun {
    fn default() -> Self {
        let verbs = Lrc::new(WordSet::new(
            &NOUN_VERB_PAIRS
                .iter()
                .map(|&(_, verb)| verb)
                .collect::<Vec<_>>(),
        ));
        Self {
            expr: Box::new(
                SequenceExpr::with(UPOSSet::new(&[UPOS::ADJ, UPOS::ADP]))
                    .then_whitespace()
                    .then(verbs.clone()),
            ),
        }
    }
}

impl ExprLinter for VerbInsteadOfNoun {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let adj_tok = &toks.first()?;
        let verb_tok = &toks.last()?;

        let adj_text = adj_tok.span.get_content_string(src);
        let verb_text = verb_tok.span.get_content_string(src);
        let verb_lower = verb_text.to_lowercase();

        if adj_tok.kind.is_auxiliary_verb() || adj_tok.kind.is_upos(UPOS::AUX) {
            return None;
        }

        let noun = NOUN_VERB_PAIRS
            .iter()
            .find(|(_, verb)| *verb == verb_lower)
            .map(|(noun, _)| noun)?;

        // Don't flag "so I better advise you", "you'd better believe this", "you'd best listen to me".
        if adj_text == "better" || adj_text == "best" {
            return None;
        }

        // "Sound" is both adjective and noun. We want to flag the common "sound advise"
        // But not "sound affect", which is just as correct as "sound effect".
        if adj_text == "sound" && verb_text == "affect" {
            return None;
        }

        Some(Lint {
            span: verb_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case(
                noun.chars().collect(),
                verb_tok.span.get_content(src),
            )],
            message: format!("`{verb_text}` is a verb, the noun should be `{noun}`."),
            priority: 63,
        })
    }

    fn description(&self) -> &'static str {
        "Corrects verbs used instead of nouns when the two are related."
    }
}
