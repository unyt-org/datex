use core::{async_iter::AsyncIterator, future::poll_fn, pin::Pin};

use crate::prelude::*;
// utility function for async next
pub async fn async_next_pin_box<I>(iter: &mut Pin<Box<I>>) -> Option<I::Item>
where
    I: AsyncIterator + ?Sized,
{
    poll_fn(|cx| iter.as_mut().poll_next(cx)).await
}
