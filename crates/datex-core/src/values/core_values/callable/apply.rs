use crate::{
    core_compiler::core_compilation_context::DXBWithSharedValues,
    prelude::*,
    runtime::{
        Runtime,
        execution::{
            context::{ExecutionContext, ExecutionMode, LocalExecutionContext},
            execution_input::ExecutionCallerMetadata,
        },
    },
    traits::apply::{Apply, ApplyError},
    values::{
        core_values::callable::{
            Callable, CallableBody, DatexBytecodeCallable, NativeCallable,
            error::CallableError,
        },
        value_container::ValueContainer,
    },
};
use crate::traits::apply::ApplyArgument;

impl Apply for Callable {
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        match &self.body {
            CallableBody::Native(native_callable) => {
                native_callable.try_apply_sync(runtime, args)
            }
            CallableBody::DatexBytecode(bytecode_callable) => {
                bytecode_callable.try_apply_sync(runtime, args)
            }
            CallableBody::CoreStub(_stub) => {
                Err(CallableError::RuntimeOnlyCallable.into())
            }
            CallableBody::Hidden => Err(CallableError::HiddenCallable.into()),
        }
    }

    async fn try_apply_async(
        &self,
        runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        match &self.body {
            CallableBody::Native(native_callable) => {
                native_callable.try_apply_async(runtime, args).await
            }
            CallableBody::DatexBytecode(bytecode_callable) => {
                bytecode_callable.try_apply_async(runtime, args).await
            }
            CallableBody::CoreStub(_stub) => {
                Err(CallableError::RuntimeOnlyCallable.into())
            }
            CallableBody::Hidden => Err(CallableError::HiddenCallable.into()),
        }
    }
}

impl Apply for NativeCallable {
    fn try_apply_sync(
        &self,
        _runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        match self {
            NativeCallable::Sync(f) => f(args).map_err(|e| e.into()),
            NativeCallable::Async(_f) => {
                Err(ApplyError::AsyncCallableRequiresAsyncExecution)
            }
        }
    }

    async fn try_apply_async(
        &self,
        _runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        match self {
            NativeCallable::Sync(f) => f(args).map_err(|e| e.into()),
            NativeCallable::Async(f) => f(args).await.map_err(|e| e.into()),
        }
    }
}

impl Apply for DatexBytecodeCallable {
    fn try_apply_sync(
        &self,
        runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        if self.requires_async {
            return Err(ApplyError::AsyncCallableRequiresAsyncExecution);
        }

        // construct the initial stack values by combining the provided arguments with the injected values
        let stack_values = args
            .into_iter()
            .map(|v| v.value)
            .chain(self.injected_values.iter().cloned())
            .collect::<Vec<_>>();

        let res = runtime
            .execute_dxb_sync(
                DXBWithSharedValues::new(self.body.clone(), vec![]), // TODO: no clone?
                Some(stack_values),
                Some(&mut ExecutionContext::Local(LocalExecutionContext::new(
                    ExecutionMode::Static,
                    runtime.clone(),
                    ExecutionCallerMetadata::local_default(), // TODO caller
                ))),
                true,
            )
            .map_err(CallableError::from)?;

        // TODO: restore borrowed stack values from execution and return
        Ok((res, vec![]))
    }

    async fn try_apply_async(
        &self,
        runtime: &Runtime,
        args: Vec<ApplyArgument>,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ApplyError> {
        // construct the initial stack values by combining the provided arguments with the injected values
        let stack_values = args
            .into_iter()
            .map(|v| v.value)
            .chain(self.injected_values.iter().cloned())
            .collect::<Vec<_>>();
        
        let res = runtime
            .execute_dxb(
                DXBWithSharedValues::new(self.body.clone(), vec![]), // TODO: no clone?
                Some(stack_values),
                Some(&mut ExecutionContext::Local(LocalExecutionContext::new(
                    ExecutionMode::Static,
                    runtime.clone(),
                    ExecutionCallerMetadata::local_default(), // TODO caller
                ))),
                true,
            )
            .await
            .map_err(CallableError::from)?;

        // TODO: restore borrowed stack values from execution and return
        Ok((res, vec![]))
    }
}
