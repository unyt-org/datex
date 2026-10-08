use crate::{
    preludes::derive::CoreLibTypeId,
    traits::get_core_lib_type_id::GetCoreLibTypeId, values::value::Value,
};

impl GetCoreLibTypeId for Value {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        match self {
            Value::Core(core_value) => core_value.inner.into(),
            Value::Native(native_value) => native_value.core_lib_type_id(),
        }
    }
}
