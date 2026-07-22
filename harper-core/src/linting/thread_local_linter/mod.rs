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

#[derive(Clone)]
pub struct ThreadLocalLinter<L: Linter> {
    pool: SelectedPool<L>,
    description: String,
}

impl<L: Linter> ThreadLocalLinter<L> {
    pub fn new(ctor: fn() -> L) -> Self {
        let pool = SelectedPool::new(ctor);
        let description = pool.run_with_pool(|i| i.description().to_string());

        Self { pool, description }
    }

    pub fn run_with_inner<B>(&self, callback: impl FnOnce(&mut L) -> B) -> B {
        self.pool.run_with_pool(callback)
    }
}

impl<L: Linter> Linter for ThreadLocalLinter<L> {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        self.run_with_inner(|linter| linter.lint(document))
    }

    fn description(&self) -> &str {
        &self.description
    }
}
