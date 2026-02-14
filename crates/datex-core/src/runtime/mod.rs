use crate::{
    network::{
        com_hub::{
            ComHub, InterfacePriority,
        },
        com_interfaces::com_interface::factory::{
            ComInterfaceAsyncFactory, ComInterfaceSyncFactory,
        },
    },
    runtime::execution::{ExecutionError, context::ExecutionMode},
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};

use crate::prelude::*;
use core::{
    cell::RefCell, fmt::Debug, result::Result,
};
use execution::context::{
    ExecutionContext, RemoteExecutionContext, ScriptExecutionError,
};
use futures::{FutureExt};
use serde::{Deserialize, Serialize};

pub mod dif_interface;
pub mod execution;
mod incoming_sections;
pub mod memory;
mod runner;
mod config;
mod internal;

pub use runner::*;
pub use config::*;
pub use internal::*;

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

use crate::{
    network::{
        com_hub::is_none_variant,
        com_interfaces::local_loopback_interface::LocalLoopbackInterfaceSetupData,
    },
};


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
