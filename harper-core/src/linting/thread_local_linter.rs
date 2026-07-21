use cached::sync_sync::RwLock;

use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::{Lint, Linter};

pub struct ThreadLocalLinter<L: Linter> {
    ctor: fn() -> L,
    pool: Arc<RwLock<Vec<Arc<Mutex<L>>>>>,
    description: String,
}

impl<L: Linter> ThreadLocalLinter<L> {
    pub fn new(ctor: fn() -> L) -> Self {
        let first = ctor();
        let description = first.description().to_string();

        Self {
            ctor,
            pool: Arc::new(RwLock::new(vec![Arc::new(Mutex::new(first))])),
            description,
        }
    }

    pub fn run_with_pool<B>(&self, callback: impl FnOnce(&mut L) -> B) -> B {
        // Attempt to grab an open copy.
        {
            let read_pool = self.pool.read();
            for i in read_pool.iter() {
                let item = i.clone();
                if let Ok(mut l) = item.try_lock() {
                    return callback(&mut l);
                }
            }
        }

        {
            let mut write_pool = self.pool.write();
            write_pool.push(Arc::new(Mutex::new((self.ctor)())));
            return callback(&mut write_pool.last().unwrap().clone().lock().unwrap());
        }
    }
}

impl<L: Linter> Linter for ThreadLocalLinter<L> {
    fn lint(&mut self, document: &crate::Document) -> Vec<Lint> {
        self.run_with_pool(|linter| linter.lint(document))
    }

    fn description(&self) -> &str {
        &self.description
    }
}

impl<L: Linter> Clone for ThreadLocalLinter<L> {
    fn clone(&self) -> Self {
        Self {
            ctor: self.ctor.clone(),
            pool: self.pool.clone(),
            description: self.description.clone(),
        }
    }
}
