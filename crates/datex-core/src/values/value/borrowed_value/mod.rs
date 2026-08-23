use core::fmt::Debug;
use crate::types::type_definition::TypeDefinition;
use crate::utils::sheep::Sheep;
use crate::values::core_values::callable::Callable;
use crate::values::core_values::native::NativeCoreValue;

/// Similar to [Value], but contains a [BorrowedCoreValue] instead of a [CoreValue]. 
/// It is used to represent a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Debug)]
pub struct BorrowedValue<'a> {
    pub(crate) inner: BorrowedCoreValue<'a>,
    pub(crate) custom_type: Option<TypeDefinition>,
}

/// Similar to [CoreValue], but it is a potentially borrowed reference to a [CoreValue] variant instead of owning it.
#[derive(Debug)]
pub enum BorrowedCoreValue<'a> {
    Callable(Sheep<'a, Callable>),
    Native(Sheep<'a, NativeCoreValue>),
}