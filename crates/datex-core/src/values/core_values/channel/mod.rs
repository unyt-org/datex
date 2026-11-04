use crate::runtime::Runtime;
use crate::shared_values::SharedContainer;
use crate::shared_wrappers::shared::shared_ref::SharedRef;
use crate::shared_wrappers::shared::to_shared::ToShared;
use crate::traits::get_datex_type::GetDatexType;
use crate::values::core_values::native::DatexNative;
use crate::values::core_values::receiver::Receiver;
use crate::values::core_values::sender::Sender;


/// TODO: Additional receiver for msmc must also be implemented
pub fn create_channel<T: DatexNative + GetDatexType>(runtime: &Runtime) -> (Sender<T>, SharedRef<Receiver<T>>) {
    let receiver_ref = Receiver::<T>::new_with_remote_sender().shared_ref(runtime);
    (
        Sender::RemoteReceiver { receiver: SharedContainer::Referenced(receiver_ref.container().clone()) },
        receiver_ref
    )
}
