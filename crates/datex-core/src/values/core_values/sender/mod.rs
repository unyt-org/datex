use crate::channel::mpsc;
use crate::channel::mpsc::create_unbounded_channel;
use crate::runtime::Runtime;
use crate::shared_values::{SharedContainer, SharedContainerMutability};
use crate::shared_values::traits::SharedContainerCommon;
use crate::shared_wrappers::shared::shared_ref::SharedRef;
use crate::shared_wrappers::shared::to_shared::ToShared;
use crate::traits::get_datex_type::GetDatexType;
use crate::types::entity_type::EntityType;
use crate::values::core_values::native::{DatexNative, DatexNativeBase};
use crate::values::core_values::receiver::Receiver;
use crate::values::value::Value;
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
mod equality;

#[derive(Debug, Clone)]
pub enum Sender<T> {
    LocalReceiver {
        sender: mpsc::UnboundedSender<T>,
    },
    RemoteReceiver {
        receiver: SharedContainer,
    },
}

impl<T> Sender<T> {
    pub fn new_with_local_receiver() -> (Self, mpsc::UnboundedReceiver<T>) {
        let (sender, receiver) = create_unbounded_channel();
        (Sender::LocalReceiver { sender }, receiver)
    }
    
    /// ### Safety
    /// The caller must ensure that the [`SharedContainer`] contains a [Receiver<T>]. 
    pub unsafe fn new_with_remote_receiver(receiver: SharedContainer) -> Self {
        Sender::RemoteReceiver { receiver }
    }

    /// Sends a value through the sender. Returns an error if the receiver has been dropped.
    pub async fn send(&mut self, value: T) -> Result<(), ()> {
        match self {
            Sender::LocalReceiver { sender } => sender.send(value).await,
            Sender::RemoteReceiver { receiver } => todo!()
        }
    }
}


impl TryFrom<SharedContainer> for Sender<ValueContainer> {
    type Error = ();
    fn try_from(value: SharedContainer) -> Result<Self, Self::Error> {
        if value.container_mutability() == SharedContainerMutability::Immutable
        {
            // check if the contained value is a receiver
            let is_receiver = {
                let val = value.collapsed_value();
                val.borrow().try_as::<Receiver<ValueContainer>>().is_some()
            };

            if is_receiver {
                Ok(Sender::RemoteReceiver { receiver: value })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
}


pub type ValueContainerSender = Sender<ValueContainer>;