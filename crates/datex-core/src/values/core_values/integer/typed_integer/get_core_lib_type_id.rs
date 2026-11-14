use crate::{
    libs::core::type_id::{CoreLibTypeId, CoreLibVariantTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::core_values::integer::typed_integer::TypedInteger,
};

impl GetCoreLibTypeId for TypedInteger {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibTypeId::Variant(CoreLibVariantTypeId::Integer(self.variant()))
    }
}
