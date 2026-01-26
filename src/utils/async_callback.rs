use crate::stdlib::sync::Arc;
use futures::future::BoxFuture;


/// A generic async callback wrapper.
/// Can be cloned and called from multiple tasks.
#[derive(Clone)]
pub struct AsyncCallback<In, Out> {
    inner: Arc<dyn Fn(In) -> BoxFuture<'static, Out> + Send + Sync>,
}

impl<In, Out> AsyncCallback<In, Out> {
    /// Create a new AsyncCallback from an async function or closure.
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(In) -> Fut + Send + Sync + 'static,
        Fut: core::future::Future<Output = Out> + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(move |arg| Box::pin(f(arg))),
        }
    }

    /// Call the async callback.
    pub async fn call(&self, arg: In) -> Out {
        (self.inner)(arg).await
    }
}