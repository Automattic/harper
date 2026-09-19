use crate::{
    CharStringExt, Token, TokenStringExt,
    expr::{Expr, FixedPhrase, LongestMatchOf, SequenceExpr},
    linting::{
        ExprLinter, Lint, LintKind, Suggestion,
        expr_linter::{Chunk, find_the_only_token_matching},
    },
};

const VERSE_VERB_CORRECTIONS: &[(&str, &str)] = &[
    ("verse against", "play against"),
    ("versed against", "played against"),
    ("versing against", "playing against"),
    ("verses against", "plays against"),
    ("verse me", "play me"),
    ("verse him", "play him"),
    ("verse her", "play her"),
    ("verse them", "play them"),
    ("verse you", "play you"),
];

pub(super) struct VerseAsVerb {
    expr: LongestMatchOf,
}

impl Default for VerseAsVerb {
    fn default() -> Self {
        let comparison = SequenceExpr::any_word()
            .t_ws()
            .t_aco("verse")
            .t_ws()
            .then_any_word()
            .t_ws()
            .then_any_word()
            .t_ws()
            .then_kind_where(|kind| kind.is_proper_noun());
        let mut expressions: Vec<Box<dyn Expr>> = VERSE_VERB_CORRECTIONS
            .iter()
            .map(|(wrong, _)| Box::new(FixedPhrase::from_phrase(wrong)) as Box<dyn Expr>)
            .collect();
        expressions.push(Box::new(comparison));

        Self {
            expr: LongestMatchOf::new(expressions),
        }
    }
}

impl ExprLinter for VerseAsVerb {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        let span = toks.span()?;
        let matched = span.get_content(src);

        if let Some((_, correction)) = VERSE_VERB_CORRECTIONS
            .iter()
            .find(|(wrong, _)| matched.eq_str(wrong))
        {
            return Some(Lint {
                span,
                lint_kind: LintKind::Nonstandard,
                suggestions: vec![Suggestion::replace_with_match_case_str(correction, matched)],
                message: "`Verse` is not a verb meaning to compete. Use `play against` or `compete against` instead.".to_owned(),
                priority: 31,
            });
        }

        if !toks.first()?.get_ch(src).first()?.is_uppercase() {
            return None;
        }

        let verse =
            find_the_only_token_matching(toks, src, |tok, src| tok.get_ch(src).eq_str("verse"))?;
        Some(Lint {
            span: verse.span,
            lint_kind: LintKind::Nonstandard,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "versus",
                verse.get_ch(src),
            )],
            message: "Use `versus` to compare the two competitors.".to_owned(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Corrects nonstandard uses of `verse` derived from `versus`."
    }
}
