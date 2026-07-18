use crate::{
    global::operators::ModificationOperator,
    value_updates::update_data::UpdateModificationOperator,
};

pub trait OperatorHandler {
    fn get_update_type_for_modification(
        &self,
        operator: ModificationOperator,
    ) -> Result<UpdateModificationOperator, ()>;
}
