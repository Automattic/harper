use parking_lot::RwLock;
use std::sync::{Arc, Mutex};

use super::pool::Pool;

pub struct MultiThreadPool<T> {
    ctor: fn() -> T,
    pool: Arc<RwLock<Vec<Arc<Mutex<T>>>>>,
}

impl<T> Pool<T> for MultiThreadPool<T> {
    fn new(ctor: fn() -> T) -> Self {
        let first = ctor();
        Self {
            pool: Arc::new(RwLock::new(vec![Arc::new(Mutex::new(first))])),
            ctor,
        }
    }

    /// Run a callback with access to a member of the pool.
    fn run_with_pool<B, C: FnOnce(&mut T) -> B>(&self, callback: C) -> B {
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

impl<T> Clone for MultiThreadPool<T> {
    fn clone(&self) -> Self {
        Self {
            ctor: self.ctor.clone(),
            pool: self.pool.clone(),
        }
    }
}
