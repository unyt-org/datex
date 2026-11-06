use crate::{
    core_compiler::instruction_input::InstructionInput,
    instruction::regular_instruction::RegularInstruction,
    prelude::*,
    runtime::{Runtime, execution::ExecutionError},
    shared_values::collapsed_container_value::CollapsedContainerValue,
    traits::{
        apply::{Apply, ApplyArgument},
        classification::Classification,
    },
    types::{
        entities::entity_impls::EntityImplMethod, entity_type::EntityType,
    },
    values::{
        core_values::endpoint::Endpoint, value_container::ValueContainer,
    },
};
use core::{
    cell::Ref,
    ops::{Deref, DerefMut},
};
use log::info;

// TODO: move somewhere else, this is an impl of ApplyArgument
impl ApplyArgument {
    /// Tries to call a method on the value container. If the method is not found, returns an error.
    /// This currently resolves the method by name (FIXME)
    pub fn try_call_method_sync(
        self,
        method_name: &str,
        mut args: Vec<ApplyArgument>,
        runtime: &Runtime,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ExecutionError>
    {
        let val = self.value.collapsed_value();
        let entity_type = Self::try_get_entity_type(val, runtime)?;
        let method = Self::try_get_method_data(&entity_type, method_name)?;

        let owner_endpoint = self.value.owner();

        // prepend callee to args
        args.insert(0, self);

        // only local calls supported for sync execution
        if !method.call_on_owner
            || owner_endpoint.is_local_or_equals_endpoint(runtime.endpoint())
        {
            let res = method.callable.try_apply_sync_checked(runtime, args)?;
            Ok(res)
        } else {
            Err(ExecutionError::RequiresAsyncExecution)
        }
    }

    pub async fn try_call_method_async(
        self,
        method_name: &str,
        mut args: Vec<ApplyArgument>,
        runtime: &Runtime,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ExecutionError>
    {
        let val = self.value.collapsed_value();
        let entity_type = Self::try_get_entity_type(val, runtime)?;
        let method = Self::try_get_method_data(&entity_type, method_name);

        match method {
            Ok(method) => {
                self.try_call_method_async_direct(method.deref(), args, runtime)
                    .await
            }
            // FIXME: only temp test, method should always be declared in the type
            Err(_) => {
                info!("fallback to remote call for method: {}", method_name);
                let owner = self.value.owner();
                // append callee to args
                args.push(self);

                Self::try_call_method_on_owner(
                    owner,
                    method_name.to_string(),
                    args,
                    runtime,
                )
                .await
            }
        }
    }

    /// Tries to call a method on the value container asynchronously. If the method is not found, returns an error.
    /// This currently resolves the method by name (FIXME)
    async fn try_call_method_async_direct<'a>(
        self,
        method: &'a EntityImplMethod,
        mut args: Vec<ApplyArgument>,
        runtime: &'a Runtime,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ExecutionError>
    {
        let owner_endpoint = self.value.owner();

        // call locally
        if !method.call_on_owner
            || owner_endpoint.is_local_or_equals_endpoint(runtime.endpoint())
        {
            // prepend callee to args
            args.insert(0, self);

            let res = method
                .callable
                .try_apply_async_checked(runtime, args)
                .await?;
            Ok(res)
        }
        // call on owner endpoint
        else {
            // append callee to args
            args.push(self);
            Self::try_call_method_on_owner(
                owner_endpoint,
                method.name().unwrap().clone(),
                args,
                runtime,
            )
            .await
        }
    }

    async fn try_call_method_on_owner(
        owner_endpoint: Endpoint,
        name: String,
        args: Vec<ApplyArgument>,
        runtime: &Runtime,
    ) -> Result<(Option<ValueContainer>, Vec<ValueContainer>), ExecutionError>
    {
        let mut instructions = vec![
            RegularInstruction::call_method(
                name, // FIXME: don't use method name here
                args.len() as u8 - 1,
            )
            .into(),
        ];
        instructions.extend(
            args.into_iter()
                .map(|v| InstructionInput::ValueContainer(v.value)),
        );

        let res = runtime
            .execute_instructions_remote_single_receiver(
                owner_endpoint,
                instructions,
            )
            .await?;

        // FIXME: restore borrowed stack values across remote execution.
        // For now, local borrows are not supported cross endpoint
        Ok((res, vec![]))
    }

    /// Tries to get the entity type of the [CollapsedContainerValue]. If the value is not an entity, returns an error.
    fn try_get_entity_type(
        value: CollapsedContainerValue,
        runtime: &Runtime,
    ) -> Result<EntityType, ExecutionError> {
        if let Some(entity_type) = value
            .borrow()
            .deref()
            .entity_type(runtime.shared_references_cache_mut().deref_mut())
        {
            Ok(entity_type)
        } else {
            Err(ExecutionError::ExpectedEntityValue)
        }
    }

    fn try_get_method_data<'a>(
        entity_type: &'a EntityType,
        method_name: &'a str,
    ) -> Result<Ref<'a, EntityImplMethod>, ExecutionError> {
        Ref::filter_map(entity_type.entity_definition(), |def| {
            def.try_get_method(method_name)
        })
        .map_err(|_| ExecutionError::MethodNotFound(method_name.to_string()))
    }
}
