use crate::LSend;
use crate::Span;
use crate::Token;

use super::Expr;

/// A map from an [`Expr`] to arbitrary data.
///
/// When used as a [`Expr`] in and of itself, it simply iterates through
/// all contained exprs, returning the first match found.
/// You should not assume this search is deterministic.
///
/// If you'd like to use this structure in a [`PatternLinter`](crate::linting::PatternLinter), you may want to provide
/// the map as the search expr, then use a pattern lookup once more to determine
/// the corresponding key.
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
    fn run(&self, cursor: usize, tokens: &[Token], source: &[char]) -> Option<Span> {
        self.rows
            .iter()
            .filter_map(|row| row.key.run(cursor, tokens, source))
            .next()
    }
}
