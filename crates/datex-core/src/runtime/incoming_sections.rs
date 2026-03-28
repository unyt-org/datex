use crate::{
    core_compiler::value_compiler::compile_value_container,
    global::{
        dxb_block::{DXBBlock, IncomingSection, OutgoingContextId},
        protocol_structures::{
            block_header::{BlockHeader, BlockType, FlagsAndTimestamp},
            encrypted_header::EncryptedHeader,
            routing_header::RoutingHeader,
        },
    },
    runtime::{RuntimeInternal, execution::ExecutionError},
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};

use crate::prelude::*;
use core::result::Result;
use log::info;
use crate::core_compiler::value_compiler::{compile_shared_container, compile_value};
use crate::decompiler::DecompileOptions;

impl RuntimeInternal {
    pub(crate) async fn handle_incoming_sections_task(
        self: Rc<RuntimeInternal>,
    ) {
        let mut sections_receiver =
            self.incoming_sections_receiver.borrow_mut();

        while let Some(section) = sections_receiver.next().await {
            let self_clone = self.clone();
            // for embassy, run all sections in the same task to avoid spawning too many tasks
            #[cfg(feature = "embassy_runtime")]
            {
                self_clone.handle_incoming_section_task(section).await;
            }
            // otherwise, run each section in its own task
            #[cfg(not(feature = "embassy_runtime"))]
            {
                // TODO #741: task
                self.task_manager.register_task(
                    self_clone.handle_incoming_section_task(section),
                );
            }
        }
    }
    async fn handle_incoming_section_task(
        self: Rc<RuntimeInternal>,
        section: IncomingSection,
    ) {
        let (result, endpoint, context_id) =
            RuntimeInternal::execute_incoming_section(self.clone(), section)
                .await;
        match &result {
            Ok(Some(result)) => info!(
                "Successful Execution result (on {} from {}): {}",
                self.endpoint, endpoint,
                {
                    #[cfg(feature = "decompiler")]
                    {
                        crate::decompiler::decompile_value(result, DecompileOptions::colorized())
                    }
                    #[cfg(not(feature = "decompiler"))]
                    {
                        result
                    }
                }
            ),
            Ok(None) => info!(
                "Successful Execution result (on {} from {}): None",
                self.endpoint, endpoint
            ),
            Err(e) => info!(
                "Execution error (on {} from {}): {e}",
                self.endpoint, endpoint
            ),
        }

        // send response back to the sender
        let _res = RuntimeInternal::send_response_block(
            self.clone(),
            result,
            endpoint,
            context_id,
        )
        .await;
        // TODO #231: handle errors in sending response
    }

    async fn send_response_block(
        self: Rc<RuntimeInternal>,
        result: Result<Option<ValueContainer>, ExecutionError>,
        receiver_endpoint: Endpoint,
        context_id: OutgoingContextId,
    ) -> Result<(), Vec<Endpoint>> {
        let routing_header: RoutingHeader = RoutingHeader::default()
            .with_sender(self.endpoint.clone())
            .to_owned();
        let block_header = BlockHeader {
            context_id,
            flags_and_timestamp: FlagsAndTimestamp::new()
                .with_block_type(BlockType::Response)
                .with_is_end_of_section(true)
                .with_is_end_of_context(true),
            ..BlockHeader::default()
        };
        let encrypted_header = EncryptedHeader::default();

        info!(
            "send response, context_id: {context_id:?}, receiver: {receiver_endpoint}"
        );

        if let Ok(value) = result {
            let dxb = if let Some(value) = value {
                match value {
                    ValueContainer::Shared(shared_container) => {
                        let compiled = compile_shared_container(&shared_container, true);
                        if shared_container.is_owned() {
                            self.add_moving_pointers(receiver_endpoint.clone(), vec![shared_container]).unwrap();
                        }
                        compiled.unwrap()
                    },
                    ValueContainer::Local(value) => compile_value(&value).unwrap(),
                }
            } else {
                vec![]
            };

            let mut block = DXBBlock::new(
                routing_header,
                block_header,
                encrypted_header,
                dxb,
            );
            block.set_receivers(core::slice::from_ref(&receiver_endpoint));

            self.com_hub.send_own_block_async(block).await
        } else {
            core::todo!("#233 Handle returning error response block");
        }
    }
}
