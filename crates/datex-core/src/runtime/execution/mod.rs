use crate::{
    runtime::{
        RuntimeInternal,
        execution::{
            context::{ExecutionMode, RemoteExecutionContext},
            execution_loop::interrupts::{
                ExternalExecutionInterrupt, InterruptResult,
            },
        },
    },
    traits::apply::Apply,
    values::value_container::ValueContainer,
};

use crate::{
    global::protocol_structures::instruction_data::{
        RawBuiltinPointerAddress, RawLocalPointerAddress,
        RawRemotePointerAddress,
    },
    libs::core::core_lib_id::CoreLibId,
    prelude::*,
    shared_values::{
        pointer_address::{ExternalPointerAddress, PointerAddress},
        shared_containers::{
            ReferenceMutability, ReferencedSharedContainer, SharedContainer,
        },
    },
};
use core::{result::Result, unreachable};
pub use errors::*;
pub use execution_input::{ExecutionInput, ExecutionOptions};
use log::info;
pub use stack_dump::*;
use crate::values::core_values::endpoint::Endpoint;

pub mod context;
mod errors;
pub mod execution_input;
pub mod execution_loop;
pub mod macros;
mod stack_dump;

#[cfg(all(test, feature = "std"))]
mod test_remote_execution;

pub fn execute_dxb_sync(
    input: ExecutionInput,
) -> Result<Option<ValueContainer>, ExecutionError> {
    let runtime = input.runtime.clone();
    let (interrupt_provider, execution_loop) = input.execution_loop();

    for output in execution_loop {
        match output? {
            ExternalExecutionInterrupt::Result(result) => return Ok(result),
            ExternalExecutionInterrupt::GetReferenceToRemotePointer(
                address,
                mutability,
            ) => interrupt_provider.provide_result(
                InterruptResult::ResolvedValue(
                    get_remote_shared_container_reference(
                        &runtime.internal,
                        address,
                        mutability,
                    )?
                    .map(|v| {
                        ValueContainer::Shared(SharedContainer::Referenced(v))
                    }),
                ),
            ),
            ExternalExecutionInterrupt::GetReferenceToLocalPointer(address) => {
                // TODO #401: in the future, local pointer addresses should be relative to the block sender, not the local runtime
                interrupt_provider.provide_result(
                    InterruptResult::ResolvedValue(
                        get_local_pointer_value(&runtime.internal, address)
                            .map(|v| {
                                ValueContainer::Shared(
                                    SharedContainer::Referenced(v),
                                )
                            }),
                    ),
                );
            }
            ExternalExecutionInterrupt::GetReferenceToBuiltinPointer(
                address,
            ) => {
                interrupt_provider.provide_result(
                    InterruptResult::ResolvedValue(Some(
                        ValueContainer::Shared(SharedContainer::Referenced(
                            get_builtin_shared_value_reference(
                                &runtime.internal,
                                address,
                            )?,
                        )),
                    )),
                );
            }
            ExternalExecutionInterrupt::Apply(callee, args) => {
                let res = handle_apply(&callee, &args)?;
                interrupt_provider
                    .provide_result(InterruptResult::ResolvedValue(res));
            }
            _ => return Err(ExecutionError::RequiresAsyncExecution),
        }
    }

    Err(ExecutionError::RequiresAsyncExecution)
}

pub async fn execute_dxb(
    input: ExecutionInput<'_>,
) -> Result<Option<ValueContainer>, ExecutionError> {
    let runtime = input.runtime.clone();
    let caller_metadata = input.caller_metadata.clone();
    let (interrupt_provider, execution_loop) = input.execution_loop();

    for output in execution_loop {
        match output? {
            ExternalExecutionInterrupt::Result(result) => return Ok(result),
            ExternalExecutionInterrupt::GetReferenceToRemotePointer(
                address,
                mutability,
            ) => {
                interrupt_provider.provide_result(
                    InterruptResult::ResolvedValue(
                        get_remote_shared_container_reference(
                            &runtime.internal,
                            address,
                            mutability,
                        )?
                        .map(|v| {
                            ValueContainer::Shared(SharedContainer::Referenced(
                                v,
                            ))
                        }),
                    ),
                );
            }
            ExternalExecutionInterrupt::GetReferenceToLocalPointer(address) => {
                // TODO #402: in the future, local pointer addresses should be relative to the block sender, not the local runtime
                interrupt_provider.provide_result(
                    InterruptResult::ResolvedValue(
                        get_local_pointer_value(&runtime.internal, address)
                            .map(|v| {
                                ValueContainer::Shared(
                                    SharedContainer::Referenced(v),
                                )
                            }),
                    ),
                );
            }
            ExternalExecutionInterrupt::GetReferenceToBuiltinPointer(
                address,
            ) => {
                interrupt_provider.provide_result(
                    InterruptResult::ResolvedValue(Some(
                        ValueContainer::Shared(SharedContainer::Referenced(
                            get_builtin_shared_value_reference(
                                &runtime.internal,
                                address,
                            )?,
                        )),
                    )),
                );
            }
            ExternalExecutionInterrupt::RemoteExecution(receivers, body) => {
                // assert that receivers is a single endpoint
                // TODO #230: support advanced receivers
                let receiver_endpoint = receivers
                    .try_as::<Endpoint>()
                    .unwrap();
                let mut remote_execution_context = RemoteExecutionContext::new(
                    receiver_endpoint,
                    ExecutionMode::Static,
                    runtime.clone()
                );
                let res = RuntimeInternal::execute_remote(
                    runtime.internal.clone(),
                    &mut remote_execution_context,
                    body,
                )
                .await?;
                interrupt_provider
                    .provide_result(InterruptResult::ResolvedValue(res));
            }
            ExternalExecutionInterrupt::Apply(callee, args) => {
                let res = handle_apply(&callee, &args)?;
                interrupt_provider
                    .provide_result(InterruptResult::ResolvedValue(res));
            }
            ExternalExecutionInterrupt::RequestMove(pointers) => {
                let moved_values = runtime.internal
                    .clone()
                    .request_pointer_move(&caller_metadata.endpoint, pointers)
                    .await?
                    .into_iter()
                    .map(|v| ValueContainer::Shared(SharedContainer::Owned(v)))
                    .collect();
                interrupt_provider.provide_result(
                    InterruptResult::ResolvedValues(moved_values),
                );
            }
            ExternalExecutionInterrupt::Move(address_mapping) => {
                let moved_values =
                    runtime.internal.clone().handle_pointer_move_to_remote(
                        &caller_metadata.endpoint,
                        address_mapping,
                        &*runtime.memory().borrow(),
                    )?;
                interrupt_provider.provide_result(
                    InterruptResult::ResolvedValues(moved_values),
                );
            }
        }
    }

    unreachable!("Execution loop should always return a result");
}

fn handle_apply(
    callee: &ValueContainer,
    args: &[ValueContainer],
) -> Result<Option<ValueContainer>, ExecutionError> {
    // callee is guaranteed to be Some here
    // apply_single if one arg, apply otherwise
    Ok(if args.len() == 1 {
        callee.apply_single(&args[0])?
    } else {
        callee.apply(args)?
    })
}

fn get_remote_shared_container_reference(
    runtime_internal: &Rc<RuntimeInternal>,
    address: RawRemotePointerAddress,
    _mutability: ReferenceMutability,
) -> Result<Option<ReferencedSharedContainer>, ExecutionError> {
    let address_provider = runtime_internal.pointer_address_provider.borrow();
    let memory = runtime_internal.memory.borrow();
    let resolved_address =
        address_provider.get_pointer_address_from_raw_full_address(address);
    // convert slot to InternalSlot enum
    // TODO #770: resolve from remote, handle mutability
    Ok(memory.get_reference(&resolved_address).cloned())
}

fn get_builtin_shared_value_reference(
    runtime_internal: &Rc<RuntimeInternal>,
    address: RawBuiltinPointerAddress,
) -> Result<ReferencedSharedContainer, ExecutionError> {
    let pointer_address =
        PointerAddress::External(ExternalPointerAddress::Builtin(address.id));
    let memory = runtime_internal.memory.borrow();
    if let Some(reference) = memory.get_reference(&pointer_address) {
        Ok(reference.clone())
    } else {
        Err(ExecutionError::ReferenceNotFound)
    }
}

fn get_local_pointer_value(
    runtime_internal: &Rc<RuntimeInternal>,
    address: RawLocalPointerAddress,
) -> Option<ReferencedSharedContainer> {
    // convert slot to InternalSlot enum
    runtime_internal
        .memory
        .borrow()
        .get_reference(&PointerAddress::owned(address.bytes))
        .cloned()
}

#[cfg(test)]
#[cfg(feature = "compiler")]
mod tests {
    use super::*;
    use crate::{
        assert_structural_eq, assert_value_eq,
        compiler::{CompileOptions, compile_script, scope::CompilationScope},
        datex_list,
        global::instruction_codes::InstructionCode,
        runtime::{
            RuntimeConfig, RuntimeRunner,
            execution::{
                context::{ExecutionContext, LocalExecutionContext},
                execution_input::{ExecutionCallerMetadata, ExecutionOptions},
            },
        },
        shared_values::shared_containers::{
            OwnedSharedContainer, ReferencedSharedContainer, SharedContainer,
            SharedContainerInner, SharedContainerMutability,
            base_shared_value_container::BaseSharedValueContainer,
        },
        traits::{structural_eq::StructuralEq, value_eq::ValueEq},
        values::{
            core_value::CoreValue,
            core_values::{
                decimal::Decimal,
                integer::{Integer, typed_integer::TypedInteger},
                list::List,
                map::Map,
            },
        },
    };
    use core::assert_matches;
    use log::{debug, info};
    use crate::libs::core::type_id::CoreLibBaseTypeId;
    use crate::runtime::Runtime;

    fn execute_datex_script_debug(
        datex_script: &str,
    ) -> Option<ValueContainer> {
        let runtime = Runtime::stub();
        let (dxb, _) =
            compile_script(datex_script, CompileOptions::default(), runtime.clone()).unwrap();
        let context = ExecutionInput::new(
            &dxb,
            ExecutionCallerMetadata::local_default(),
            ExecutionOptions { verbose: true },
            runtime
        );
        execute_dxb_sync(context).unwrap()
    }

    fn execute_datex_script_debug_unbounded(
        datex_script_parts: impl Iterator<Item = &'static str>,
        runtime: Runtime,
    ) -> impl Iterator<Item = Result<Option<ValueContainer>, ExecutionError>>
    {
        gen move {

            let datex_script_parts = datex_script_parts.collect::<Vec<_>>();
            let mut execution_context =
                ExecutionContext::Local(LocalExecutionContext::new(
                    ExecutionMode::unbounded(),
                    runtime.clone(),
                    ExecutionCallerMetadata::local_default(),
                ));
            let mut compilation_scope =
                CompilationScope::new(ExecutionMode::unbounded());

            let len = datex_script_parts.len();
            for (index, script_part) in
                datex_script_parts.into_iter().enumerate()
            {
                // if last part, compile and return static value if possible
                if index == len - 1 {
                    compilation_scope.mark_as_last_execution();
                }

                let (dxb, new_compilation_scope) = compile_script(
                    script_part,
                    CompileOptions::new_with_scope(compilation_scope),
                    runtime.clone()
                )
                .unwrap();
                compilation_scope = new_compilation_scope;
                yield execution_context.execute_dxb_sync(&dxb)
            }
        }
    }

    fn assert_unbounded_input_matches_output(
        input: Vec<&'static str>,
        expected_output: Vec<Option<ValueContainer>>,
        runtime: Runtime,
    ) {
        let input = input.into_iter();
        let expected_output = expected_output.into_iter();
        for (result, expected) in
            execute_datex_script_debug_unbounded(input.into_iter(), runtime.clone())
                .zip(expected_output.into_iter())
        {
            let result = result.unwrap();
            assert_eq!(result, expected);
        }
    }

    fn execute_datex_script_debug_with_error(
        datex_script: &str,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let runtime = Runtime::stub();
        let (dxb, _) =
            compile_script(datex_script, CompileOptions::default(), runtime.clone()).unwrap();
        let context = ExecutionInput::new(
            &dxb,
            ExecutionCallerMetadata::local_default(),
            ExecutionOptions { verbose: true },
            runtime
        );
        execute_dxb_sync(context)
    }

    fn execute_datex_script_debug_with_result(
        datex_script: &str,
    ) -> ValueContainer {
        execute_datex_script_debug(datex_script).unwrap()
    }

    fn execute_dxb_debug(
        dxb_body: &[u8],
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        let context = ExecutionInput::new(
            dxb_body,
            ExecutionCallerMetadata::local_default(),
            ExecutionOptions { verbose: true },
            Runtime::stub(),
        );
        execute_dxb_sync(context)
    }

    async fn execute_datex_script_with_runtime(
        config: RuntimeConfig,
        datex_script: &str,
    ) -> Result<Option<ValueContainer>, ExecutionError> {
        RuntimeRunner::new(config)
            .run(async |runtime| {
                let (dxb, _) =
                    compile_script(datex_script, CompileOptions::default(), runtime.clone())
                        .unwrap();
                let context = ExecutionInput::new(
                    &dxb,
                    ExecutionCallerMetadata::local_default(),
                    ExecutionOptions { verbose: true },
                    runtime,
                );
                execute_dxb(context).await
            })
            .await
    }

    #[test]
    fn empty_script() {
        assert_eq!(execute_datex_script_debug(""), None);
    }

    #[test]
    fn empty_script_semicolon() {
        assert_eq!(execute_datex_script_debug(";;;"), None);
    }

    #[test]
    fn single_value() {
        assert_eq!(
            execute_datex_script_debug_with_result("42"),
            Integer::from(42i8).into()
        );
    }

    #[test]
    fn single_value_semicolon() {
        assert_eq!(execute_datex_script_debug("42;"), None)
    }

    #[test]
    fn is() {
        let result = execute_datex_script_debug_with_result("1 is 1");
        assert_eq!(result, false.into());
        assert_structural_eq!(result, ValueContainer::from(false));
    }

    #[test]
    fn equality() {
        let result = execute_datex_script_debug_with_result("1 == 1");
        assert_eq!(result, true.into());
        assert_structural_eq!(result, ValueContainer::from(true));

        let result = execute_datex_script_debug_with_result("1 == 2");
        assert_eq!(result, false.into());
        assert_structural_eq!(result, ValueContainer::from(false));

        let result = execute_datex_script_debug_with_result("1 != 2");
        assert_eq!(result, true.into());
        assert_structural_eq!(result, ValueContainer::from(true));

        let result = execute_datex_script_debug_with_result("1 != 1");
        assert_eq!(result, false.into());
        assert_structural_eq!(result, ValueContainer::from(false));
        let result = execute_datex_script_debug_with_result("1 === 1");
        assert_eq!(result, true.into());

        assert_structural_eq!(result, ValueContainer::from(true));
        let result = execute_datex_script_debug_with_result("1 !== 2");
        assert_eq!(result, true.into());
        assert_structural_eq!(result, ValueContainer::from(true));

        let result = execute_datex_script_debug_with_result("1 !== 1");
        assert_eq!(result, false.into());
        assert_structural_eq!(result, ValueContainer::from(false));
    }

    #[test]
    fn single_value_scope() {
        let result = execute_datex_script_debug_with_result("(42)");
        assert_eq!(result, Integer::from(42i8).into());
        assert_structural_eq!(result, ValueContainer::from(42_u128));
    }

    #[test]
    fn add() {
        let result = execute_datex_script_debug_with_result("1 + 2");
        assert_eq!(result, Integer::from(3i8).into());
        assert_structural_eq!(result, ValueContainer::from(3i8));
    }

    #[test]
    fn nested_scope() {
        let result = execute_datex_script_debug_with_result("1 + (2 + 3)");
        assert_eq!(result, Integer::from(6i8).into());
    }

    #[test]
    fn empty_list() {
        let result = execute_datex_script_debug_with_result("[]");
        let list: List = result.try_as().unwrap();
        assert_eq!(list.len(), 0);
        assert_eq!(result, Vec::<ValueContainer>::new().into());
        assert_eq!(result, ValueContainer::from(Vec::<ValueContainer>::new()));
    }

    #[test]
    fn list() {
        let result = execute_datex_script_debug_with_result("[1, 2, 3]");
        let list: List = result.try_as().unwrap();
        let expected = datex_list![
            Integer::from(1i8),
            Integer::from(2i8),
            Integer::from(3i8)
        ];
        assert_eq!(list.len(), 3);
        assert_eq!(result, expected.into());
        assert_ne!(result, ValueContainer::from(vec![1, 2, 3]));
        assert_structural_eq!(result, ValueContainer::from(vec![1, 2, 3]));
    }

    #[test]
    fn list_with_nested_scope() {
        let result = execute_datex_script_debug_with_result("[1, (2 + 3), 4]");
        let expected = datex_list![
            Integer::from(1i8),
            Integer::from(5i8),
            Integer::from(4i8)
        ];

        assert_eq!(result, expected.into());
        assert_ne!(result, ValueContainer::from(vec![1_u8, 5_u8, 4_u8]));
        assert_structural_eq!(
            result,
            ValueContainer::from(vec![1_u8, 5_u8, 4_u8])
        );
    }

    #[test]
    fn boolean() {
        let result = execute_datex_script_debug_with_result("true");
        assert_eq!(result, true.into());
        assert_structural_eq!(result, ValueContainer::from(true));

        let result = execute_datex_script_debug_with_result("false");
        assert_eq!(result, false.into());
        assert_structural_eq!(result, ValueContainer::from(false));
    }

    #[test]
    fn decimal() {
        let result = execute_datex_script_debug_with_result("1.5");
        assert_eq!(result, Decimal::from_string("1.5").unwrap().into());
        assert_structural_eq!(result, ValueContainer::from(1.5));
    }

    #[test]
    fn decimal_and_integer() {
        let result = execute_datex_script_debug_with_result("-2341324.0");
        assert_eq!(result, Decimal::from_string("-2341324").unwrap().into());
        assert!(!result.structural_eq(&ValueContainer::from(-2341324)));
    }

    #[test]
    fn integer() {
        let result = execute_datex_script_debug_with_result("2");
        assert_eq!(result, Integer::from(2).into());
        assert_ne!(result, 2_u8.into());
        assert_structural_eq!(result, ValueContainer::from(2_i8));
    }

    #[test]
    fn typed_integer() {
        let result = execute_datex_script_debug_with_result("-2i16");
        assert_eq!(result, TypedInteger::from(-2i16).into());
        assert_structural_eq!(result, ValueContainer::from(-2_i16));

        let result = execute_datex_script_debug_with_result("2i32");
        assert_eq!(result, TypedInteger::from(2i32).into());
        assert_structural_eq!(result, ValueContainer::from(2_i32));

        let result = execute_datex_script_debug_with_result("-2i64");
        assert_eq!(result, TypedInteger::from(-2i64).into());
        assert_structural_eq!(result, ValueContainer::from(-2_i64));

        let result = execute_datex_script_debug_with_result("2i128");
        assert_eq!(result, TypedInteger::from(2i128).into());
        assert_structural_eq!(result, ValueContainer::from(2_i128));

        let result = execute_datex_script_debug_with_result("2u8");
        assert_eq!(result, TypedInteger::from(2_u8).into());
        assert_structural_eq!(result, ValueContainer::from(2_u8));

        let result = execute_datex_script_debug_with_result("2u16");
        assert_eq!(result, TypedInteger::from(2_u16).into());
        assert_structural_eq!(result, ValueContainer::from(2_u16));

        let result = execute_datex_script_debug_with_result("2u32");
        assert_eq!(result, TypedInteger::from(2_u32).into());
        assert_structural_eq!(result, ValueContainer::from(2_u32));

        let result = execute_datex_script_debug_with_result("2u64");
        assert_eq!(result, TypedInteger::from(2_u64).into());
        assert_structural_eq!(result, ValueContainer::from(2_u64));

        let result = execute_datex_script_debug_with_result("2u128");
        assert_eq!(result, TypedInteger::from(2_u128).into());
        assert_structural_eq!(result, ValueContainer::from(2_u128));

        let result = execute_datex_script_debug_with_result("2ibig");
        assert_eq!(result, TypedInteger::IBig(Integer::from(2)).into());
        assert_structural_eq!(result, ValueContainer::from(2));
    }

    #[test]
    fn null() {
        let result = execute_datex_script_debug_with_result("null");
        assert_eq!(result, ValueContainer::from(CoreValue::Null));
        assert_eq!(result, CoreValue::Null.into());
        assert_structural_eq!(result, ValueContainer::from(CoreValue::Null));
    }

    #[test]
    fn map() {
        let result =
            execute_datex_script_debug_with_result("{x: 1, y: 2, z: 42}");
        let map: CoreValue = result.get_cloned_value().inner;
        let map: Map = map.try_into().unwrap();

        // form and size
        assert_eq!(map.to_string(), "{\"x\": 1, \"y\": 2, \"z\": 42}");
        assert_eq!(map.size(), 3);

        info!("Map: {:?}", map);

        // access by key
        assert_eq!(map.get("x"), Ok(&Integer::from(1).into()));
        assert_eq!(map.get("y"), Ok(&Integer::from(2).into()));
        assert_eq!(map.get("z"), Ok(&Integer::from(42).into()));

        // structural equality checks
        let expected_se: Map = Map::from(vec![
            ("x".to_string(), 1.into()),
            ("y".to_string(), 2.into()),
            ("z".to_string(), 42.into()),
        ]);
        assert_structural_eq!(map, expected_se);

        // strict equality checks
        let expected_strict: Map = Map::from(vec![
            ("x".to_string(), Integer::from(1).into()),
            ("y".to_string(), Integer::from(2).into()),
            ("z".to_string(), Integer::from(42).into()),
        ]);
        debug!("Expected map: {expected_strict}");
        debug!("Map result: {map}");
        // FIXME #104 type information gets lost on compile
        // assert_eq!(result, expected.into());
    }

    #[test]
    fn empty_map() {
        let result = execute_datex_script_debug_with_result("{}");
        let map: CoreValue = result.clone().get_cloned_value().inner;
        let map: Map = map.try_into().unwrap();

        // form and size
        assert_eq!(map.to_string(), "{}");
        assert_eq!(map.size(), 0);

        info!("Map: {:?}", map);
    }

    #[test]
    fn statements() {
        let result = execute_datex_script_debug_with_result("1; 2; 3");
        assert_eq!(result, Integer::from(3).into());
    }

    #[test]
    fn single_terminated_statement() {
        let result = execute_datex_script_debug("1;");
        assert_eq!(result, None);
    }

    #[test]
    fn const_declaration() {
        let result = execute_datex_script_debug_with_result("const x = 42; x");
        assert_eq!(result, Integer::from(42).into());
    }

    #[test]
    fn const_declaration_with_addition() {
        let result =
            execute_datex_script_debug_with_result("const x = 1 + 2; x");
        assert_eq!(result, Integer::from(3).into());
    }

    #[test]
    fn unbox_shared() {
        let result =
            execute_datex_script_debug_with_result("const x = shared 42; *x");
        assert_eq!(result, ValueContainer::from(Integer::from(42)));
    }

    #[test]
    fn shared_assignment_mut_ref_to_mut() {
        let result = execute_datex_script_debug_with_result(
            "const x = 'mut shared mut 42; x",
        );
        assert_matches!(result, ValueContainer::Shared(SharedContainer::Referenced(ref container)) if
            container.container_mutability().clone() == SharedContainerMutability::Mutable &&
            container.reference_mutability() == ReferenceMutability::Mutable
        );
        assert_value_eq!(result, ValueContainer::from(Integer::from(42)));
    }

    #[test]
    fn shared_assignment_immut_ref_to_mut() {
        let result = execute_datex_script_debug_with_result(
            "const x = 'shared mut 42; x",
        );
        assert_matches!(result, ValueContainer::Shared(SharedContainer::Referenced(ref container)) if
            container.container_mutability().clone() == SharedContainerMutability::Mutable &&
            container.reference_mutability() == ReferenceMutability::Immutable
        );

        assert_value_eq!(result, ValueContainer::from(Integer::from(42)));
    }

    #[test]
    fn shared_assignment_immut_ref() {
        let result =
            execute_datex_script_debug_with_result("const x = 'shared 42; x");
        assert_matches!(result, ValueContainer::Shared(SharedContainer::Referenced(ref container)) if
            container.container_mutability().clone() == SharedContainerMutability::Immutable &&
            container.reference_mutability() == ReferenceMutability::Immutable
        );

        assert_value_eq!(result, ValueContainer::from(Integer::from(42)));
    }

    #[test]
    fn shared_assignment_immut() {
        let result =
            execute_datex_script_debug_with_result("const x = shared 42; x");
        assert_matches!(result, ValueContainer::Shared(SharedContainer::Owned(ref container)) if
            container.container_mutability().clone() == SharedContainerMutability::Immutable
        );

        assert_value_eq!(result, ValueContainer::from(Integer::from(42)));
    }

    #[test]
    fn shared_assignment_mut() {
        let result = execute_datex_script_debug_with_result(
            "const x = shared mut 42; x",
        );
        assert_matches!(result, ValueContainer::Shared(SharedContainer::Owned(
            ref container @ OwnedSharedContainer { .. }
        )) if container.container_mutability().clone() == SharedContainerMutability::Mutable);
        assert_value_eq!(result, ValueContainer::from(Integer::from(42)));
    }

    #[test]
    fn shared_assignment_mut_ref_to_immut() {
        let result = execute_datex_script_debug_with_error(
            "const x = 'mut shared 42; x",
        );
        assert_matches!(
            result,
            Err(ExecutionError::MutableReferenceToNonMutableValue)
        );
    }

    #[test]
    fn shared_value_add_assignment() {
        let result = execute_datex_script_debug_with_result(
            "var x = shared mut 42; *x += 1; x",
        );

        assert_value_eq!(result, ValueContainer::from(Integer::from(43)));
        assert_matches!(result, ValueContainer::Shared(..));
        if let ValueContainer::Shared(shared) = &result {
            assert_eq!(
                shared.inner().base_shared_container().mutability,
                SharedContainerMutability::Mutable
            );
        } else {
            panic!("Expected shared value");
        }
    }

    #[test]
    fn shared_value_sub_assignment() {
        let result = execute_datex_script_debug_with_result(
            "const x = 'mut shared mut 42; *x -= 1",
        );
        assert_value_eq!(result, ValueContainer::from(Integer::from(41)));

        let result = execute_datex_script_debug_with_result(
            "const x = 'mut shared mut 42; *x -= 1; x",
        );

        // FIXME #414 due to addition the resulting value container of the slot
        // is no longer a reference but a value what is incorrect.
        // assert_matches!(result, ValueContainer::Reference(..));
        assert_value_eq!(result, ValueContainer::from(Integer::from(41)));
    }

    #[tokio::test]
    async fn env_slot() {
        let res = execute_datex_script_with_runtime(
            RuntimeConfig {
                env: Some(HashMap::from([(
                    "TEST_ENV_VAR".to_string(),
                    "test_value".to_string(),
                )])),
                ..Default::default()
            },
            "#env",
        )
        .await
        .unwrap();
        assert!(res.is_some());
        let env = res
            .unwrap()
            .try_as::<Map>()
            .unwrap();
        assert_eq!(env.get("TEST_ENV_VAR"), Ok(&"test_value".into()));
    }

    #[test]
    fn shebang() {
        let result = execute_datex_script_debug_with_result("#!datex\n42");
        assert_eq!(result, Integer::from(42).into());
    }

    #[test]
    fn single_line_comment() {
        let result =
            execute_datex_script_debug_with_result("// this is a comment\n42");
        assert_eq!(result, Integer::from(42).into());

        let result = execute_datex_script_debug_with_result(
            "// this is a comment\n// another comment\n42",
        );
        assert_eq!(result, Integer::from(42).into());
    }

    #[test]
    fn multi_line_comment() {
        let result = execute_datex_script_debug_with_result(
            "/* this is a comment */\n42",
        );
        assert_eq!(result, Integer::from(42).into());

        let result = execute_datex_script_debug_with_result(
            "/* this is a comment\n   with multiple lines */\n42",
        );
        assert_eq!(result, Integer::from(42).into());

        let result = execute_datex_script_debug_with_result("[1, /* 2, */ 3]");
        let expected = datex_list![Integer::from(1), Integer::from(3)];
        assert_eq!(result, expected.into());
    }

    #[test]
    fn continuous_execution() {
        assert_unbounded_input_matches_output(
            vec!["1", "2"],
            vec![Some(Integer::from(1).into()), Some(Integer::from(2).into())],
            Runtime::stub()
        )
    }

    #[test]
    fn continuous_execution_multiple_external_interrupts() {
        let runtime = Runtime::stub();

        assert_unbounded_input_matches_output(
            vec!["1", "integer", "boolean"],
            vec![
                Some(Integer::from(1).into()),
                Some(ValueContainer::Shared(runtime.clone().memory().borrow().get_core_type_reference(CoreLibBaseTypeId::Integer).into())),
                Some(ValueContainer::Shared(runtime.clone().memory().borrow().get_core_type_reference(CoreLibBaseTypeId::Boolean).into())),
            ],
            runtime,
        )
    }
}
