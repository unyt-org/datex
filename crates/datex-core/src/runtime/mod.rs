//! This module contains the implementation of the runtime, which is the backbone of the DATEX system.
use crate::{
    network::com_hub::ComHub,
    runtime::execution::ExecutionError,
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};

use crate::prelude::*;
use core::{cell::RefCell, fmt::Debug, result::Result};
use execution::context::{
    ExecutionContext, RemoteExecutionContext, ScriptExecutionError,
};

mod config;
// pub mod dif_interface;
pub mod execution;
mod incoming_sections;
mod internal;
pub mod memory;
mod runner;

pub mod pointer_address_provider;
mod request_move;
#[cfg(test)]
pub mod test_utils;

use self::memory::Memory;
use crate::runtime::pointer_address_provider::SelfOwnedPointerAddressProvider;
pub use config::*;
pub use internal::*;
pub use runner::*;

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

/// publicly exposed wrapper impl for the Runtime
/// around RuntimeInternal
impl Runtime {
    pub(crate) fn new(runtime_internal: RuntimeInternal) -> Runtime {
        Runtime {
            version: VERSION.to_string(),
            internal: Rc::new(runtime_internal),
        }
    }

    pub fn stub() -> Runtime {
        Runtime::new(RuntimeInternal::stub())
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

    pub fn pointer_address_provider(
        &self,
    ) -> &RefCell<SelfOwnedPointerAddressProvider> {
        &self.internal.pointer_address_provider
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
