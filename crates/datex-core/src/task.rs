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
        use alloc::boxed::Box;


        pub async fn sleep(duration: core::time::Duration) {
            let mut interval = async_timer::Interval::platform_new(duration);
            interval.wait().await;
        }

        pub async fn timeout<T>(
            duration: core::time::Duration,
            fut: impl Future<Output = T>,
        ) -> Result<T, ()> {
            let mut pinned_fut = Box::pin(fut);
            let work = async_timer::Timed::platform_new(pinned_fut.as_mut(), duration);

            work.await.map_err(|_| ())
        }
    }
}
