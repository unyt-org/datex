use cfg_if::cfg_if;
use core::{clone::Clone, prelude::rust_2024::*};
use futures_util::{SinkExt, StreamExt};

#[cfg(not(feature = "std"))]
pub use async_unsync::{
    bounded::{Receiver as _Receiver, Sender as _Sender},
    unbounded::{
        UnboundedReceiver as _UnboundedReceiver,
        UnboundedSender as _UnboundedSender,
    },
};
#[cfg(feature = "std")]
use futures::channel::mpsc::{
    Receiver as _Receiver, Sender as _Sender,
    UnboundedReceiver as _UnboundedReceiver,
    UnboundedSender as _UnboundedSender,
};

#[derive(Debug)]
pub struct Receiver<T>(_Receiver<T>);
impl<T> Receiver<T> {
    pub fn new(receiver: _Receiver<T>) -> Self {
        Receiver(receiver)
    }

    pub async fn next(&mut self) -> Option<T> {
        #[cfg(feature = "std")]
        {
            self.0.next().await
        }
        #[cfg(not(feature = "std"))]
        {
            self.0.recv().await
        }
    }
}

#[derive(Debug)]
pub struct UnboundedReceiver<T>(_UnboundedReceiver<T>);
impl<T> UnboundedReceiver<T> {
    /// Create a new [UnboundedReceiver] from a given [_UnboundedReceiver].
    pub fn new(receiver: _UnboundedReceiver<T>) -> Self {
        UnboundedReceiver(receiver)
    }

    /// Asynchronously receive the next item from the channel.
    pub async fn next(&mut self) -> Option<T> {
        #[cfg(feature = "std")]
        {
            self.0.next().await
        }
        #[cfg(not(feature = "std"))]
        {
            self.0.recv().await
        }
    }
}

#[derive(Debug)]
pub struct Sender<T>(_Sender<T>);

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender(self.0.clone())
    }
}
impl<T> Sender<T> {
    /// Create a new [Sender] from a given [_Sender].
    pub fn new(sender: _Sender<T>) -> Self {
        Sender(sender)
    }

    /// Attempt to send an item into the channel without waiting.
    /// Returns `Ok(())` if the item was sent successfully, or `Err(())` is closed.
    pub fn start_send(&mut self, item: T) -> Result<(), ()> {
        #[cfg(feature = "std")]
        {
            self.0.start_send(item).map_err(|_| ())
        }
        #[cfg(not(feature = "std"))]
        {
            self.0.try_send(item).map_err(|_| ())
        }
    }

    /// Asynchronously send an item into the channel
    pub async fn send(&mut self, item: T) -> Result<(), ()> {
        #[cfg(feature = "std")]
        {
            self.0.send(item).await.map_err(|_| ()).map(|_| ())
        }
        #[cfg(not(feature = "std"))]
        {
            self.0.send(item).await.map(|_| ()).map_err(|_| ())
        }
    }

    /// Close the channel, preventing any further sends.
    pub fn close_channel(&mut self) {
        #[cfg(feature = "std")]
        {
            self.0.close_channel();
        }
        #[cfg(not(feature = "std"))]
        {}
    }
}

#[derive(Debug)]
pub struct UnboundedSender<T>(_UnboundedSender<T>);

// FIXME #603: derive Clone?
impl<T> Clone for UnboundedSender<T> {
    fn clone(&self) -> Self {
        UnboundedSender(self.0.clone())
    }
}

impl<T> UnboundedSender<T> {
    /// Create a new [UnboundedSender] from a given [_UnboundedSender].
    pub fn new(sender: _UnboundedSender<T>) -> Self {
        UnboundedSender(sender)
    }

    /// Attempt to send an item into the channel without waiting.
    pub fn start_send(&mut self, item: T) -> Result<(), ()> {
        #[cfg(feature = "std")]
        {
            self.0.start_send(item).map_err(|_| ())
        }
        #[cfg(not(feature = "std"))]
        {
            self.0.send(item).map_err(|_| ())
        }
    }

    /// Asynchronously send an item into the channel
    pub async fn send(&mut self, item: T) -> Result<(), ()> {
        #[cfg(feature = "std")]
        {
            self.0.send(item).await.map_err(|_| ()).map(|_| ())
        }
        #[cfg(not(feature = "std"))]
        {
            self.0.send(item).map(|_| ()).map_err(|_| ())
        }
    }

    /// Close the channel, preventing any further sends.
    pub fn close_channel(&self) {
        #[cfg(feature = "std")]
        {
            self.0.close_channel();
        }
        #[cfg(not(feature = "std"))]
        {}
    }
}

cfg_if! {
    if #[cfg(feature = "std")] {
        /// Create a new bounded channel with the specified capacity, returning the sender and receiver.
        pub fn create_bounded_channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
            let (sender, receiver) = futures::channel::mpsc::channel::<T>(capacity);
            (Sender::new(sender), Receiver::new(receiver))
        }

        /// Create a new unbounded channel, returning the sender and receiver.
        pub fn create_unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
            let (sender, receiver) = futures::channel::mpsc::unbounded::<T>();
            (UnboundedSender::new(sender), UnboundedReceiver::new(receiver))
        }
    }
    else {
        /// Create a new bounded channel with the specified capacity, returning the sender and receiver.
        pub fn create_bounded_channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
            let (sender, receiver) = async_unsync::bounded::channel::<T>(capacity).into_split();
            (Sender::new(sender), Receiver::new(receiver))
        }

        /// Create a new unbounded channel, returning the sender and receiver.
        pub fn create_unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
            let (sender, receiver) = async_unsync::unbounded::channel::<T>().into_split();
            (UnboundedSender::new(sender), UnboundedReceiver::new(receiver))
        }
    }
}
