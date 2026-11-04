mod classification;
mod convert_parts;
mod convert_value;
mod datex_hash;
mod datex_native;
mod datex_native_structural;
mod get_core_lib_type_id;
mod get_datex_type;
mod iter_parts;
mod serde_dif;
mod to_datex_expression_data;
mod to_instructions;
mod update_handler;
mod value_access;

use crate::channel::mpsc;
use crate::traits::try_clone::TryClone;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

#[derive(Debug)]
pub struct Receiver<T> {
    sender: mpsc::Receiver<T>,
}


impl<T> TryClone for Receiver<T> {
    fn try_clone(&self) -> Result<Value, ()> {
        Err(())
    }
}


pub type ValueContainerReceiver = Receiver<ValueContainer>;