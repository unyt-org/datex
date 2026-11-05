use crate::{
    runtime::Runtime,
    shared_wrappers::shared::{shared_ref::SharedRef, to_shared::ToShared},
    traits::get_datex_type::GetDatexType,
    values::core_values::{
        native::DatexNative, receiver::Receiver, sender::Sender,
    },
};

/// TODO: Additional receiver for msmc must also be implemented
pub fn create_channel<T: DatexNative + GetDatexType>(
    runtime: &Runtime,
) -> (Sender<T>, SharedRef<Receiver<T>>) {
    let receiver_ref =
        Receiver::<T>::new_with_remote_sender().shared_ref(runtime);
    (
        Sender::try_from(receiver_ref.container()).unwrap(), // Safe because receiver_ref contains Receiver<T>
        receiver_ref,
    )
}

#[cfg(test)]
mod tests {
    use crate::runtime::Runtime;

    use super::*;

    #[test]
    fn test_create_channel() {
        let runtime = Runtime::stub();
        let (sender, receiver_ref) = create_channel::<i32>(&runtime);
    }
}
