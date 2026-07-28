use boxcar::Vec;
use std::sync::{Arc, Mutex};

use super::pool::Pool;

pub struct MultiThreadPool<T> {
    ctor: fn() -> T,
    pool: Arc<Vec<Mutex<T>>>,
}

impl<T> Pool<T> for MultiThreadPool<T> {
    fn new(ctor: fn() -> T) -> Self {
        let first = ctor();
        let pool = Vec::new();
        pool.push(Mutex::new(first));

        Self {
            pool: Arc::new(pool),
            ctor,
        }
    }

    /// Run a callback with access to a member of the pool.
    fn run_with_pool<B, C: FnOnce(&mut T) -> B>(&self, callback: C) -> B {
        // Attempt to grab an open copy.
        {
            for (_, item) in self.pool.iter() {
                if let Ok(mut l) = item.try_lock() {
                    return callback(&mut l);
                }
            }
        }

        let mut new_item = (self.ctor)();
        let result = callback(&mut new_item);

        self.pool.push(Mutex::new(new_item));

        return result;
    }
}

impl<T> Clone for MultiThreadPool<T> {
    fn clone(&self) -> Self {
        Self {
            ctor: self.ctor.clone(),
            pool: self.pool.clone(),
        }
    }
}
