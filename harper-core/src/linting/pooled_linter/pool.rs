pub trait Pool<T>: Clone {
    fn new(ctor: fn() -> T) -> Self;
    fn run_with_pool<B, C: FnOnce(&mut T) -> B>(&self, callback: C) -> B;
}
