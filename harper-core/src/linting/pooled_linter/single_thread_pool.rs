use super::pool::Pool;

pub struct SingleThreadPool<T> {
    ctor: fn() -> T,
}

impl<T> Pool<T> for SingleThreadPool<T> {
    fn new(ctor: fn() -> T) -> Self {
        Self { ctor }
    }

    fn run_with_pool<B, C: FnOnce(&mut T) -> B>(&self, callback: C) -> B {
        let mut item = (self.ctor)();
        callback(&mut item)
    }
}

impl<T> Clone for SingleThreadPool<T> {
    fn clone(&self) -> Self {
        Self {
            ctor: self.ctor.clone(),
        }
    }
}
