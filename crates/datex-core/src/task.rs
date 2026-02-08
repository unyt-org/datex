use cfg_if::cfg_if;
use core::future::Future;
cfg_if! {
    if #[cfg(feature = "tokio_runtime")] {
        pub async fn timeout<T>(
            duration: core::time::Duration,
            fut: impl Future<Output = T>,
        ) -> Result<T, ()> {
            tokio::time::timeout(duration, fut)
                .await
                .map_err(|_| ())
        }

        // pub fn spawn_local<F>(fut: F)-> tokio::task::JoinHandle<()>
        // where
        //     F: Future<Output = ()> + 'static,
        // {
        //     tokio::task::spawn_local(fut)
        // }
        // pub fn spawn<F>(fut: F) -> tokio::task::JoinHandle<F::Output>
        // where
        //     F: Future<Output = ()> + Send + 'static,
        // {
        //     tokio::spawn(fut)
        // }
        // pub fn spawn_blocking<F, R>(f: F) -> tokio::task::JoinHandle<R>
        // where
        //     F: FnOnce() -> R + Send + 'static,
        //     R: Send + 'static,
        // {
        //     tokio::task::spawn_blocking(f)
        // }
        pub async fn sleep(dur: core::time::Duration) {
            tokio::time::sleep(dur).await;
        }

    }

    else if #[cfg(feature = "wasm_runtime")] {
        use futures::future;

        pub async fn timeout<T>(
            duration: core::time::Duration,
            fut: impl Future<Output = T>,
        ) -> Result<T, ()> {
            let timeout_fut = sleep(duration);
            futures::pin_mut!(fut);
            futures::pin_mut!(timeout_fut);

            match future::select(fut, timeout_fut).await {
                future::Either::Left((res, _)) => Ok(res),
                future::Either::Right(_) => Err(()),
            }
        }
        pub async fn sleep(dur: core::time::Duration) {
            gloo_timers::future::sleep(dur).await;
        }

        // pub fn spawn_local<F>(fut: F)
        // where
        //     F: core::future::Future<Output = ()> + 'static,
        // {
        //     wasm_bindgen_futures::spawn_local(fut);
        // }
        // pub fn spawn<F>(fut: F)
        // where
        //     F: core::future::Future<Output = ()> + 'static,
        // {
        //     wasm_bindgen_futures::spawn_local(fut);
        // }
        // pub fn spawn_blocking<F>(_fut: F) -> !
        // where
        //     F: core::future::Future + 'static,
        // {
        //     core::panic!("`spawn_blocking` is not supported in the wasm runtime.");
        // }
    }

    else if #[cfg(feature = "embassy_runtime")] {
        use embassy_time::{Duration, Timer};
        use embassy_futures::select::select;
        use embassy_futures::select::Either;

        pub async fn sleep(dur: core::time::Duration) {
            let emb_dur = Duration::from_millis(dur.as_millis() as u64);
            Timer::after(emb_dur).await;
        }

        pub async fn timeout<T>(
            duration: core::time::Duration,
            fut: impl Future<Output = T>,
        ) -> Result<T, ()> {
            let emb_dur = Duration::from_millis(duration.as_millis() as u64);
            let timeout = Timer::after(emb_dur);

            match select(fut, timeout).await {
                Either::First(t) => Ok(t),
                Either::Second(_) => Err(()),
            }
        }

    }
    else {
        compile_error!("Unsupported runtime. Please enable either 'tokio_runtime', 'embassy_runtime' or 'wasm_runtime' feature.");
    }
}
