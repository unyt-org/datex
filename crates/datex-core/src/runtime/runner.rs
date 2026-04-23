use crate::{
    channel::mpsc::create_unbounded_channel,
    global::dxb_block::IncomingSection,
    network::com_hub::ComHub,
    prelude::*,
    runtime::{
        Runtime, RuntimeConfig, RuntimeInternal, VERSION, memory::Memory,
        pointer_address_provider::SelfOwnedPointerAddressProvider,
    },
    time::now_ms,
    utils::task_manager::TaskManager,
    values::core_values::endpoint::Endpoint,
};
use async_select::select;
use core::{cell::RefCell, pin::Pin};
use futures::channel::oneshot;
use futures_util::{
    future,
    future::{Join, join},
    join,
};
use log::info;

pub struct RuntimeRunner {
    pub runtime: Runtime,
    pub task_future: Pin<Box<dyn Future<Output = ()>>>,
}

impl RuntimeRunner {
    /// Creates a new runtime instance with the given configuration and global context.
    /// Note: If the endpoint is not specified in the config, a random endpoint will be generated.
    pub fn new(config: RuntimeConfig) -> RuntimeRunner {
        let endpoint = config.endpoint.clone().unwrap_or_else(Endpoint::random);

        let (task_manager, runtime_task_future) = TaskManager::create();

        let (incoming_sections_sender, incoming_sections_receiver) =
            create_unbounded_channel::<IncomingSection>();

        let (com_hub, com_hub_task_future) =
            ComHub::create(endpoint.clone(), incoming_sections_sender);
        let memory = RefCell::new(Memory::new());
        let pointer_address_provider = RefCell::new(
            SelfOwnedPointerAddressProvider::new(endpoint.clone()),
        );

        let runtime = Runtime::new(RuntimeInternal::new(
            endpoint,
            memory,
            pointer_address_provider,
            config,
            com_hub,
            task_manager,
            incoming_sections_receiver,
        ));

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
        let (runtime_future, app_future) =
            self.create_runtime_and_app_future(app_logic);

        // run until the app logic completes
        select! {
            _ = runtime_future => {
                unreachable!("Runtime task future exited unexpectedly");
            },
            exit_value = app_future => {
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
        let (runtime_future, app_future) =
            self.create_runtime_and_app_future(app_logic);

        // run indefinitely (runtime future should never exit)
        future::join(runtime_future, app_future).await;
        unreachable!("Both runtime and app logic futures exited unexpectedly");
    }

    /// Creates the runtime future and the app future, ensuring that the app logic starts executing only after the runtime has completed its initialization.
    fn create_runtime_and_app_future<AppReturn, AppFuture>(
        self,
        app_logic: impl FnOnce(Runtime) -> AppFuture,
    ) -> (
        Join<impl Future<Output = ()>, impl Future<Output = ()>>,
        impl Future<Output = AppReturn>,
    )
    where
        AppFuture: Future<Output = AppReturn>,
    {
        let (init_ready_sender, init_ready_receiver) = oneshot::channel();
        let runtime = self.runtime.clone();
        let runtime_future = join(
            // initialize the runtime
            async move {
                runtime.internal.init_async().await;
                init_ready_sender.send(()).unwrap();
            },
            // run tasks
            self.task_future,
        );

        // start the app logic
        let app_future = async move {
            // wait for runtime initialization to complete before starting app logic
            init_ready_receiver.await.unwrap();

            info!("Runtime initialized - Version {VERSION} Time: {}", now_ms());

            app_logic(self.runtime).await
        };

        (runtime_future, app_future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_runner() {
        let runner = RuntimeRunner::new(RuntimeConfig::default());
        let mut runtime = None;
        runner
            .run(async |runtime_inner| {
                runtime = Some(runtime_inner);
            })
            .await;

        assert!(runtime.is_some());

        // check if local loopback interface was fully initialized and is present in the com hub
        let com_hub = runtime.unwrap().com_hub();
        let interface_map = com_hub.interfaces_manager().interfaces.borrow();
        let local_loopback_interface =
            interface_map.iter().find(|(uuid, interface)| {
                interface.properties.interface_type == "local"
            });
        let (local_loopback_interface_uuid, _) = local_loopback_interface
            .expect("Local loopback interface not found in com hub");

        // check if socket for the local loopback interface is present in the com hub
        let socket_map = com_hub.socket_manager().sockets.borrow();
        let local_loopback_socket = socket_map
            .values()
            .find(|socket| {
                &socket.interface_uuid == local_loopback_interface_uuid
            })
            .expect("Local loopback socket not found in com hub");
    }
}
