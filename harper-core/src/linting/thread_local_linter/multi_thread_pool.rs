use parking_lot::RwLock;
use smallvec::{SmallVec, smallvec};
use std::sync::{Arc, Mutex};

use super::pool::Pool;

pub struct MultiThreadPool<T> {
    ctor: fn() -> T,
    pool: Arc<RwLock<SmallVec<[Arc<Mutex<T>>; 32]>>>,
}

impl<T> Pool<T> for MultiThreadPool<T> {
    fn new(ctor: fn() -> T) -> Self {
        let first = ctor();
        Self {
            pool: Arc::new(RwLock::new(smallvec![Arc::new(Mutex::new(first))])),
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

        let mut new_item = (self.ctor)();
        let result = callback(&mut new_item);

        {
            let mut write_pool = self.pool.write();
            write_pool.push(Arc::new(Mutex::new(new_item)));
        }

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
