use crate::expr::Expr;
use crate::expr::SequenceExpr;
use crate::linting::expr_linter::Chunk;
use crate::{
    Span, Token,
    linting::{ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

pub struct AskNoPreposition {
    expr: Box<dyn Expr>,
}

impl Default for AskNoPreposition {
    fn default() -> Self {
        let verbs = WordSet::new(&[
            "ask", "asks", "asked", "asking", "tell", "tells", "told", "telling",
        ]);

        let objs = WordSet::new(&["me", "you", "him", "her", "it", "us", "them", "one"]);

        let pattern = SequenceExpr::with(verbs)
            .then_whitespace()
            .t_aco("to")
            .then_whitespace()
            .then(objs);

        Self {
            expr: Box::new(pattern),
        }
    }
}

impl ExprLinter for AskNoPreposition {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        self.expr.as_ref()
    }

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        if toks.len() < 5 {
            return None;
        }

        let verb = toks[0].span.get_content_string(src).to_lowercase();
        let span = Span::new(toks[2].span.start, toks[3].span.end);

        Some(Lint {
            span,
            lint_kind: LintKind::WordChoice,
            suggestions: vec![Suggestion::ReplaceWith(Vec::new())],
            message: format!(
                "The verb `to {verb} someone` should not be preceded by the preposition `to`."
            ),
            priority: 63,
        })
    }

    fn description(&self) -> &str {
        "Identifies sequences like `ask to us` or `tell to him` and recommends removing the superfluous “to”."
    }
}

#[cfg(test)]
mod tests {
    use super::AskNoPreposition;
    use crate::linting::tests::{assert_lint_count, assert_suggestion_result};

    #[test]
    fn flags_ask() {
        assert_suggestion_result(
            "Nora asked to us about the concert lineup.",
            AskNoPreposition::default(),
            "Nora asked us about the concert lineup.",
        );
    }

    #[test]
    fn flags_ask_all_caps() {
        assert_suggestion_result(
            "NORA ASKED TO US ABOUT THE CONCERT LINEUP.",
            AskNoPreposition::default(),
            "NORA ASKED US ABOUT THE CONCERT LINEUP.",
        );
    }

    #[test]
    fn flags_tell() {
        assert_suggestion_result(
            "Please tell to him the results promptly.",
            AskNoPreposition::default(),
            "Please tell him the results promptly.",
        );
    }

    #[test]
    fn ignores_correct_usage() {
        assert_lint_count(
            "She asked her mentor a difficult question.",
            AskNoPreposition::default(),
            0,
        );
    }

    #[test]
    fn flags_ask_us() {
        assert_suggestion_result(
            "Can you ask to us for directions?",
            AskNoPreposition::default(),
            "Can you ask us for directions?",
        );
    }

    #[test]
    fn flags_asks_him() {
        assert_suggestion_result(
            "Julia asks to him every morning about the report.",
            AskNoPreposition::default(),
            "Julia asks him every morning about the report.",
        );
    }

    #[test]
    fn flags_asked_me() {
        assert_suggestion_result(
            "They asked to me why I left early.",
            AskNoPreposition::default(),
            "They asked me why I left early.",
        );
    }

    #[test]
    fn flags_told_one() {
        assert_suggestion_result(
            "The guide told to one the secret path.",
            AskNoPreposition::default(),
            "The guide told one the secret path.",
        );
    }

    #[test]
    fn flags_telling_it() {
        assert_suggestion_result(
            "She is telling to it with gentle words.",
            AskNoPreposition::default(),
            "She is telling it with gentle words.",
        );
    }

    #[test]
    fn flags_tells_them() {
        assert_suggestion_result(
            "He tells to them stories at night.",
            AskNoPreposition::default(),
            "He tells them stories at night.",
        );
    }

    #[test]
    fn flags_telling_him() {
        assert_suggestion_result(
            "I was telling to him the latest news.",
            AskNoPreposition::default(),
            "I was telling him the latest news.",
        );
    }

    #[test]
    fn flags_asking_you() {
        assert_suggestion_result(
            "Someone is asking to you for help.",
            AskNoPreposition::default(),
            "Someone is asking you for help.",
        );
    }

    #[test]
    fn ignores_ask_question() {
        assert_lint_count(
            "Ask her the question directly.",
            AskNoPreposition::default(),
            0,
        );
    }

    #[test]
    fn ignores_told_to_leave() {
        assert_lint_count(
            "He was told to leave immediately.",
            AskNoPreposition::default(),
            0,
        );
    }

    #[test]
    fn ignores_tell_us() {
        assert_lint_count("Please tell us your name.", AskNoPreposition::default(), 0);
    }

    #[test]
    fn ignores_ask_about() {
        assert_lint_count(
            "They asked about the schedule.",
            AskNoPreposition::default(),
            0,
        );
    }
}
