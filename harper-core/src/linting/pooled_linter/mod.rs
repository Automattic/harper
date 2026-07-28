#[cfg(feature = "concurrent")]
mod multi_thread_pool;
mod pool;
mod single_thread_pool;

use self::pool::Pool;
use self::single_thread_pool::SingleThreadPool;
use crate::Lint;

use super::Linter;

#[cfg(feature = "concurrent")]
type SelectedPool<T> = multi_thread_pool::MultiThreadPool<T>;
#[cfg(not(feature = "concurrent"))]
type SelectedPool<T> = SingleThreadPool<T>;

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
