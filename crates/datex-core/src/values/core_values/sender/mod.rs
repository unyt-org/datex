use crate::channel::mpsc;
use crate::values::value_container::ValueContainer;

mod datex_native;
mod classification;
mod convert_parts;
mod convert_value;
mod datex_hash;
mod datex_native_structural;
mod get_core_lib_type_id;
mod get_datex_type;
mod iter_parts;
mod serde_dif;
mod value_access;
mod to_instructions;
mod to_datex_expression_data;
mod update_handler;

#[derive(Clone, Debug)]
pub struct Sender<T> {
    sender: mpsc::UnboundedSender<T>
}

pub type ValueContainerSender = Sender<ValueContainer>;