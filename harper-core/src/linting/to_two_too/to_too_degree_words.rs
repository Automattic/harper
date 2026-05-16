use crate::{
    Token, TokenKind,
    char_string::CharStringExt,
    expr::{AnchorEnd, Expr, FirstMatchOf, SequenceExpr},
    patterns::{SingleTokenPattern, prepositional_preceder},
};

use super::{ExprLinter, Lint, LintKind, Suggestion};
use crate::linting::expr_linter::Chunk;

/// Verbs that commonly take "to" as a prepositional complement.
/// Guards against false positives like "give to many charities".
const TO_TAKING_VERBS: &[&str] = &[
    "add",
    "added",
    "adding",
    "adds",
    "amount",
    "amounted",
    "amounting",
    "amounts",
    "applied",
    "apply",
    "applies",
    "applying",
    "attend",
    "attended",
    "attending",
    "attends",
    "belong",
    "belonged",
    "belonging",
    "belongs",
    "came",
    "cater",
    "catered",
    "catering",
    "caters",
    "come",
    "comes",
    "coming",
    "contribute",
    "contributed",
    "contributes",
    "contributing",
    "donate",
    "donated",
    "donates",
    "donating",
    "expose",
    "exposed",
    "exposes",
    "exposing",
    "gave",
    "give",
    "given",
    "gives",
    "giving",
    "go",
    "goes",
    "going",
    "gone",
    "lead",
    "leading",
    "leads",
    "led",
    "listen",
    "listened",
    "listening",
    "listens",
    "move",
    "moved",
    "moves",
    "moving",
    "refer",
    "referred",
    "referring",
    "refers",
    "relate",
    "related",
    "relates",
    "relating",
    "respond",
    "responded",
    "responding",
    "responds",
    "return",
    "returned",
    "returning",
    "returns",
    "send",
    "sending",
    "sends",
    "sent",
    "speak",
    "speaking",
    "speaks",
    "spoke",
    "spoken",
    "talk",
    "talked",
    "talking",
    "talks",
    "went",
    "write",
    "writes",
    "writing",
    "written",
    "wrote",
];

pub struct ToTooDegreeWords {
    expr: Box<dyn Expr>,
}

impl Default for ToTooDegreeWords {
    fn default() -> Self {
        // Pattern 1: degree word at clause end.
        let at_end = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_word_set(&["many", "much", "few", "little"])
            .then_any_of(vec![
                Box::new(SequenceExpr::default().then_kind_is_but_is_not_except(
                    TokenKind::is_punctuation,
                    |_| false,
                    &[
                        "`", "\"", "'", "\u{201c}", "\u{201d}", "\u{2018}", "\u{2019}", "-",
                        "\u{2013}", "\u{2014}",
                    ],
                )),
                Box::new(AnchorEnd),
            ]);

        // Pattern 2: degree word followed by a noun (e.g. "to many cookies").
        let before_noun = SequenceExpr::default()
            .t_aco("to")
            .t_ws()
            .then_word_set(&["many", "much", "few", "little"])
            .t_ws()
            .then_noun();

        Self {
            expr: Box::new(FirstMatchOf::new(vec![
                Box::new(at_end),
                Box::new(before_noun),
            ])),
        }
    }
}

impl ExprLinter for ToTooDegreeWords {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint_with_context(
        &self,
        tokens: &[Token],
        source: &[char],
        context: Option<(&[Token], &[Token])>,
    ) -> Option<Lint> {
        let to_tok = tokens
            .iter()
            .find(|t| t.get_ch(source).eq_ch(&['t', 'o']))?;

        // Suppress when "to" is a preposition, not a typo for "too".
        if let Some((before, _)) = context
            && let Some(prev) = before.iter().rfind(|t| !t.kind.is_whitespace())
            && (prepositional_preceder().matches_token(prev, source)
                || prev
                    .get_ch(source)
                    .eq_any_ignore_ascii_case_str(TO_TAKING_VERBS))
        {
            return None;
        }

        Some(Lint {
            span: to_tok.span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::replace_with_match_case_str(
                "too",
                to_tok.get_ch(source),
            )],
            message: "Use `too` here to mean 'also' or an excessive degree.".to_string(),
            ..Default::default()
        })
    }

    fn description(&self) -> &str {
        "Detects `to` used before degree words like `many`, `much`, `few`, or `little`."
    }
}
