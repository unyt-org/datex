use crate::{
    channel::mpsc::{UnboundedReceiver, create_unbounded_channel},
    collections::HashMap,
    core_compiler::core_compilation_context::DXBWithSharedValues,
    dif::dif_interface::DIFInterface,
    disassembler::{disassemble_body_to_string, options::DisassemblerOptions},
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
    libs::core::CoreLibrary,
    network::{
        com_hub::{
            ComHub, InterfacePriority, network_response::ResponseOptions,
        },
        com_interfaces::local_loopback_interface::LocalLoopbackInterfaceSetupData,
    },
    prelude::*,
    runtime::{
        Runtime, RuntimeConfig, RuntimeConfigInterface,
        cache::shared_references_cache::SharedReferencesCache,
        execution::{
            ExecutionError, InvalidProgramError,
            context::{
                ExecutionContext, ExecutionMode, RemoteExecutionContext,
                ScriptExecutionError,
            },
            execution_input::ExecutionCallerMetadata,
        },
        pointer_address_provider::SelfOwnedPointerAddressProvider,
        pointer_availability_lookup::PointerAvailabilityLookup,
        request_move::compile_request_move,
    },
    shared_values::{
        OwnedSharedContainer, PointerAddress, RemotePointerAddress,
        SelfOwnedPointerAddress, SharedContainer, SharedContainerMutability,
        base_shared_value_container::observers::TransceiverId,
    },
    time::Instant,
    utils::task_manager::TaskManager,
    values::{
        core_value::CoreValue, core_values::endpoint::Endpoint, value::Value,
        value_container::ValueContainer,
    },
};
use alloc::rc::Rc;
use core::{
    cell::{Ref, RefCell, RefMut},
    pin::Pin,
    slice,
};
use log::{debug, error, info};

#[derive(Debug)]
pub struct RuntimeInternal {
    version: String,
    endpoint: Endpoint,

    core_library: CoreLibrary,

    memory: RefCell<SharedReferencesCache>,
    pointer_address_provider: Rc<RefCell<SelfOwnedPointerAddressProvider>>,
    com_hub: Rc<ComHub>,
    config: RuntimeConfig,

    /// counter to keep track of transceiver ids
    transceiver_counter: RefCell<u32>,

    task_manager: TaskManager,

    // receiver for incoming sections from com hub
    incoming_sections_receiver: RefCell<UnboundedReceiver<IncomingSection>>,

    /// active execution contexts, stored by context_id
    execution_contexts:
        RefCell<HashMap<IncomingEndpointContextSectionId, ExecutionContext>>,

    /// list of currently owned shared values that were approved to be transfered to another endpoint
    moving_pointers: RefCell<
        HashMap<
            Endpoint,
            HashMap<SelfOwnedPointerAddress, OwnedSharedContainer>,
        >,
    >,

    pointer_availability_lookup: RefCell<PointerAvailabilityLookup>,
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

impl From<Rc<RuntimeInternal>> for Runtime {
    fn from(value: Rc<RuntimeInternal>) -> Self {
        Runtime { internal: value }
    }
}

impl RuntimeInternal {
    pub(crate) fn new(
        endpoint: Endpoint,
        memory: RefCell<SharedReferencesCache>,
        pointer_address_provider: Rc<RefCell<SelfOwnedPointerAddressProvider>>,
        config: RuntimeConfig,
        com_hub: Rc<ComHub>,
        task_manager: TaskManager,
        incoming_sections_receiver: UnboundedReceiver<IncomingSection>,
    ) -> RuntimeInternal {
        let pointer_availability_lookup =
            PointerAvailabilityLookup::new(endpoint.clone());
        RuntimeInternal {
            version: env!("CARGO_PKG_VERSION").to_string(),
            endpoint,
            memory,
            pointer_address_provider,
            config,
            com_hub,
            task_manager,
            core_library: CoreLibrary::default(),
            incoming_sections_receiver: RefCell::new(
                incoming_sections_receiver,
            ),
            execution_contexts: RefCell::new(HashMap::new()),
            moving_pointers: RefCell::new(HashMap::new()),
            transceiver_counter: RefCell::new(0),
            pointer_availability_lookup: RefCell::new(
                pointer_availability_lookup,
            ),
        }
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
    pub fn com_hub(&self) -> Rc<ComHub> {
        self.com_hub.clone()
    }
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
    pub fn pointer_availability_lookup(
        &self,
    ) -> Ref<'_, PointerAvailabilityLookup> {
        self.pointer_availability_lookup.borrow()
    }
    pub fn pointer_availability_lookup_mut(
        &self,
    ) -> RefMut<'_, PointerAvailabilityLookup> {
        self.pointer_availability_lookup.borrow_mut()
    }
    pub fn memory(&self) -> &RefCell<SharedReferencesCache> {
        &self.memory
    }
    pub fn core_library(&self) -> &CoreLibrary {
        &self.core_library
    }

    pub fn pointer_address_provider(
        &self,
    ) -> &RefCell<SelfOwnedPointerAddressProvider> {
        &self.pointer_address_provider
    }
    pub fn incoming_sections_receiver_mut(
        &self,
    ) -> RefMut<'_, UnboundedReceiver<IncomingSection>> {
        self.incoming_sections_receiver.borrow_mut()
    }
    pub fn incoming_sections_receiver(
        &self,
    ) -> Ref<'_, UnboundedReceiver<IncomingSection>> {
        self.incoming_sections_receiver.borrow()
    }
    pub fn task_manager(&self) -> &TaskManager {
        &self.task_manager
    }

    pub fn stub() -> RuntimeInternal {
        let (sender, receiver) = create_unbounded_channel();
        RuntimeInternal::new(
            Endpoint::default(),
            RefCell::new(SharedReferencesCache::default()),
            Rc::new(RefCell::new(SelfOwnedPointerAddressProvider::new(
                Endpoint::default(),
            ))),
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
                config,
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
                            "Failed to create interface \"{interface_type}\": {err}"
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
        let execution_context = get_execution_context!(
            Runtime::from(self.clone()),
            execution_context
        );
        let compile_start = Instant::now();
        let dxb = execution_context.compile(
            script,
            inserted_values,
            execution_context.receivers(),
        )?;
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
        let execution_context = get_execution_context!(
            Runtime::from(self.clone()),
            execution_context
        );
        let compile_start = Instant::now();
        let dxb = execution_context.compile(
            script,
            inserted_values,
            execution_context.receivers(),
        )?;
        debug!(
            "[Compilation took {} ms]",
            compile_start.elapsed().as_millis()
        );
        let execute_start = Instant::now();
        let result = RuntimeInternal::execute_dxb_sync(
            self,
            dxb,
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
        input: DXBWithSharedValues,
        execution_context: Option<&'a mut ExecutionContext>,
        _end_execution: bool,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<Option<ValueContainer>, ExecutionError>>
                + 'a,
        >,
    > {
        Box::pin(async move {
            let execution_context = get_execution_context!(
                Runtime::from(self.clone()),
                execution_context
            );
            match execution_context {
                ExecutionContext::Remote(context) => {
                    RuntimeInternal::execute_remote(self, context, input).await
                }
                ExecutionContext::Local(_) => {
                    execution_context.execute_dxb(input).await
                }
            }
        })
    }

    pub fn execute_dxb_sync(
        self: Rc<RuntimeInternal>,
        dxb: DXBWithSharedValues,
        execution_context: Option<&mut ExecutionContext>,
        _end_execution: bool,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let execution_context =
            get_execution_context!(Runtime::from(self), execution_context);
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
        let runtime = Runtime::from(self.clone());
        let mut execution_contexts = self.execution_contexts.borrow_mut();
        // get execution context by context_id or create a new one if it doesn't exist
        let execution_context = execution_contexts.remove(context_id);
        if let Some(context) = execution_context {
            context
        } else {
            let caller_metadata = ExecutionCallerMetadata {
                endpoint: incoming_section.get_sender(),
            };
            ExecutionContext::local(
                ExecutionMode::unbounded(),
                runtime,
                caller_metadata,
            )
        }
    }

    pub async fn execute_remote(
        self: Rc<RuntimeInternal>,
        remote_execution_context: &mut RemoteExecutionContext,
        input: DXBWithSharedValues,
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

        let mut block = DXBBlock::new(
            routing_header,
            block_header,
            encrypted_header,
            input.dxb,
        );

        block
            .set_receivers(slice::from_ref(&remote_execution_context.endpoint));

        let response = self
            .com_hub
            .send_own_block_await_response(block, ResponseOptions::default())
            .await
            .remove(0)?;
        let incoming_section = response.take_incoming_section();

        RuntimeInternal::execute_incoming_section(
            self,
            incoming_section,
            Some(input.shared_values),
        )
        .await
        .0
    }

    pub(crate) async fn execute_incoming_section(
        self: Rc<RuntimeInternal>,
        mut incoming_section: IncomingSection,
        shared_values: Option<Vec<SharedContainer>>,
    ) -> (
        Result<Option<ValueContainer>, ExecutionError>,
        Endpoint,
        OutgoingContextId,
    ) {
        let section_context_id =
            incoming_section.get_section_context_id().clone();
        let mut context = Self::take_execution_context(
            self.clone(),
            &section_context_id,
            &incoming_section,
        );
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
                    // NOTE: this assumes that all shared values are only needed as references, not as owned values
                    shared_values.clone(),
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
        self.execution_contexts
            .borrow_mut()
            .insert(section_context_id, context);

        (Ok(result), sender_endpoint, context_id)
    }

    async fn execute_dxb_block_local(
        self: Rc<RuntimeInternal>,
        block: DXBBlock,
        execution_context: Option<&mut ExecutionContext>,
        shared_values: Option<Vec<SharedContainer>>,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let execution_context = get_execution_context!(
            Runtime::from(self.clone()),
            execution_context
        );
        // assert that the execution context is local
        if !core::matches!(execution_context, ExecutionContext::Local(_)) {
            unreachable!(
                "Execution context must be local for executing a DXB block"
            );
        }
        let dxb = block.body;
        info!(
            "executing on {}:\n{}",
            self.endpoint,
            disassemble_body_to_string(&dxb, DisassemblerOptions::default())
        );
        let end_execution =
            block.block_header.flags_and_timestamp.is_end_of_section();

        RuntimeInternal::execute_dxb(
            self,
            DXBWithSharedValues::new(dxb, shared_values.unwrap_or_default()),
            Some(execution_context),
            end_execution,
        )
        .await
    }

    /// Registers a list of shared containers for a single endpoint.
    pub fn register_shared_containers_for_single_endpoint(
        &self,
        endpoint: &Endpoint,
        shared_containers: Vec<SharedContainer>,
    ) {
        unsafe {
            self.register_shared_containers_for_endpoints(
                &[endpoint],
                shared_containers,
            )
            .unwrap()
        }
    }

    /// Registers a list of shared containers for a list of endpoints.
    /// The caller must ensure that endpoints is not empty.
    pub unsafe fn register_shared_containers_for_endpoints(
        &self,
        endpoints: &[&Endpoint],
        shared_containers: Vec<SharedContainer>,
    ) -> Result<(), ()> {
        if endpoints.is_empty() {
            panic!("endpoints must not be empty");
        }

        for shared_container in shared_containers {
            match shared_container {
                SharedContainer::Owned(owned_shared_container) => {
                    if endpoints.len() == 1 {
                        self.add_moving_shared_container(
                            endpoints[0].clone(),
                            owned_shared_container,
                        );
                    } else {
                        return Err(());
                    }
                }
                SharedContainer::Referenced(_referenced_shared_container) => {
                    // TODO: Subscribe
                }
            }
        }

        Ok(())
    }

    /// Request to move a list of external pointers from an endpoint to the local endpoint
    /// This only works if the local endpoint has the permission to move the pointers, either because
    /// it was allowed via a PERFORM_MOVE from the remote endpoint, or because the local endpoint has
    /// extended permissions
    pub(crate) async fn request_pointer_move(
        self: Rc<RuntimeInternal>,
        from_endpoint: &Endpoint,
        pointers: Vec<(SharedContainerMutability, SelfOwnedPointerAddress)>,
    ) -> Result<Vec<OwnedSharedContainer>, ExecutionError> {
        let pointer_mapping = pointers
            .into_iter()
            .map(|original| {
                (
                    original,
                    self.pointer_address_provider
                        .borrow_mut()
                        .get_new_self_owned_address(),
                )
            })
            .collect::<Vec<_>>();
        let body = compile_request_move(
            pointer_mapping
                .iter()
                .map(|((_, original), new)| (original.clone(), new.clone()))
                .collect::<Vec<_>>(),
        );
        let moved_values = self
            .clone()
            .execute_dxb(
                DXBWithSharedValues::new(body, vec![]),
                Some(&mut ExecutionContext::Remote(
                    RemoteExecutionContext::new(
                        from_endpoint.clone(),
                        ExecutionMode::Static,
                        Runtime::from(self),
                    ),
                )),
                true,
            )
            .await?;

        // moved values should be list
        match moved_values {
            Some(ValueContainer::Local(Value {
                inner: CoreValue::List(list),
                ..
            })) => {
                let pointer_values = list.into_vec();
                let owned_values = pointer_values.into_iter()
                    .zip(pointer_mapping)
                    .map(|(value, ((mutability, _), new_address))| {
                        // SAFETY: we got the new address from the pointer address provider above, is ensured to be unique
                        unsafe {
                            // TODO: also pass type information
                            OwnedSharedContainer::new_with_inferred_allowed_type_unsafe(
                                value,
                                mutability,
                                new_address,
                            )
                        }
                }).collect::<Vec<_>>();
                Ok(owned_values)
            }
            _ => Err(ExecutionError::InvalidProgram(
                InvalidProgramError::ExpectedValue,
            )),
        }
    }

    /// Adds a pointer that is approved for move to a specific endpoint
    /// Returns an error if any moving shared container is not an owned pointer
    pub(crate) fn add_moving_shared_container(
        &self,
        new_owner: Endpoint,
        moving_owned_container: OwnedSharedContainer,
    ) {
        let address = moving_owned_container.pointer_address().clone();

        self.moving_pointers
            .borrow_mut()
            .entry(new_owner)
            .or_default()
            .insert(address, moving_owned_container);
    }

    pub(crate) fn handle_pointer_move_to_remote(
        self: Rc<RuntimeInternal>,
        from_endpoint: &Endpoint,
        pointer_mapping: Vec<(
            SelfOwnedPointerAddress,
            SelfOwnedPointerAddress,
        )>,
        memory: &SharedReferencesCache,
    ) -> Result<Vec<ValueContainer>, ExecutionError> {
        let mut pointer_borrow = self.moving_pointers.borrow_mut();
        let moving_pointers = pointer_borrow
            .get_mut(from_endpoint)
            .ok_or(ExecutionError::UnauthorizedMove)?;

        let values = pointer_mapping
            .into_iter()
            .map(|(original_address, new)| {
                let new_address =
                    RemotePointerAddress::for_endpoint(from_endpoint, &new);
                let shared_container = moving_pointers
                    .remove(&original_address)
                    .ok_or(ExecutionError::UnauthorizedMove)?;

                let value = shared_container.value_container().clone();

                // make sure external pointer does not already exist in memory
                if memory
                    .has_reference(&PointerAddress::Remote(new_address.clone()))
                {
                    return Err(ExecutionError::InvalidMove);
                } else {
                    // Note: safe because we checked before if address is already in memory
                    unsafe {
                        shared_container.move_to_remote(new_address, memory);
                    }
                }

                Ok(value)
            })
            .collect::<Result<Vec<_>, ExecutionError>>()?;
        Ok(values)
    }

    pub fn get_env(&self) -> HashMap<String, String> {
        self.config.env.clone().unwrap_or_default()
    }

    /// Creates a new [DIFInterface] with a unique transceiver id and the runtime's pointer address provider
    pub fn create_dif_interface(&self) -> DIFInterface {
        let id = TransceiverId(*self.transceiver_counter.borrow());
        self.transceiver_counter.replace(id.0 + 1);
        DIFInterface::new(id, self.pointer_address_provider.clone())
    }
}
