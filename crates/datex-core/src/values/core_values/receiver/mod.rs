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
mod equality;

use crate::channel::mpsc;
use crate::channel::mpsc::create_unbounded_channel;
use crate::shared_values::SharedContainer;
use crate::traits::try_clone::TryClone;
use crate::values::core_values::native::DatexNative;
use crate::values::value::Value;
use crate::values::value_container::ValueContainer;

/// TODO: Receivers should not be referencable, or at least no be readable from reference, only from owned value
#[derive(Debug)]
pub enum Receiver<T> {
    LocalSender {
        receiver: mpsc::UnboundedReceiver<T>,
    },
    RemoteSender {
        sender: mpsc::UnboundedSender<T>,
        receiver: mpsc::UnboundedReceiver<T>,
    },
}

impl<T> Receiver<T> {
    pub fn new_with_local_sender(receiver: mpsc::UnboundedReceiver<T>) -> Self {
        Receiver::LocalSender {
            receiver,
        }
    }

    pub fn new_with_remote_sender() -> Self {
        let (sender, receiver) = create_unbounded_channel();
        Receiver::RemoteSender {
            sender,
            receiver,
        }
    }

    pub fn receiver(&mut self) -> &mut mpsc::UnboundedReceiver<T> {
        match self {
            Receiver::LocalSender { receiver } => receiver,
            Receiver::RemoteSender { receiver, .. } => receiver,
        }
    }
}


impl<T> TryClone for Receiver<T>
where
    Receiver<T>: DatexNative,
{
    fn try_clone(&self) -> Result<Value, ()> {
        Err(())
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        panic!("Receiver cannot be cloned");
    }
}


pub type ValueContainerReceiver = Receiver<ValueContainer>;