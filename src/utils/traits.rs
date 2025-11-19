use futures::Future;

pub trait AsyncFrom<T> {
    fn async_from(value: T) -> impl Future<Output = Self>;
}
