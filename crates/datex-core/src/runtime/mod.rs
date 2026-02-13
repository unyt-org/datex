use crate::{
    channel::mpsc::{UnboundedReceiver, create_unbounded_channel},
    collections::HashMap,
    global::{
        dxb_block::{
            DXBBlock, IncomingEndpointContextSectionId, IncomingSection,
            OutgoingContextId,
        },
        protocol_structures::{
            block_header::BlockHeader, encrypted_header::EncryptedHeader,
            routing_header::RoutingHeader,
        },
    },
    network::{
        com_hub::{
            ComHub, InterfacePriority, network_response::ResponseOptions,
        },
        com_interfaces::com_interface::factory::{
            ComInterfaceAsyncFactory, ComInterfaceSyncFactory,
        },
    },
    runtime::execution::{ExecutionError, context::ExecutionMode},
    serde::{error::SerializationError, serializer::to_value_container},
    time::Instant,
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};

use crate::prelude::*;
use async_select::select;
use core::{
    cell::RefCell, fmt::Debug, pin::Pin, result::Result, slice, unreachable,
};
use execution::context::{
    ExecutionContext, RemoteExecutionContext, ScriptExecutionError,
};
use futures::{FutureExt, future};
use futures_util::join;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

pub mod dif_interface;
pub mod execution;
mod incoming_sections;
pub mod memory;

use self::memory::Memory;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct Runtime {
    pub version: String,
    pub internal: Rc<RuntimeInternal>,
}

impl Debug for Runtime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Runtime")
            .field("version", &self.version)
            .finish()
    }
}

#[derive(Debug)]
pub struct RuntimeInternal {
    pub memory: RefCell<Memory>,
    pub com_hub: Rc<ComHub>,
    pub endpoint: Endpoint,
    pub config: RuntimeConfig,

    pub task_manager: TaskManager,

    // receiver for incoming sections from com hub
    pub(crate) incoming_sections_receiver:
        RefCell<UnboundedReceiver<IncomingSection>>,

    /// active execution contexts, stored by context_id
    pub execution_contexts:
        RefCell<HashMap<IncomingEndpointContextSectionId, ExecutionContext>>,
}

macro_rules! get_execution_context {
    // take context and self_rc as parameters
    ($self_rc:expr, $execution_context:expr) => {
        match $execution_context {
            Some(context) => {
                // set current runtime in execution context if local execution context
                if let &mut ExecutionContext::Local(ref mut local_context) = context {
                    local_context.set_runtime_internal($self_rc.clone());
                }
                context
            },
            None => {
               &mut ExecutionContext::local_with_runtime_internal($self_rc.clone(), ExecutionMode::Static)
            }
        }
    };
}

impl RuntimeInternal {
    /// Creates all interfaces configured in the runtime config
    async fn create_configured_interfaces(&self) {
        if let Some(interfaces) = &self.config.interfaces {
            for RuntimeConfigInterface {
                interface_type,
                setup_data: config,
                priority,
            } in interfaces.iter()
            {
                if let Err(err) = self
                    .com_hub
                    .clone()
                    .create_interface(interface_type, config.clone(), *priority)
                    .await
                {
                    error!(
                        "Failed to create interface {interface_type}: {err:?}"
                    );
                } else {
                    info!("Created interface: {interface_type}");
                }
            }
        }
    }

    /// Performs asynchronous initialization of the runtime
    async fn init_async(&self) {
        // create configured interfaces
        self.create_configured_interfaces().await;
    }

    #[cfg(feature = "compiler")]
    pub async fn execute(
        self_rc: Rc<RuntimeInternal>,
        script: &str,
        inserted_values: &[ValueContainer],
        execution_context: Option<&mut ExecutionContext>,
    ) -> Result<Option<ValueContainer>, ScriptExecutionError> {
        let execution_context =
            get_execution_context!(self_rc, execution_context);
        let compile_start = Instant::now();
        let dxb = execution_context.compile(script, inserted_values)?;
        debug!(
            "[Compilation took {} ms]",
            compile_start.elapsed().as_millis()
        );
        let execute_start = Instant::now();
        let result = RuntimeInternal::execute_dxb(
            self_rc,
            dxb,
            Some(execution_context),
            true,
        )
        .await
        .map_err(ScriptExecutionError::from);
        debug!(
            "[Execution took {} ms]",
            execute_start.elapsed().as_millis()
        );
        result
    }

    #[cfg(feature = "compiler")]
    pub fn execute_sync(
        self_rc: Rc<RuntimeInternal>,
        script: &str,
        inserted_values: &[ValueContainer],
        execution_context: Option<&mut ExecutionContext>,
    ) -> Result<Option<ValueContainer>, ScriptExecutionError> {
        let execution_context =
            get_execution_context!(self_rc, execution_context);
        let compile_start = Instant::now();
        let dxb = execution_context.compile(script, inserted_values)?;
        debug!(
            "[Compilation took {} ms]",
            compile_start.elapsed().as_millis()
        );
        let execute_start = Instant::now();
        let result = RuntimeInternal::execute_dxb_sync(
            self_rc,
            &dxb,
            Some(execution_context),
            true,
        )
        .map_err(ScriptExecutionError::from);
        debug!(
            "[Execution took {} ms]",
            execute_start.elapsed().as_millis()
        );
        result
    }

    pub fn execute_dxb<'a>(
        self_rc: Rc<RuntimeInternal>,
        dxb: Vec<u8>,
        execution_context: Option<&'a mut ExecutionContext>,
        _end_execution: bool,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ValueContainer>, ExecutionError>>
                + 'a,
        >,
    > {
        Box::pin(async move {
            let execution_context =
                get_execution_context!(self_rc, execution_context);
            match execution_context {
                ExecutionContext::Remote(context) => {
                    RuntimeInternal::execute_remote(self_rc, context, dxb).await
                }
                ExecutionContext::Local(_) => {
                    execution_context.execute_dxb(&dxb).await
                }
            }
        })
    }

    pub fn execute_dxb_sync(
        self_rc: Rc<RuntimeInternal>,
        dxb: &[u8],
        execution_context: Option<&mut ExecutionContext>,
        _end_execution: bool,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let execution_context =
            get_execution_context!(self_rc, execution_context);
        match execution_context {
            ExecutionContext::Remote(_) => {
                Err(ExecutionError::RequiresAsyncExecution)
            }
            ExecutionContext::Local(_) => {
                execution_context.execute_dxb_sync(dxb)
            }
        }
    }

    /// Returns the existing execution context for the given context_id,
    /// or creates a new one if it doesn't exist.
    /// To reuse the context later, the caller must store it back in the map after use.
    fn take_execution_context(
        self_rc: Rc<RuntimeInternal>,
        context_id: &IncomingEndpointContextSectionId,
    ) -> ExecutionContext {
        let mut execution_contexts = self_rc.execution_contexts.borrow_mut();
        // get execution context by context_id or create a new one if it doesn't exist
        let execution_context = execution_contexts.remove(context_id);
        if let Some(context) = execution_context {
            context
        } else {
            ExecutionContext::local_with_runtime_internal(
                self_rc.clone(),
                ExecutionMode::unbounded(),
            )
        }
    }

    pub async fn execute_remote(
        self_rc: Rc<RuntimeInternal>,
        remote_execution_context: &mut RemoteExecutionContext,
        dxb: Vec<u8>,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let routing_header: RoutingHeader = RoutingHeader::default()
            .with_sender(self_rc.endpoint.clone())
            .to_owned();

        // get existing context_id for context, or create a new one
        let context_id =
            remote_execution_context.context_id.unwrap_or_else(|| {
                // if the context_id is not set, we create a new one
                remote_execution_context.context_id =
                    Some(self_rc.com_hub.block_handler.get_new_context_id());
                remote_execution_context.context_id.unwrap()
            });

        let block_header = BlockHeader {
            context_id,
            ..BlockHeader::default()
        };
        let encrypted_header = EncryptedHeader::default();

        let mut block =
            DXBBlock::new(routing_header, block_header, encrypted_header, dxb);

        block
            .set_receivers(slice::from_ref(&remote_execution_context.endpoint));

        let response = self_rc
            .com_hub
            .send_own_block_await_response(block, ResponseOptions::default())
            .await
            .remove(0)?;
        let incoming_section = response.take_incoming_section();
        RuntimeInternal::execute_incoming_section(self_rc, incoming_section)
            .await
            .0
    }

    async fn execute_incoming_section(
        self_rc: Rc<RuntimeInternal>,
        mut incoming_section: IncomingSection,
    ) -> (
        Result<Option<ValueContainer>, ExecutionError>,
        Endpoint,
        OutgoingContextId,
    ) {
        let section_context_id =
            incoming_section.get_section_context_id().clone();
        let mut context =
            Self::take_execution_context(self_rc.clone(), &section_context_id);
        info!(
            "Executing incoming section with index: {}",
            incoming_section.get_section_index()
        );

        let mut result = None;
        let mut last_block = None;

        // iterate over the blocks in the incoming section
        loop {
            let block = incoming_section.next().await;
            if let Some(block) = block {
                let res = RuntimeInternal::execute_dxb_block_local(
                    self_rc.clone(),
                    block.clone(),
                    Some(&mut context),
                )
                .await;
                if let Err(err) = res {
                    return (
                        Err(err),
                        block.sender().clone(),
                        block.block_header.context_id,
                    );
                }
                result = res.unwrap();
                last_block = Some(block);
            } else {
                break;
            }
        }

        if last_block.is_none() {
            unreachable!("Incoming section must contain at least one block");
        }
        let last_block = last_block.unwrap();
        let sender_endpoint = last_block.sender().clone();
        let context_id = last_block.block_header.context_id;

        // insert the context back into the map for future use
        // TODO #638: is this needed or can we drop the context after execution here?
        self_rc
            .execution_contexts
            .borrow_mut()
            .insert(section_context_id, context);

        (Ok(result), sender_endpoint, context_id)
    }

    async fn execute_dxb_block_local(
        self_rc: Rc<RuntimeInternal>,
        block: DXBBlock,
        execution_context: Option<&mut ExecutionContext>,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let execution_context =
            get_execution_context!(self_rc, execution_context);
        // assert that the execution context is local
        if !core::matches!(execution_context, ExecutionContext::Local(_)) {
            unreachable!(
                "Execution context must be local for executing a DXB block"
            );
        }
        let dxb = block.body;
        let end_execution =
            block.block_header.flags_and_timestamp.is_end_of_section();
        RuntimeInternal::execute_dxb(
            self_rc,
            dxb,
            Some(execution_context),
            end_execution,
        )
        .await
    }
}
use crate::{
    network::{
        com_hub::is_none_variant,
        com_interfaces::local_loopback_interface::LocalLoopbackInterfaceSetupData,
    },
    utils::task_manager::TaskManager,
};
use crate::serde::deserializer;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "wasm_runtime", derive(tsify::Tsify))]
pub struct RuntimeConfigInterface {
    #[serde(rename = "type")]
    pub interface_type: String,
    #[serde(rename = "config")]
    #[cfg_attr(feature = "wasm_runtime", tsify(type = "unknown"))]
    pub setup_data: ValueContainer,

    #[serde(default, skip_serializing_if = "is_none_variant")]
    pub priority: InterfacePriority,
}

impl RuntimeConfigInterface {
    pub fn new<T: Serialize>(
        interface_type: &str,
        setup_data: T,
    ) -> Result<RuntimeConfigInterface, SerializationError> {
        Ok(RuntimeConfigInterface {
            interface_type: interface_type.to_string(),
            priority: InterfacePriority::default(),
            setup_data: to_value_container(&setup_data)?,
        })
    }

    pub fn new_from_value_container(
        interface_type: &str,
        config: ValueContainer,
    ) -> RuntimeConfigInterface {
        RuntimeConfigInterface {
            priority: InterfacePriority::default(),
            interface_type: interface_type.to_string(),
            setup_data: config,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct RuntimeConfig {
    pub endpoint: Option<Endpoint>,
    pub interfaces: Option<Vec<RuntimeConfigInterface>>,
    pub env: Option<HashMap<String, String>>,
}

impl RuntimeConfig {
    pub fn new_with_endpoint(endpoint: Endpoint) -> Self {
        RuntimeConfig {
            endpoint: Some(endpoint),
            interfaces: None,
            env: None,
        }
    }

    pub fn add_interface<T: Serialize>(
        &mut self,
        interface_type: String,
        config: T,
        priority: InterfacePriority,
    ) -> Result<(), SerializationError> {
        let config = to_value_container(&config)?;
        let interface = RuntimeConfigInterface {
            interface_type,
            setup_data: config,
            priority,
        };
        if let Some(interfaces) = &mut self.interfaces {
            interfaces.push(interface);
        } else {
            self.interfaces = Some(vec![interface]);
        }

        Ok(())
    }
}

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

/// publicly exposed wrapper impl for the Runtime
/// around RuntimeInternal
impl Runtime {
    fn init_local_loopback_interface(&self) {
        // add default local loopback interface
        let local_interface_setup_data =
            LocalLoopbackInterfaceSetupData.create_interface().unwrap();

        self.com_hub()
            .add_interface_from_configuration(
                local_interface_setup_data,
                InterfacePriority::None,
            )
            .expect("Failed to add local loopback interface");
    }

    pub fn com_hub(&self) -> Rc<ComHub> {
        self.internal.com_hub.clone()
    }
    pub fn endpoint(&self) -> Endpoint {
        self.internal.endpoint.clone()
    }

    pub fn internal(&self) -> Rc<RuntimeInternal> {
        Rc::clone(&self.internal)
    }

    pub fn memory(&self) -> &RefCell<Memory> {
        &self.internal.memory
    }

    #[cfg(feature = "compiler")]
    pub async fn execute(
        &self,
        script: &str,
        inserted_values: &[ValueContainer],
        execution_context: Option<&mut ExecutionContext>,
    ) -> Result<Option<ValueContainer>, ScriptExecutionError> {
        RuntimeInternal::execute(
            self.internal(),
            script,
            inserted_values,
            execution_context,
        )
        .await
    }

    #[cfg(feature = "compiler")]
    pub fn execute_sync(
        &self,
        script: &str,
        inserted_values: &[ValueContainer],
        execution_context: Option<&mut ExecutionContext>,
    ) -> Result<Option<ValueContainer>, ScriptExecutionError> {
        RuntimeInternal::execute_sync(
            self.internal(),
            script,
            inserted_values,
            execution_context,
        )
    }

    pub async fn execute_dxb<'a>(
        &'a self,
        dxb: Vec<u8>,
        execution_context: Option<&'a mut ExecutionContext>,
        end_execution: bool,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        RuntimeInternal::execute_dxb(
            self.internal(),
            dxb,
            execution_context,
            end_execution,
        )
        .await
    }

    pub fn execute_dxb_sync(
        &self,
        dxb: &[u8],
        execution_context: Option<&mut ExecutionContext>,
        end_execution: bool,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        RuntimeInternal::execute_dxb_sync(
            self.internal(),
            dxb,
            execution_context,
            end_execution,
        )
    }

    async fn execute_remote(
        &self,
        remote_execution_context: &mut RemoteExecutionContext,
        dxb: Vec<u8>,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        RuntimeInternal::execute_remote(
            self.internal(),
            remote_execution_context,
            dxb,
        )
        .await
    }
}
