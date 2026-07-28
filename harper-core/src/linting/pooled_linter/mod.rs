#[cfg(feature = "concurrent")]
mod multi_thread_pool;
mod pool;
mod single_thread_pool;

use self::pool::Pool;
#[cfg(not(feature = "concurrent"))]
use self::single_thread_pool::SingleThreadPool;
use crate::Lint;

use super::Linter;

#[cfg(feature = "concurrent")]
type SelectedPool<T> = multi_thread_pool::MultiThreadPool<T>;
#[cfg(not(feature = "concurrent"))]
type SelectedPool<T> = SingleThreadPool<T>;

/// A `PooledLinter` is a data structure that wraps around a [`Linter`] constructor.
///
/// When the `concurrent` feature is enabled, it uses the provided constructor function to create a
/// pool of child linters. This pool exists so a thread pool can perform linting operations
/// without needing to repeatedly construct or otherwise manage linters. When a linter is needed, an
/// unlocked linter from the pool is borrowed, the linting is performed, then it is released back to
/// the pool. If there are no available linters in the pool, one is added. The pool will rarely, if
/// ever, have more than N elements in it, where N is the number of CPUs available to the program.
///
///
/// When the `concurrent` feature is _disabled_, no pool is instantiated. Instead, a new linter
/// will be constructed for each call. This is understandably quite expensive, so if you intend to
/// use this data structure, it's highly recommended that you enable `concurrent`
pub struct PooledLinter<L: Linter> {
    pool: SelectedPool<L>,
    description: String,
}

impl<L: Linter> PooledLinter<L> {
    pub fn new(ctor: fn() -> L) -> Self {
        let pool = SelectedPool::new(ctor);
        let description = pool.run_with_pool(|i| i.description().to_string());

        Self { pool, description }
    }

    pub fn run_with_inner<B>(&self, callback: impl FnOnce(&mut L) -> B) -> B {
        self.pool.run_with_pool(callback)
    }
}

impl<L: Linter> Linter for PooledLinter<L> {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        self.run_with_inner(|linter| linter.lint(document))
    }

    fn description(&self) -> &str {
        &self.description
    }
}

impl<L: Linter> Clone for PooledLinter<L> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            description: self.description.clone(),
        }
    }
}

#[cfg(test)]
pub mod for_tests {
    /// Constructs a static `PooledLinter` for use in tests. Provide the name of the linter under
    /// test, its type, and an expression to construct it.
    macro_rules! create_test_pool {
        ($linter_name:ident, $linter_ty:ty, $linter_ctor:expr) => {
            mod test_group_container {
                use super::$linter_name;
                use super::*;
                use crate::linting::PooledLinter;
                use std::sync::LazyLock;

                pub static TEST_GROUP: LazyLock<PooledLinter<$linter_ty>> =
                    LazyLock::new(|| PooledLinter::new(|| $linter_ctor));
            }

            fn test_linter() -> crate::linting::PooledLinter<$linter_ty> {
                (*test_group_container::TEST_GROUP).clone()
            }
        };
    }

    pub(crate) use create_test_pool;
}
