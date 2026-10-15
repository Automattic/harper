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
        let filler_words = Lrc::new(WordSet::new([
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

    fn match_to_lint(&self, toks: &[Token], src: &[char]) -> Option<Lint> {
        // `HM` is the most-cited patient in the memory literature, not someone
        // hesitating. A filler is a transcribed sound, so it is written `hm`
        // or, opening a sentence, `Hm`; all caps means initials or an
        // abbreviation, and the suggestion is to *delete* it.
        let word = toks.iter().find(|token| token.kind.is_word())?;
        let spelling = word.span.get_content(src);
        if spelling.len() > 1 && spelling.iter().all(|c| !c.is_lowercase()) {
            return None;
        }

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
    use crate::linting::{Linter, tests::assert_suggestion_result};

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

    /// Initials are not hesitations, and the suggestion here is to delete.
    #[test]
    fn does_not_flag_initials() {
        use crate::{
            Document, language::german::parsers::PlainGerman,
            language::german::spell::german_dictionary,
        };
        let dict = german_dictionary();
        let doc = Document::new(
            "Oft zitiert wird der Fall des Patienten HM, dem beide Hippocampi entfernt wurden.",
            &PlainGerman,
            &*dict,
        );
        assert!(
            GermanFillerWords::default().lint(&doc).is_empty(),
            "'HM' in all caps is a name, not a filler"
        );
    }
}
