use async_trait::async_trait;
use futures::{Future, StreamExt, future::BoxFuture};

use tokio_stream::wrappers::ReceiverStream;

#[async_trait]
pub trait AsyncFrom<T> {
    async fn async_from(value: T) -> Self;
}
pub trait Apply<T> {
    fn apply_async<F, Fut>(self, call: F) -> impl Future<Output = ()>
    where
        F: Fn(T) -> Fut,
        Fut: Future<Output = ()> + Send + 'static;

    fn apply_with_capture_async<C, F>(self, capture: C, call: F) -> impl Future<Output = C>
    where
        F: for<'a> Fn(&'a mut C, T) -> BoxFuture<'a, ()>;

    fn fold_async<F, C, Fut>(self, capture: C, call: F) -> impl Future<Output = C>
    where
        F: Fn(C, T) -> Fut,
        Fut: Future<Output = C> + Send + 'static;
}

impl<T> Apply<T> for ReceiverStream<T> {
    async fn apply_async<F, Fut>(mut self, call: F)
    where
        F: Fn(T) -> Fut,
        Fut: Future + Send + 'static,
    {
        while let Some(result) = self.next().await {
            call(result).await;
        }
    }

    async fn apply_with_capture_async<C, F>(mut self, mut capture: C, call: F) -> C
    where
        F: for<'a> Fn(&'a mut C, T) -> BoxFuture<'a, ()>,
    {
        while let Some(result) = self.next().await {
            call(&mut capture, result).await;
        }
        capture
    }

    async fn fold_async<F, C, Fut>(mut self, mut capture: C, call: F) -> C
    where
        F: Fn(C, T) -> Fut,
        Fut: Future<Output = C> + Send + 'static,
    {
        while let Some(result) = self.next().await {
            capture = call(capture, result).await;
        }
        capture
    }
}
