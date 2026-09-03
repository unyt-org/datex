use crate::{
    prelude::*,
    runtime::cache::shared_references_cache::SharedReferencesCache,
    traits::{apply::ApplyArgument, callable::IntoDatexCallable},
    types::type_definition::callable::{CallableKind, CallableTypeDefinition},
    values::{
        core_values::{callable::error::CallableError, endpoint::Endpoint},
        value_container::ValueContainer,
    },
};
use core::{fmt::Debug, hash::Hash, pin::Pin};
use core::ops::DerefMut;
use crate::runtime::Runtime;
use crate::traits::convert_value_container::ConvertValueContainer;
use crate::traits::get_datex_type::GetDatexType;

pub mod apply;
pub mod equality;
pub mod error;
mod serde_dif;
#[cfg(feature = "decompiler")]
mod to_datex_expression_data;
mod value_access;
mod datex_native;
mod get_core_lib_type_id;
mod get_datex_type;
mod convert_parts;
mod classification;
mod datex_native_structural;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

type AsyncCallable = Rc<
    dyn Fn(
        Vec<ApplyArgument>,
        &Runtime,
    ) -> BoxFuture<
        Result<(Option<ValueContainer>, Vec<ValueContainer>), CallableError>,
    >,
>;

type SyncCallable = Rc<
    dyn Fn(
        Vec<ApplyArgument>,
        &Runtime,
    ) -> Result<
        (Option<ValueContainer>, Vec<ValueContainer>),
        CallableError,
    >,
>;

#[derive(Clone)]
pub enum NativeCallable {
    Sync(SyncCallable),
    Async(AsyncCallable),
}

impl Debug for NativeCallable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NativeCallable::Sync(_) => write!(f, "NativeCallable(Sync)"),
            NativeCallable::Async(_) => write!(f, "NativeCallable(Async)"),
        }
    }
}
impl PartialEq for NativeCallable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NativeCallable::Sync(f1), NativeCallable::Sync(f2)) => {
                Rc::as_ptr(f1) == Rc::as_ptr(f2)
            }
            (NativeCallable::Async(f1), NativeCallable::Async(f2)) => {
                Rc::as_ptr(f1) == Rc::as_ptr(f2)
            }
            _ => false,
        }
    }
}
impl Eq for NativeCallable {}

impl Hash for NativeCallable {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            NativeCallable::Sync(f) => {
                let ptr = Rc::as_ptr(f) as *const ();
                ptr.hash(state);
            }
            NativeCallable::Async(f) => {
                let ptr = Rc::as_ptr(f) as *const ();
                ptr.hash(state);
            }
        }
    }
}

impl NativeCallable {
    pub fn new_sync(
        function: impl Fn(
            Vec<ApplyArgument>,
            &Runtime,
        ) -> Result<
            (Option<ValueContainer>, Vec<ValueContainer>),
            CallableError,
        > + 'static,
    ) -> Self {
        NativeCallable::Sync(Rc::new(function))
    }

    pub fn new_async(
        function: impl Fn(
            Vec<ApplyArgument>,
            &Runtime,
        ) -> BoxFuture<
            Result<
                (Option<ValueContainer>, Vec<ValueContainer>),
                CallableError,
            >,
        > + 'static,
    ) -> Self {
        NativeCallable::Async(Rc::new(function))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DatexBytecodeCallable {
    pub injected_values: Vec<ValueContainer>,
    pub body: Vec<u8>,
    pub requires_async: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CallableBody {
    /// A callable implemented in native Rust code.
    Native(NativeCallable),
    /// A callable implemented in Datex bytecode.
    DatexBytecode(DatexBytecodeCallable),
    /// A callable that is a stub for core library functions that are implemented in the runtime.
    CoreStub(CoreStub),
    /// A callable that is hidden and cannot be called directly (normally a callable that exists on a remote endpoint behind a shared value)
    Hidden,
}

impl CallableBody {
    pub fn native_sync(
        native_callable: impl Fn(
            Vec<ApplyArgument>,
            &Runtime,
        ) -> Result<
            (Option<ValueContainer>, Vec<ValueContainer>),
            CallableError,
        > + 'static,
    ) -> Self {
        CallableBody::Native(NativeCallable::new_sync(native_callable))
    }
    pub fn native_async(
        native_callable: impl Fn(
            Vec<ApplyArgument>,
            &Runtime,
        ) -> BoxFuture<
            Result<
                (Option<ValueContainer>, Vec<ValueContainer>),
                CallableError,
            >,
        > + 'static,
    ) -> Self {
        CallableBody::Native(NativeCallable::new_async(native_callable))
    }

    pub fn requires_async(&self) -> bool {
        match self {
            CallableBody::Native(NativeCallable::Sync(_)) => false,
            CallableBody::Native(NativeCallable::Async(_)) => true,
            CallableBody::DatexBytecode(bytecode_callable) => {
                bytecode_callable.requires_async
            }
            CallableBody::CoreStub(_) => false,
            CallableBody::Hidden => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CoreStub {
    Panic,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Callable {
    pub name: Option<String>,
    pub signature: CallableTypeDefinition,
    pub body: CallableBody,
    pub creator: Endpoint,
}

/// Creates a new [Callable] from a native Rust function or closure
pub fn native_sync_callable<F, Args, R>(
    func: F,
    name: Option<String>,
    kind: CallableKind,
    cache: &mut SharedReferencesCache,
) -> Callable
where
    F: IntoDatexCallable<Args, R> + Send + Sync + 'static,
    R: GetDatexType + ConvertValueContainer + 'static,
{
    let parameters = F::parameters(cache);
    let return_type = R::datex_type(cache);
    Callable {
        name,
        signature: CallableTypeDefinition {
            kind,
            parameters,
            requires_async: false,
            rest_parameter: None,
            return_type: Some(Box::new(return_type)),
            yeet_type: None,
        },
        body: CallableBody::Native(NativeCallable::new_sync(move |args, runtime| {
            let result =
                func.invoke(args.into_iter().map(|v| v.value).collect())?;
            Ok((Some(result.to_value_container(runtime.shared_references_cache_mut().deref_mut())), vec![]))
        })),
        creator: Default::default(),
    }
}

/// Creates a new [Callable] from a native Rust async function or closure
pub fn native_async_callable<F, Args, R>(
    func: F,
    name: Option<String>,
    kind: CallableKind,
    cache: &mut SharedReferencesCache,
) -> Callable
where
    F: IntoDatexCallable<Args, R> + Send + Sync + 'static,
    R: GetDatexType + ConvertValueContainer + 'static,
{
    let parameters = F::parameters(cache);
    let return_type = R::datex_type(cache);
    Callable {
        name,
        signature: CallableTypeDefinition {
            kind,
            parameters,
            requires_async: true,
            rest_parameter: None,
            return_type: Some(Box::new(return_type)),
            yeet_type: None,
        },
        body: CallableBody::Native(NativeCallable::new_async(move |args, runtime| {
            // TODO: async invoke
            let result =
                func.invoke(args.into_iter().map(|v| v.value).collect());
            let runtime = runtime.clone();
            Box::pin(async move {
                let result = result?;
                Ok((Some(result.to_value_container(runtime.shared_references_cache_mut().deref_mut())), vec![]))
            })
        })),
        creator: Default::default(),
    }
}
