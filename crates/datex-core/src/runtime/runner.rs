use crate::prelude::*;
use futures_util::FutureExt;
use alloc::rc::Rc;
use core::cell::RefCell;
use crate::collections::HashMap;
use core::pin::Pin;
use async_select::select;
use futures_util::{future, join};
use log::info;
use crate::channel::mpsc::create_unbounded_channel;
use crate::global::dxb_block::IncomingSection;
use crate::network::com_hub::ComHub;
use crate::runtime::{Runtime, RuntimeConfig, RuntimeInternal, VERSION};
use crate::runtime::env::RuntimeEnv;
use crate::runtime::memory::Memory;
use crate::utils::task_manager::TaskManager;
use crate::values::core_values::endpoint::Endpoint;

pub struct RuntimeRunner {
    pub runtime: Runtime,
    pub task_future: Pin<Box<dyn Future<Output = ()>>>,
}

impl RuntimeRunner {
    /// Creates a new runtime instance with the given configuration and global context.
    /// Note: If the endpoint is not specified in the config, a random endpoint will be generated.
    pub fn new(config: RuntimeConfig) -> RuntimeRunner {
        info!(
            "Runtime initialized - Version {VERSION} Time: {}",
            crate::time::now_ms()
        );

        let endpoint = config.endpoint.clone().unwrap_or_else(Endpoint::random);

        let (task_manager, runtime_task_future) = TaskManager::create();

        let (incoming_sections_sender, incoming_sections_receiver) =
            create_unbounded_channel::<IncomingSection>();

        let (com_hub, com_hub_task_future) =
            ComHub::create(endpoint.clone(), incoming_sections_sender);
        let memory = RefCell::new(Memory::new(endpoint.clone()));

        let runtime = Runtime {
            version: VERSION.to_string(),
            internal: Rc::new(RuntimeInternal {
                endpoint,
                memory,
                config,
                com_hub,
                task_manager,
                incoming_sections_receiver: RefCell::new(
                    incoming_sections_receiver,
                ),
                execution_contexts: RefCell::new(HashMap::new()),
                env: RuntimeEnv::default()
            }),
        };
        runtime.init_local_loopback_interface();

        let runtime_internal = runtime.internal.clone();

        // await all task futures
        let task_future = async {
            join!(
                // com hub task manager
                com_hub_task_future,
                // runtime task manager
                runtime_task_future,
                // runtime incoming sections handler
                runtime_internal.handle_incoming_sections_task()
            );
        };

        RuntimeRunner {
            runtime,
            task_future: Box::pin(task_future),
        }
    }

    // Starts the runtime, runs the provided app logic, and returns its result.
    // The runtime will exit when the app logic completes.
    pub async fn run<AppReturn, AppFuture>(
        self,
        app_logic: impl FnOnce(Runtime) -> AppFuture,
    ) -> AppReturn
    where
        AppFuture: Future<Output = AppReturn>,
    {
        // initialize the runtime
        self.runtime.internal.init_async().await;
        // start the app logic
        let app_future = app_logic(self.runtime);

        // run until the app logic completes
        select! {
            _ = self.task_future.fuse() => {
                unreachable!("Runtime task future exited unexpectedly");
            },
            exit_value = app_future.fuse() => {
                exit_value
            },
        }
    }

    /// Starts the runtime and runs indefinitely, executing the provided app logic.
    pub async fn run_forever<AppReturn, AppFuture>(
        self,
        app_logic: impl FnOnce(Runtime) -> AppFuture,
    ) -> !
    where
        AppFuture: Future<Output = AppReturn>,
    {
        // initialize the runtime
        self.runtime.internal.init_async().await;
        // start the app logic
        let app_future = app_logic(self.runtime);

        // run indefinitely (runtime future should never exit)
        future::join(self.task_future, app_future).await;
        unreachable!("Both runtime and app logic futures exited unexpectedly");
    }
}
