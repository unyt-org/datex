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
use crate::disassembler::print_disassembled;
use crate::global::protocol_structures::instruction_data::RawLocalPointerAddress;
use crate::runtime::execution::execution_input::ExecutionCallerMetadata;
use crate::runtime::execution::InvalidProgramError;
use crate::runtime::request_move::compile_request_move;
use crate::shared_values::pointer::EndpointOwnedPointer;
use crate::shared_values::pointer_address::{EndpointOwnedPointerAddress, PointerAddress, ExternalPointerAddress};
use crate::shared_values::shared_container::{SharedContainer, SharedContainerInner, SharedContainerMutability};
use crate::values::core_value::CoreValue;
use crate::values::value::Value;

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

    /// list of currently owned shared values that are in the approved for moving to another endpoint
    pub moving_pointers: RefCell<HashMap<Endpoint, HashMap<EndpointOwnedPointerAddress, SharedContainer>>>,
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
               &mut ExecutionContext::local(ExecutionMode::Static, $self_rc.clone(), ExecutionCallerMetadata::local_default())
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
            moving_pointers: RefCell::new(HashMap::new()),
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
        self: Rc<RuntimeInternal>,
        script: &str,
        inserted_values: &[ValueContainer],
        execution_context: Option<&mut ExecutionContext>,
    ) -> Result<Option<ValueContainer>, ScriptExecutionError> {
        let execution_context =
            get_execution_context!(self, execution_context);
        let compile_start = Instant::now();
        let dxb = execution_context.compile(script, inserted_values)?;
        debug!(
            "[Compilation took {} ms]",
            compile_start.elapsed().as_millis()
        );
        let execute_start = Instant::now();
        let result = RuntimeInternal::execute_dxb(
            self,
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
        self: Rc<RuntimeInternal>,
        script: &str,
        inserted_values: &[ValueContainer],
        execution_context: Option<&mut ExecutionContext>,
    ) -> Result<Option<ValueContainer>, ScriptExecutionError> {
        let execution_context =
            get_execution_context!(self, execution_context);
        let compile_start = Instant::now();
        let dxb = execution_context.compile(script, inserted_values)?;
        debug!(
            "[Compilation took {} ms]",
            compile_start.elapsed().as_millis()
        );
        let execute_start = Instant::now();
        let result = RuntimeInternal::execute_dxb_sync(
            self,
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
        self: Rc<RuntimeInternal>,
        dxb_body: Vec<u8>,
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
                get_execution_context!(self, execution_context);
            match execution_context {
                ExecutionContext::Remote(context) => {
                    RuntimeInternal::execute_remote(self, context, dxb_body).await
                }
                ExecutionContext::Local(_) => {
                    execution_context.execute_dxb(&dxb_body).await
                }
            }
        })
    }

    pub fn execute_dxb_sync(
        self: Rc<RuntimeInternal>,
        dxb: &[u8],
        execution_context: Option<&mut ExecutionContext>,
        _end_execution: bool,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let execution_context =
            get_execution_context!(self, execution_context);
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
        self: Rc<RuntimeInternal>,
        context_id: &IncomingEndpointContextSectionId,
        incoming_section: &IncomingSection,
    ) -> ExecutionContext {
        let mut execution_contexts = self.execution_contexts.borrow_mut();
        // get execution context by context_id or create a new one if it doesn't exist
        let execution_context = execution_contexts.remove(context_id);
        if let Some(context) = execution_context {
            context
        } else {
            let caller_metadata = ExecutionCallerMetadata {
                endpoint: incoming_section.get_sender(),
            };
            ExecutionContext::local(ExecutionMode::unbounded(), self.clone(), caller_metadata)
        }
    }

    pub async fn execute_remote(
        self: Rc<RuntimeInternal>,
        remote_execution_context: &mut RemoteExecutionContext,
        dxb_body: Vec<u8>,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let routing_header: RoutingHeader = RoutingHeader::default()
            .with_sender(self.endpoint.clone())
            .to_owned();

        // get existing context_id for context, or create a new one
        let context_id =
            remote_execution_context.context_id.unwrap_or_else(|| {
                // if the context_id is not set, we create a new one
                remote_execution_context.context_id =
                    Some(self.com_hub.block_handler.get_new_context_id());
                remote_execution_context.context_id.unwrap()
            });

        let block_header = BlockHeader {
            context_id,
            ..BlockHeader::default()
        };
        let encrypted_header = EncryptedHeader::default();

        let mut block =
            DXBBlock::new(routing_header, block_header, encrypted_header, dxb_body);

        block
            .set_receivers(slice::from_ref(&remote_execution_context.endpoint));

        let response = self
            .com_hub
            .send_own_block_await_response(block, ResponseOptions::default())
            .await
            .remove(0)?;
        let incoming_section = response.take_incoming_section();
        RuntimeInternal::execute_incoming_section(self, incoming_section)
            .await
            .0
    }

    pub(crate) async fn execute_incoming_section(
        self: Rc<RuntimeInternal>,
        mut incoming_section: IncomingSection,
    ) -> (
        Result<Option<ValueContainer>, ExecutionError>,
        Endpoint,
        OutgoingContextId,
    ) {
        let section_context_id =
            incoming_section.get_section_context_id().clone();
        let mut context =
            Self::take_execution_context(self.clone(), &section_context_id, &incoming_section);
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
                    self.clone(),
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
        self
            .execution_contexts
            .borrow_mut()
            .insert(section_context_id, context);

        (Ok(result), sender_endpoint, context_id)
    }

    async fn execute_dxb_block_local(
        self: Rc<RuntimeInternal>,
        block: DXBBlock,
        execution_context: Option<&mut ExecutionContext>,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let execution_context =
            get_execution_context!(self, execution_context);
        // assert that the execution context is local
        if !core::matches!(execution_context, ExecutionContext::Local(_)) {
            unreachable!(
                "Execution context must be local for executing a DXB block"
            );
        }
        let dxb = block.body;
        print_disassembled(&dxb);
        let end_execution =
            block.block_header.flags_and_timestamp.is_end_of_section();
        RuntimeInternal::execute_dxb(
            self,
            dxb,
            Some(execution_context),
            end_execution,
        )
        .await
    }

    /// Request to move a list of external pointers from an endpoint to the local endpoint
    /// This only works if the local endpoint has the permission to move the pointers, either because
    /// it was allowed via a PERFORM_MOVE from the remote endpoint, or because the local endpoint has
    /// extended permissions
    pub(crate) async fn request_pointer_move(
        self: Rc<RuntimeInternal>,
        from_endpoint: &Endpoint,
        pointers: Vec<(SharedContainerMutability, RawLocalPointerAddress)>,
    ) -> Result<Vec<SharedContainer>, ExecutionError> {
        let pointer_mapping = pointers.into_iter().map(|original| {
            (original, RawLocalPointerAddress { bytes: self.memory.borrow_mut().get_new_owned_local_pointer().address().address})
        }).collect::<Vec<_>>();
        let body = compile_request_move(
            &(pointer_mapping
                .iter()
                .map(|((_, original), new)| (original.clone(), new.clone()))
                .collect::<Vec<_>>())
        );
        let moved_values = self.clone().execute_dxb(
            body,
            Some(&mut ExecutionContext::Remote(RemoteExecutionContext::new(from_endpoint.clone(), ExecutionMode::Static))),
            true
        ).await?;
        // moved values should be list
        match moved_values {
            Some(ValueContainer::Local(Value {inner: CoreValue::List(list), ..})) => {
                let pointer_values = list.into_vec();
                let owned_values = pointer_values.into_iter()
                    .zip(pointer_mapping.into_iter())
                    .map(|(value, ((mutability, _), new_address))| {
                    SharedContainer::boxed_owned(
                        value,
                        EndpointOwnedPointer::new(EndpointOwnedPointerAddress::new(new_address.bytes)),
                        mutability
                    )
                }).collect::<Vec<_>>();
                Ok(owned_values)
            }
            _ => Err(ExecutionError::InvalidProgram(InvalidProgramError::ExpectedValue))
        }
    }

    /// Adds a pointer that is approved for move to a specific endpoint
    /// Returns an error if any moving shared container is not an owned pointer
    pub(crate) fn add_moving_pointers(
        &self,
        new_owner: Endpoint,
        moving_pointers: Vec<SharedContainer>,
    ) -> Result<(), ()> {
        let pointers = moving_pointers
            .into_iter()
            .map(|pointer| {
                let address = EndpointOwnedPointerAddress::new(pointer.try_get_owned_local_address().ok_or(())?);
                Ok((address, pointer))
            })
            .collect::<Result<Vec<(EndpointOwnedPointerAddress, SharedContainer)>, ()>>()?;

        self.moving_pointers.borrow_mut()
            .entry(new_owner)
            .or_insert_with(HashMap::new)
            .extend(pointers);

        Ok(())
    }

    pub(crate) fn handle_pointer_move_to_remote(
        self: Rc<RuntimeInternal>,
        from_endpoint: &Endpoint,
        pointer_mapping: Vec<(RawLocalPointerAddress, RawLocalPointerAddress)>,
    ) -> Result<Vec<ValueContainer>, ExecutionError> {
        let mut pointer_borrow = self.moving_pointers.borrow_mut();
        let moving_pointers = pointer_borrow.get_mut(from_endpoint).ok_or(ExecutionError::UnauthorizedMove)?;

        let values = pointer_mapping.into_iter().map(|(original, new)| {
            let original_address = EndpointOwnedPointerAddress::new(original.bytes);
            let new_address = ExternalPointerAddress::remote_for_endpoint(from_endpoint, new.bytes);
            let shared_container = moving_pointers.remove(&original_address).ok_or(ExecutionError::UnauthorizedMove)?;

            let value = shared_container.value_container();
            shared_container.move_to_remote(new_address).map_err(|_| ExecutionError::UnauthorizedMove)?;
            Ok(value)
        })
            .collect::<Result<Vec<_>, ExecutionError>>()?;
        Ok(values)
    }

    pub fn get_env(&self) -> HashMap<String, String> {
        self.config.env.clone().unwrap_or_default()
    }
}
