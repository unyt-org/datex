use core::async_iter::AsyncIterator;
use core::future::poll_fn;
use core::pin::Pin;

// utility function for async next
pub async fn async_next_pin_box<I>(iter: &mut Pin<Box<I>>) -> Option<I::Item>
where
    I: AsyncIterator + ?Sized,
{
    poll_fn(|cx| {
        iter.as_mut().poll_next(cx)
    })
        .await
}