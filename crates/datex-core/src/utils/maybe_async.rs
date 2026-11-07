use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Ready<T>(Option<T>);
pub fn ready<T>(t: T) -> Ready<T> {
    Ready(Some(t))
}
impl<T> Future for Ready<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<T> {
        let this = unsafe { self.get_unchecked_mut() };
        Poll::Ready(this.0.take().unwrap())
    }
}
pub enum MaybeAsync<T, F: Future<Output = T>> {
    Sync(T),
    Async(F),
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}
impl<L, R> Future for Either<L, R>
where
    L: Future,
    R: Future<Output = L::Output>,
{
    type Output = L::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            match self.get_unchecked_mut() {
                Either::Left(l) => Pin::new_unchecked(l).poll(cx),
                Either::Right(r) => Pin::new_unchecked(r).poll(cx),
            }
        }
    }
}

pub type MaybeAsyncResult<T, E, F> = MaybeAsync<Result<T, E>, F>;

impl<T, F: Future<Output = T>> MaybeAsync<T, F> {
    pub fn into_future(self) -> Either<Ready<T>, F> {
        match self {
            MaybeAsync::Sync(v) => Either::Left(ready(v)),
            MaybeAsync::Async(f) => Either::Right(f),
        }
    }

    pub fn map<U, Func>(self, func: Func) -> MaybeAsync<U, MapFuture<F, Func>>
    where
        Func: FnOnce(T) -> U,
    {
        match self {
            MaybeAsync::Sync(v) => MaybeAsync::Sync(func(v)),
            MaybeAsync::Async(fut) => MaybeAsync::Async(MapFuture {
                fut,
                func: Some(func),
            }),
        }
    }
}

pub struct MapFuture<Fut, Func> {
    fut: Fut,
    func: Option<Func>,
}

impl<T, U, Fut, Func> Future for MapFuture<Fut, Func>
where
    Fut: Future<Output = T>,
    Func: FnOnce(T) -> U,
{
    type Output = U;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<U> {
        unsafe {
            let this = self.get_unchecked_mut();
            let t = match Pin::new_unchecked(&mut this.fut).poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(t) => t,
            };
            let f = this.func.take().unwrap();
            Poll::Ready(f(t))
        }
    }
}
//
// pub enum SyncOrAsync<SyncValue, AsyncValue, F: Future<Output = AsyncValue>> {
//     Sync(SyncValue),
//     Async(F),
// }
//
pub enum SyncOrAsyncResolved<SyncValue, AsyncValue> {
    Sync(SyncValue),
    Async(AsyncValue),
}

use core::marker::PhantomData;

pub struct MapToResolvedAsync<SyncValue, Fut> {
    fut: Fut,
    _pd: PhantomData<SyncValue>,
}

pub struct ErrOptionFuture<Fut>(Fut);

impl<AsyncOkValue, E, Fut> Future for ErrOptionFuture<Fut>
where
    Fut: Future<Output = Result<AsyncOkValue, E>>,
{
    type Output = Option<E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<E>> {
        unsafe {
            let this = self.get_unchecked_mut();
            match Pin::new_unchecked(&mut this.0).poll(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(r) => Poll::Ready(r.err()),
            }
        }
    }
}

pub struct OkOptionFuture<SyncOkValue, AsyncOkValue, E, Fut> {
    fut: Fut,
    _phantom: core::marker::PhantomData<(SyncOkValue, AsyncOkValue, E)>,
}


impl<SyncOkValue, E, F>
    MaybeAsyncResult<SyncOkValue, E, F>
where
    F: Future<Output = Result<SyncOkValue, E>>,
{
    pub fn into_error_future(
        self,
    ) -> Either<Ready<Option<E>>, ErrOptionFuture<F>> {
        match self {
            MaybeAsyncResult::Sync(r) => Either::Left(ready(r.err())),
            MaybeAsyncResult::Async(fut) => Either::Right(ErrOptionFuture(fut)),
        }
    }

    pub fn into_ok_future(
        self,
    ) -> Either<
        Ready<Option<SyncOrAsyncResolved<SyncOkValue, SyncOkValue>>>,
        OkOptionFuture<SyncOkValue, SyncOkValue, E, F>,
    > {
        match self {
            MaybeAsyncResult::Sync(r) => {
                let v = r.ok().map(SyncOrAsyncResolved::Sync);
                Either::Left(ready(v))
            }
            MaybeAsyncResult::Async(fut) => Either::Right(OkOptionFuture {
                fut,
                _phantom: core::marker::PhantomData,
            }),
        }
    }

    pub fn into_result(self) -> Either<Ready<Result<SyncOkValue, E>>, F>
    where
        SyncOkValue: Into<SyncOkValue>,
    {
        match self {
            MaybeAsyncResult::Sync(r) => Either::Left(ready(r.map(Into::into))),
            MaybeAsyncResult::Async(fut) => Either::Right(fut),
        }
    }
}


pub struct FlattenFuture<OuterF, InnerF, T> {
    state: FlattenState<OuterF, InnerF, T>,
}

enum FlattenState<OuterF, InnerF, T> {
    Outer(OuterF),
    Inner(InnerF),
    Done(Option<T>),
}

impl<OuterF, InnerF, T> Future for FlattenFuture<OuterF, InnerF, T>
where
    OuterF: Future<Output = MaybeAsync<T, InnerF>>,
    InnerF: Future<Output = T>,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        unsafe {
            let this = self.get_unchecked_mut();
            loop {
                match &mut this.state {
                    FlattenState::Outer(outer) => {
                        let inner = match Pin::new_unchecked(outer).poll(cx) {
                            Poll::Pending => return Poll::Pending,
                            Poll::Ready(v) => v,
                        };
                        match inner {
                            MaybeAsync::Sync(v) => {
                                this.state = FlattenState::Done(Some(v))
                            }
                            MaybeAsync::Async(f) => {
                                this.state = FlattenState::Inner(f)
                            }
                        }
                    }
                    FlattenState::Inner(inner) => {
                        let v = match Pin::new_unchecked(inner).poll(cx) {
                            Poll::Pending => return Poll::Pending,
                            Poll::Ready(v) => v,
                        };
                        this.state = FlattenState::Done(Some(v));
                    }
                    FlattenState::Done(v) => {
                        return Poll::Ready(v.take().unwrap());
                    }
                }
            }
        }
    }
}

impl<T, InnerF, OuterF> MaybeAsync<MaybeAsync<T, InnerF>, OuterF>
where
    InnerF: Future<Output = T>,
    OuterF: Future<Output = MaybeAsync<T, InnerF>>,
{
    pub fn flatten(self) -> MaybeAsync<T, FlattenFuture<OuterF, InnerF, T>> {
        match self {
            MaybeAsync::Sync(inner) => match inner {
                MaybeAsync::Sync(v) => MaybeAsync::Sync(v),
                MaybeAsync::Async(inner_fut) => {
                    MaybeAsync::Async(FlattenFuture {
                        state: FlattenState::Inner(inner_fut),
                    })
                }
            },
            MaybeAsync::Async(outer_fut) => MaybeAsync::Async(FlattenFuture {
                state: FlattenState::Outer(outer_fut),
            }),
        }
    }
}

pub struct MapAsyncResultToResolved<SyncOkValue, AsyncOkValue, E, Fut> {
    fut: Fut,
    _pd: PhantomData<(SyncOkValue, AsyncOkValue, E)>,
}