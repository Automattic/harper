use crate::LSend;
use crate::Span;
use crate::Token;

use super::Expr;

/// A map from an [`Expr`] to arbitrary data.
///
/// It has been a common pattern for rule authors to build a list of expressions that match a
/// grammatical error.
/// Then, depending on which expression was matched, a suggestion is chosen from another list.
///
/// The [`ExprMap`] unifies these two lists into one.
///
/// A great example of this is the [`PronounInfectionBe`](crate::linting::PronounInflectionBe)
/// rule.
/// It builds a list of incorrect `PRONOUN + BE` combinations, alongside their corrections.
///
/// When used as a [`Expr`] in and of itself, it simply iterates through
/// all contained expressions, returning the first match found.
/// You should not assume this search is deterministic.
pub struct ExprMap<T>
where
    T: LSend,
{
    rows: Vec<Row<T>>,
}

struct Row<T>
where
    T: LSend,
{
    pub key: Box<dyn Expr>,
    pub element: T,
}

impl<T> Default for ExprMap<T>
where
    T: LSend,
{
    fn default() -> Self {
        Self {
            rows: Default::default(),
        }
    }
}

impl<T> ExprMap<T>
where
    T: LSend,
{
    pub fn insert(&mut self, expr: impl Expr + 'static, value: T) {
        self.rows.push(Row {
            key: Box::new(expr),
            element: value,
        });
    }

    /// Look up the corresponding value for the given map.
    pub fn lookup(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<&T> {
        for row in &self.rows {
            let len = row.key.run(cursor, tokens, source);

            if len.is_some() {
                return Some(&row.element);
            }
        }

        None
    }
}

impl<T> Expr for ExprMap<T>
where
    T: LSend,
{
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span<Token>> {
        self.rows
            .iter()
            .filter_map(|row| row.key.run(cursor, tokens, source))
            .next()
    }
}
