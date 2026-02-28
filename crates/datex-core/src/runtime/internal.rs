use crate::{
    channel::mpsc::UnboundedReceiver,
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
        com_interfaces::local_loopback_interface::LocalLoopbackInterfaceSetupData,
    },
    prelude::*,
    runtime::{
        RuntimeConfig, RuntimeConfigInterface,
        execution::{
            ExecutionError,
            context::{
                ExecutionContext, ExecutionMode, RemoteExecutionContext,
                ScriptExecutionError,
            },
        },
        memory::Memory,
    },
    time::Instant,
    utils::task_manager::TaskManager,
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};
use alloc::rc::Rc;
use core::{cell::RefCell, pin::Pin, slice};
use log::{debug, error, info};
use crate::channel::mpsc::create_unbounded_channel;

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
               &mut ExecutionContext::local(ExecutionMode::Static, $self_rc.clone())
            }
        }
    };
}

impl RuntimeInternal {
    pub(crate) fn new(
        endpoint: Endpoint,
        memory: RefCell<Memory>,
        config: RuntimeConfig,
        com_hub: Rc<ComHub>,
        task_manager: TaskManager,
        incoming_sections_receiver: UnboundedReceiver<IncomingSection>,
    ) -> RuntimeInternal {
        RuntimeInternal {
            endpoint,
            memory,
            config,
            com_hub,
            task_manager,
            incoming_sections_receiver: RefCell::new(
                incoming_sections_receiver,
            ),
            execution_contexts: RefCell::new(HashMap::new()),
        }
    }

    pub fn stub() -> RuntimeInternal {
        let (sender, receiver) = create_unbounded_channel();
        RuntimeInternal::new(
            Endpoint::default(),
            RefCell::new(Memory::default()),
            RuntimeConfig::default(),
            ComHub::create(Endpoint::default(), sender).0,
            TaskManager::create().0,
            receiver,
        )
    }

    /// Creates all interfaces configured in the runtime config
    async fn create_configured_interfaces(&self) {
        if let Some(interfaces) = &self.config.interfaces {
            for RuntimeConfigInterface {
                interface_type,
                setup_data: config,
                priority,
            } in interfaces.iter()
            {
                let create_future = self
                    .com_hub
                    .clone()
                    .create_interface(interface_type, config.clone(), *priority)
                    .await;
                match create_future {
                    Err(err) => {
                        error!(
                            "Failed to create interface {interface_type}: {err:?}"
                        )
                    }
                    Ok((_, ready_receiver)) => {
                        if let Some(ready_receiver) = ready_receiver {
                            let _ = ready_receiver.await;
                        }
                    }
                }
            }
        }
    }

    async fn init_local_loopback_interface(&self) {
        // add default local loopback interface
        let local_interface_setup_data =
            LocalLoopbackInterfaceSetupData.create_interface().unwrap();

        let ready_signal = self
            .com_hub
            .clone()
            .add_interface_from_configuration(
                local_interface_setup_data,
                InterfacePriority::None,
            )
            .expect("Failed to add local loopback interface");
        // local loopback interface is single socket interface and should always return a ready signal
        // which should always resolve to Ok
        ready_signal.unwrap().await.unwrap()
    }

    /// Performs asynchronous initialization of the runtime
    pub(crate) async fn init_async(&self) {
        // create local loopback interface and other configured interfaces
        self.init_local_loopback_interface().await;
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
            ExecutionContext::local(
                ExecutionMode::unbounded(),
                self_rc.clone(),
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

    pub(crate) async fn execute_incoming_section(
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

    pub fn get_env(&self) -> HashMap<String, String> {
        self.config.env.clone().unwrap_or_default()
    }
}
