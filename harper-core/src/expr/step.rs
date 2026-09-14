use crate::{LSend, Token, patterns::Pattern};

/// An atomic step within a larger expression.
///
/// Its principle job is to identify (if any) the next position of the cursor.
/// When cursor is moved, all tokens between the current cursor and the target position will be
/// added to the match group.
pub trait Step: LSend {
    fn step(&self, tokens: &[Token], cursor: usize, source: &[char]) -> Option<isize>;
    fn step_rev(&self, tokens: &[Token], cursor: usize, source: &[char]) -> Option<isize>;
}

impl<P> Step for P
where
    P: Pattern,
{
    fn step(&self, tokens: &[Token], cursor: usize, source: &[char]) -> Option<isize> {
        self.matches(&tokens[cursor..], source).map(|i| i as isize)
    }
    fn step_rev(&self, tokens: &[Token], cursor: usize, source: &[char]) -> Option<isize> {
        self.matches(&tokens[cursor..], source).map(|i| i as isize)
    }
}

#[cfg(test)]
mod tests {
    use super::Step;
    use crate::{
        Document,
        patterns::{WhitespacePattern, Word},
    };

    #[test]
    fn test_step() {
        let doc = Document::new_plain_english_curated("Hello wurld");
        let toks = doc.get_tokens();
        let src = doc.get_source();
        let hello = Word::new("Hello").step(&toks, 0, &src);
        assert_eq!(hello, Some(1));
        let ws = WhitespacePattern.step(&toks, 1, &src);
        assert_eq!(ws, Some(1));
        let world = Word::new("world").step(&toks, 2, &src);
        assert_eq!(world, None);
    }

    #[test]
    fn test_step_rev() {
        let doc = Document::new_plain_english_curated("Hello world");
        let toks = doc.get_tokens();
        let src = doc.get_source();
        let world = Word::new("world").step_rev(&toks, 2, &src);
        assert_eq!(world, Some(1)); // step_rev now returns positive offset
        let ws = WhitespacePattern.step_rev(&toks, 1, &src);
        assert_eq!(ws, Some(1)); // step_rev now returns positive offset
        let hello = Word::new("Helelo").step_rev(&toks, 0, &src);
        assert_eq!(hello, None);
    }
}
