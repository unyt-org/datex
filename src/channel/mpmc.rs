use cfg_if::cfg_if;

#[cfg(any(feature = "tokio_runtime", feature = "wasm_runtime"))]
use async_broadcast::{
    Receiver as _BroadcastReceiver, Sender as _BroadcastSender,
};

#[cfg(feature = "embassy_runtime")]
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{
        Channel, Receiver as _BroadcastReceiver, Sender as _BroadcastSender,
    },
};

#[derive(Debug)]
pub struct BroadcastChannel<T: Clone> {
    sender: BroadcastSender<T>,
    receiver: BroadcastReceiver<T>,
}
impl<T: Clone> BroadcastChannel<T> {
    pub fn new<const CAPACITY: usize>() -> Self {
        let (sender, receiver) = create_bounded_channel::<T, CAPACITY>();
        BroadcastChannel { sender, receiver }
    }

    pub fn sender(&self) -> BroadcastSender<T> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> BroadcastReceiver<T> {
        self.receiver.clone()
    }
    pub fn close_channel(&mut self) {
        self.sender.close_channel();
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastSender<T: Clone>(_BroadcastSender<T>);

impl<T: Clone> BroadcastSender<T> {
    pub fn new(sender: _BroadcastSender<T>) -> Self {
        BroadcastSender(sender)
    }

    pub fn start_send(&mut self, item: T) -> Result<(), ()> {
        #[cfg(any(feature = "tokio_runtime", feature = "wasm_runtime"))]
        {
            self.0.try_broadcast(item).map_err(|_| ()).map(|_| ())
        }
        #[cfg(feature = "embassy_runtime")]
        {
            self.0.try_send(item).map_err(|_| ())
        }
    }

    pub async fn send(&mut self, item: T) -> Result<(), ()> {
        #[cfg(any(feature = "tokio_runtime", feature = "wasm_runtime"))]
        {
            self.0.broadcast(item).await.map_err(|_| ()).map(|_| ())
        }
        #[cfg(feature = "embassy_runtime")]
        {
            self.0.send(item).await.map(|_| ()).map_err(|_| ())
        }
    }

    pub fn close_channel(&mut self) {
        #[cfg(any(feature = "tokio_runtime", feature = "wasm_runtime"))]
        {
            self.0.close();
        }
        #[cfg(feature = "embassy_runtime")]
        {}
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastReceiver<T: Clone>(_BroadcastReceiver<T>);
impl<T: Clone> BroadcastReceiver<T> {
    pub fn new(receiver: _BroadcastReceiver<T>) -> Self {
        BroadcastReceiver(receiver)
    }

    pub async fn next(&mut self) -> Result<T, ()> {
        #[cfg(any(feature = "tokio_runtime", feature = "wasm_runtime"))]
        {
            self.0.recv().await.map_err(|_| ())
        }
        #[cfg(feature = "embassy_runtime")]
        {
            self.0.receive().await.map_err(|_| ())
        }
    }

    pub fn try_next(&mut self) -> Result<T, ()> {
        #[cfg(any(feature = "tokio_runtime", feature = "wasm_runtime"))]
        {
            self.0.try_recv().map_err(|_| ())
        }
        #[cfg(feature = "embassy_runtime")]
        {
            self.0.try_receive().map_err(|_| ())
        }
    }
}

cfg_if! {
    if #[cfg(any(feature = "tokio_runtime", feature = "wasm_runtime"))] {
        pub fn create_bounded_channel<T: Clone, const CAPACITY: usize>() -> (BroadcastSender<T>, BroadcastReceiver<T>) {
            let (sender, receiver) = async_broadcast::broadcast(CAPACITY);
            (BroadcastSender::new(sender), BroadcastReceiver::new(receiver))
        }
    }
    else if #[cfg(feature = "embassy_runtime")] {
        pub fn create_bounded_channel<T, const CAPACITY: usize>() -> (BroadcastSender<T>, BroadcastReceiver<T>) {
            let channel = embassy_sync::channel::Channel::<NoopRawMutex, T, CAPACITY>::new();
            (BroadcastSender::new(channel.sender()), BroadcastReceiver::new(channel.receiver()))
        }
    }
    else {
        compile_error!("Unsupported runtime. Please enable either 'tokio_runtime', 'embassy_runtime' or 'wasm_runtime' feature.");
    }
}
