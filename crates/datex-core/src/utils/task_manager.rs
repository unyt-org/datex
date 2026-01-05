use crate::channel::mpsc::{UnboundedSender, create_unbounded_channel};
use async_select::select;
use core::{cell::RefCell, fmt::Debug, pin::Pin};
use futures::future::Future;
use futures_util::{
    FutureExt, StreamExt, future::Fuse, stream::FuturesUnordered,
};

use crate::prelude::*;
pub type TaskFuture = Pin<Box<dyn Future<Output = ()>>>;

pub struct TaskManager {
    pub task_sender: RefCell<UnboundedSender<TaskFuture>>,
}

impl Debug for TaskManager {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaskManager").finish()
    }
}

impl TaskManager {
    /// Async task to handle all events for the ComHub
    pub(crate) fn create() -> (TaskManager, impl Future<Output = ()>) {
        let (sender, mut receiver) = create_unbounded_channel::<TaskFuture>();

        (
            TaskManager {
                task_sender: RefCell::new(sender),
            },
            async move {
                let mut tasks = FuturesUnordered::<Fuse<TaskFuture>>::new();

                // iterate over new_socket_iterators
                loop {
                    // check for new sockets from all iterators
                    select! {
                        // Poll for completed futures
                        Some(_) = tasks.next() => {}

                        // Poll for new futures from channel
                        Some(new_fut) = receiver.next().fuse() => {
                            tasks.push(new_fut.fuse());
                        }
                        complete => unreachable!(),
                    }
                }
            },
        )
    }

    /// Registers a new task on the ComHub
    pub(crate) fn register_task<F>(&self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.task_sender
            .borrow_mut()
            .start_send(Box::pin(fut))
            .unwrap();
    }
}
