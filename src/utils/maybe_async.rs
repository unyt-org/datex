pub enum MaybeAsync<T, F: Future<Output = T>> {
    Sync(T),
    Async(F),
}

pub type MaybeAsyncResult<T, E, F> = MaybeAsync<Result<T, E>, F>;

impl<T, F: Future<Output = T>> MaybeAsync<T, F> {
    pub async fn into_inner(self) -> T {
        match self {
            MaybeAsync::Sync(value) => value,
            MaybeAsync::Async(fut) => fut.await,
        }
    }

    pub fn map<U, Func>(
        self,
        func: Func,
    ) -> MaybeAsync<U, impl Future<Output = U>>
    where
        Func: FnOnce(T) -> U,
    {
        match self {
            MaybeAsync::Sync(value) => MaybeAsync::Sync(func(value)),
            MaybeAsync::Async(fut) => MaybeAsync::Async(async move {
                let value = fut.await;
                func(value)
            }),
        }
    }
}
