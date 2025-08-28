use async_trait::async_trait;
use futures::{Future, StreamExt, future::BoxFuture};

use std::pin::Pin;
use tokio_stream::wrappers::ReceiverStream;

#[async_trait]
pub trait AsyncFrom<T> {
    async fn async_from(value: T) -> Self;
}

pub trait Apply<T> {
    fn apply_async<F>(self, call: F) -> impl Future<Output = ()>
    where
        F: Fn(T) -> Pin<Box<dyn Future<Output = ()>>>;

    fn apply_with_capture_async<C, F>(self, capture: C, call: F) -> impl Future<Output = C>
    where
        F: for<'a> Fn(&'a mut C, T) -> BoxFuture<'a, ()>;

    fn fold_async<F, C>(self, capture: C, call: F) -> impl Future<Output = C>
    where
        F: Fn(C, T) -> Pin<Box<dyn Future<Output = C>>>;
}

impl<T> Apply<T> for ReceiverStream<T> {
    async fn apply_async<F>(mut self, call: F)
    where
        F: Fn(T) -> Pin<Box<dyn Future<Output = ()>>>,
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

    async fn fold_async<F, C>(mut self, mut capture: C, call: F) -> C
    where
        F: Fn(C, T) -> Pin<Box<dyn Future<Output = C>>>,
    {
        while let Some(result) = self.next().await {
            capture = call(capture, result).await;
        }
        capture
    }
}
