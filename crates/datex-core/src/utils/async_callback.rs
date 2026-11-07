use futures::future::LocalBoxFuture;

use crate::prelude::*;

/// A generic async callback wrapper.
/// Can be cloned and called from multiple tasks.
pub struct AsyncCallback<In, Out> {
    inner: Rc<dyn Fn(In) -> LocalBoxFuture<'static, Out>>,
}

impl<In, Out> Clone for AsyncCallback<In, Out> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }
}


impl<In, Out> AsyncCallback<In, Out> {
    /// Create a new AsyncCallback from an async function or closure.
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(In) -> Fut + 'static,
        Fut: core::future::Future<Output = Out> + 'static,
    {
        Self {
            inner: Rc::new(move |arg| Box::pin(f(arg))),
        }
    }

    /// Call the async callback.
    pub async fn call(&self, arg: In) -> Out {
        (self.inner)(arg).await
    }
}
