use crate::{
    libs::core::type_id::{CoreLibTypeId, CoreLibVariantTypeId},
    traits::get_core_lib_type_id::GetCoreLibTypeId,
    values::core_values::decimal::typed_decimal::TypedDecimal,
};

impl GetCoreLibTypeId for TypedDecimal {
    fn core_lib_type_id(&self) -> CoreLibTypeId {
        CoreLibTypeId::Variant(CoreLibVariantTypeId::Decimal(self.variant()))
    }
}
