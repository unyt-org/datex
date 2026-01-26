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
}