use crate::shared_values::shared_container::SharedContainer;
use crate::values::core_value::CoreValue;

/// A wrapper around an [SharedContainer] which guarantees
/// that the contained value is always a [CoreValue::Type]
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SharedContainerContainingType(SharedContainer);


impl SharedContainerContainingType {
    
    pub fn new(container: SharedContainer) -> Self {
        
    }
    
}