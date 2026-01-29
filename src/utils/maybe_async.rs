pub enum MaybeAsync<T, F: Future<Output = T>> {
    Sync(T),
    Async(F),
}

pub type MaybeAsyncResult<T, E, F> = MaybeAsync<Result<T, E>, F>;

impl<T, F: Future<Output = T>> MaybeAsync<T, F> {
    pub async fn into_future(self) -> T {
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


pub enum SyncOrAsync<SyncValue, AsyncValue, F: Future<Output = AsyncValue>> {
    Sync(SyncValue),
    Async(F),
}

pub enum SyncOrAsyncResolved<SyncValue, AsyncValue> {
    Sync(SyncValue),
    Async(AsyncValue),
}

impl <SyncValue, AsyncValue, F: Future<Output = AsyncValue>>
    SyncOrAsync<SyncValue, AsyncValue, F>
{
    /// Converts the SyncOrAsync into a future that resolves to SyncOrAsyncResolved.
    pub async fn into_future(self) -> SyncOrAsyncResolved<SyncValue, AsyncValue> {
        match self {
            SyncOrAsync::Sync(value) => SyncOrAsyncResolved::Sync(value),
            SyncOrAsync::Async(fut) => {
                let value = fut.await;
                SyncOrAsyncResolved::Async(value)
            }
        }
    }
}

pub type SyncOrAsyncResult<SyncOkValue, AsyncOkValue, E, F> =
    SyncOrAsync<Result<SyncOkValue, E>, Result<AsyncOkValue, E>, F>;

impl<SyncOkValue, AsyncOkValue, E, F> SyncOrAsyncResult<SyncOkValue, AsyncOkValue, E, F>
where
    F: Future<Output = Result<AsyncOkValue, E>>,
{
    /// Converts the SyncOrAsyncResult into an Option containing the error value.
    pub async fn into_error_future(
        self,
    ) -> Option<E> {
        match self {
            SyncOrAsync::Sync(result) => result.err(),
            SyncOrAsync::Async(fut) => fut.await.err(),
        }
    }
    
    /// Converts the SyncOrAsyncResult into an Option containing the successful value.
    pub async fn into_ok_future(
        self,
    ) -> Option<SyncOrAsyncResolved<SyncOkValue, AsyncOkValue>> {
        match self {
            SyncOrAsync::Sync(result) => {
                match result {
                    Ok(value) => Some(SyncOrAsyncResolved::Sync(value)),
                    Err(_) => None,
                }
            }
            SyncOrAsync::Async(fut) => {
                match fut.await {
                    Ok(value) => Some(SyncOrAsyncResolved::Async(value)),
                    Err(_) => None,
                }
            }
        }
    }
}