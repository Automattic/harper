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
        self.matches(&tokens[cursor..], source).map(|i| i as isize)
    }
}
