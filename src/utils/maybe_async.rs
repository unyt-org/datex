pub enum MaybeAsync<T, F: Future<Output = T> + 'static> {
    Sync(T),
    Async(F),
}

pub type MaybeAsyncResult<T, E, F> = MaybeAsync<Result<T, E>, F>;

impl <T, F: Future<Output = T> + 'static> MaybeAsync<T, F> {
    pub async fn into_inner(self) -> T {
        match self {
            MaybeAsync::Sync(value) => value,
            MaybeAsync::Async(fut) => fut.await,
        }
    }

    /// Maps a MaybeAsync<T, F> to MaybeAsync<U, G> by applying the provided function to the contained value.
    pub fn map<U, G: Future<Output = U> + 'static, Func>(self, func: Func) -> MaybeAsync<U, G>
    where
        Func: FnOnce(T) -> U + 'static,
    {
        match self {
            MaybeAsync::Sync(value) => MaybeAsync::Sync(func(value)),
            MaybeAsync::Async(fut) => {
                let new_fut = async move {
                    let value = fut.await;
                    func(value)
                };
                MaybeAsync::Async(new_fut)
            }
        }
    }
}