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
    "give",
    "gives",
    "gave",
    "given",
    "giving",
    "talk",
    "talks",
    "talked",
    "talking",
    "speak",
    "speaks",
    "spoke",
    "spoken",
    "speaking",
    "listen",
    "listens",
    "listened",
    "listening",
    "send",
    "sends",
    "sent",
    "sending",
    "write",
    "writes",
    "wrote",
    "written",
    "writing",
    "go",
    "goes",
    "went",
    "gone",
    "going",
    "come",
    "comes",
    "came",
    "coming",
    "belong",
    "belongs",
    "belonged",
    "belonging",
    "lead",
    "leads",
    "led",
    "leading",
    "refer",
    "refers",
    "referred",
    "referring",
    "apply",
    "applies",
    "applied",
    "applying",
    "contribute",
    "contributes",
    "contributed",
    "contributing",
    "respond",
    "responds",
    "responded",
    "responding",
    "donate",
    "donates",
    "donated",
    "donating",
    "attend",
    "attends",
    "attended",
    "attending",
    "return",
    "returns",
    "returned",
    "returning",
    "add",
    "adds",
    "added",
    "adding",
    "move",
    "moves",
    "moved",
    "moving",
    "relate",
    "relates",
    "related",
    "relating",
    "expose",
    "exposes",
    "exposed",
    "exposing",
    "cater",
    "caters",
    "catered",
    "catering",
    "amount",
    "amounts",
    "amounted",
    "amounting",
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
        if let Some((before, _)) = context {
            if let Some(prev) = before.iter().rfind(|t| !t.kind.is_whitespace()) {
                if prepositional_preceder().matches_token(prev, source)
                    || prev
                        .get_ch(source)
                        .eq_any_ignore_ascii_case_str(TO_TAKING_VERBS)
                {
                    return None;
                }
            }
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
