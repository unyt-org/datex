use futures_util::FutureExt;
use crate::stdlib::rc::Rc;
use core::cell::RefCell;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::future::Future;
use core::pin::Pin;
use async_select::select;
use crate::channel::mpsc::{create_unbounded_channel, UnboundedReceiver, UnboundedSender};

struct TaskManager {
    tasks: RefCell<FuturesUnordered<Pin<Box<dyn Future<Output = ()>>>>>,
    new_task_notifier: RefCell<UnboundedReceiver<()>>, // signals when a new task is added
    new_task_sender: UnboundedSender<()>,
}

impl TaskManager {
    fn new() -> Rc<Self> {
        let (tx, rx) = create_unbounded_channel::<()>();
        Rc::new(TaskManager {
            tasks: RefCell::new(FuturesUnordered::new()),
            new_task_notifier: RefCell::new(rx),
            new_task_sender: tx,
        })
    }

    /// Starts the task handling loop
    async fn handle_tasks(self: Rc<Self>) {
        loop {
            // temporarily take ownership of the tasks
            let task_fut_opt = {
                let mut tasks_ref = self.tasks.borrow_mut();
                tasks_ref.next()
            };

            match task_fut_opt.await {
                Some(_) => {
                    // `.next()` returns a future that borrows, so now we can await safely
                }
                None => {
                    // no tasks, can yield to runtime or wait for a notifier
                    futures::future::pending::<()>().await;
                }
            }
        }
    }

    fn add_task(&mut self, fut: Pin<Box<dyn Future<Output = ()>>>) {
        self.tasks.borrow_mut().push(fut);
        let _ = self.new_task_sender.start_send(()); // notify
    }
}
