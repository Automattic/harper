use crate::{LSend, Token, patterns::Pattern};

/// An atomic step within a larger expression.
///
/// Its principle job is to identify (if any) the next position of the cursor.
/// When cursor is moved, all tokens between the current cursor and the target position will be
/// added to the match group.
pub trait Step: LSend {
    fn step(&self, tokens: &[Token], cursor: usize, source: &[char]) -> Option<isize>;
}

impl<P> Step for P
where
    P: Pattern,
{
    fn step(&self, tokens: &[Token], cursor: usize, source: &[char]) -> Option<isize> {
        // Callers may hand us a cursor that sits past the end, which happens when the
        // preceding step consumed the final token. That is a failed match, not a bug.
        self.matches(tokens.get(cursor..)?, source)
            .map(|i| i as isize)
    }
}

#[cfg(test)]
mod tests {
    use super::Step;
    use crate::Document;
    use crate::patterns::NominalPhrase;

    /// `QuantifierNumeralConflict` steps a pattern at cursor 1 over the tokens that
    /// follow a match. When the match ends the chunk, that slice is empty and the
    /// cursor sits past the end, which used to panic instead of failing to match.
    #[test]
    fn cursor_past_the_end_is_a_failed_match() {
        let doc = Document::new_plain_english_curated("both 2");
        let empty: &[crate::Token] = &[];

        assert_eq!(NominalPhrase.step(empty, 1, doc.get_source()), None);
        assert_eq!(NominalPhrase.step(empty, 0, doc.get_source()), None);
    }

    #[test]
    fn cursor_at_the_end_is_a_failed_match() {
        let doc = Document::new_plain_english_curated("both 2");
        let toks: Vec<crate::Token> = doc.tokens().cloned().collect();

        assert_eq!(
            NominalPhrase.step(&toks, toks.len(), doc.get_source()),
            None
        );
    }
}
