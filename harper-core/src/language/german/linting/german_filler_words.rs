use crate::{
    Lrc, Token, TokenStringExt,
    expr::{Expr, SequenceExpr},
    linting::{Chunk, ExprLinter, Lint, LintKind, Suggestion},
    patterns::WordSet,
};

/// Removes common German filler words such as "äh" and "ähm".
pub struct GermanFillerWords {
    expr: SequenceExpr,
}

impl Default for GermanFillerWords {
    fn default() -> Self {
        // Keep this list conservative to avoid false positives on semantic words.
        let filler_words = Lrc::new(WordSet::new(&[
            "äh", "ähm", "öhm", "hm", "hmm", "aeh", "aehm", "oehm",
        ]));

        let pattern = SequenceExpr::any_of(vec![
            Box::new(SequenceExpr::with(filler_words.clone()).then_whitespace()),
            Box::new(SequenceExpr::whitespace().then(filler_words)),
        ]);

        Self { expr: pattern }
    }
}

impl ExprLinter for GermanFillerWords {
    type Unit = Chunk;

    fn expr(&self) -> &dyn Expr {
        &self.expr
    }

    fn match_to_lint(&self, toks: &[Token], _src: &[char]) -> Option<Lint> {
        Some(Lint {
            span: toks.span()?,
            lint_kind: LintKind::Miscellaneous,
            suggestions: vec![Suggestion::Remove],
            message: "Entfernen Sie dieses unnötige Füllwort.".to_string(),
            priority: 31,
        })
    }

    fn description(&self) -> &str {
        "Entfernt unnötige deutsche Füllwörter."
    }
}

#[cfg(test)]
mod tests {
    use super::GermanFillerWords;
    use crate::linting::tests::assert_suggestion_result;

    #[test]
    fn removes_aehm() {
        assert_suggestion_result(
            "Das ist ähm ein Beispiel.",
            GermanFillerWords::default(),
            "Das ist ein Beispiel.",
        );
    }

    #[test]
    fn removes_hm_at_start() {
        assert_suggestion_result(
            "Hm wir müssen das prüfen.",
            GermanFillerWords::default(),
            "wir müssen das prüfen.",
        );
    }

    #[test]
    fn does_not_flag_um_preposition() {
        use crate::{
            Document, language::german::parsers::PlainGerman,
            language::german::spell::german_dictionary,
        };
        let dict = german_dictionary();
        let doc = Document::new(
            "Es ist nicht gut, dass ich mir immer die Nächste um die Ohren schlage.",
            &PlainGerman,
            &*dict,
        );
        let lints = GermanFillerWords::default().lint(&doc);
        assert_eq!(
            lints.len(),
            0,
            "Should not flag 'um' as it's a valid preposition"
        );
    }
}
