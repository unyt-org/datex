use crate::{
    channel::{mpsc, mpsc::create_unbounded_channel},
    shared_values::{
        ReferencedSharedContainer, traits::SharedContainerCommon,
        weak_shared_container::WeakSharedContainer,
    },
    values::{
        core_values::{native::DatexNativeBase, receiver::Receiver},
        value_container::ValueContainer,
    },
};

mod classification;
mod convert_parts;
mod convert_value;
mod datex_hash;
mod datex_native;
mod datex_native_structural;
mod equality;
mod get_core_lib_type_id;
mod get_datex_type;
mod iter_parts;
mod serde_dif;
#[cfg(feature = "ast")]
mod to_datex_expression_data;
mod to_instructions;
mod update_handler;
mod value_access;

#[derive(Debug, Clone)]
pub enum Sender<T> {
    LocalReceiver { sender: mpsc::UnboundedSender<T> },
    RemoteReceiver { receiver: WeakSharedContainer },
}

impl<T> Sender<T> {
    pub fn new_with_local_receiver() -> (Self, mpsc::UnboundedReceiver<T>) {
        let (sender, receiver) = create_unbounded_channel();
        (Sender::LocalReceiver { sender }, receiver)
    }

    /// Creates a new Sender that sends to a remote receiver.
    /// The receiver is expected to be a SharedContainer that contains a Receiver<T>.
    /// # Safety
    /// The caller must ensure that the receiver is a valid SharedContainer that contains a Receiver<T>.
    pub unsafe fn new_with_remote_receiver(
        receiver: &ReferencedSharedContainer,
    ) -> Self {
        Sender::RemoteReceiver {
            receiver: receiver.downgrade(),
        }
    }

    /// Sends a value through the sender. Returns an error if the receiver has been dropped.
    pub async fn send(&mut self, value: T) -> Result<(), ()> {
        match self {
            Sender::LocalReceiver { sender } => sender.send(value).await,
            Sender::RemoteReceiver { receiver: _ } => todo!(),
        }
    }
}

impl<T: DatexNativeBase + 'static> TryFrom<&ReferencedSharedContainer>
    for Sender<T>
{
    type Error = ();
    fn try_from(
        value: &ReferencedSharedContainer,
    ) -> Result<Self, Self::Error> {
        // TODO: also check if immutable, but currently not supported by SharedRef

        // check if the contained value is a receiver
        let is_receiver = {
            let val = value.value_container();
            val.try_as::<Receiver<T>>().is_some()
        };

        if is_receiver {
            Ok(Sender::RemoteReceiver {
                receiver: value.downgrade(),
            })
        } else {
            Err(())
        }
    }
}

pub type ValueContainerSender = Sender<ValueContainer>;
