#[cfg(test)]
pub mod test;

use crate::{
    prelude::*,
    runtime::Runtime,
    traits::get_datex_type::GetDatexType,
    utils::logger::{LogMessage, register_log_sender},
    values::value_container::ValueContainer,
    random::RandomState
};
use datex_macros_internal::{Datex, datex};

#[datex(name = "inspector")]
mod datex_inspector {
    use core::cell::RefCell;
    use log::info;
    use super::*;
    use crate::{
        shared_wrappers::shared::{shared_ref::SharedRef, to_shared::ToShared},
        values::core_values::{channel::create_channel, receiver::Receiver},
    };
    use crate::network::com_hub::{BlockDirection, BlockInterceptor};
    use crate::network::com_hub::metadata::ComHubMetadata;
    use crate::runtime::{RuntimeInfo, RuntimeStats};

    #[derive(Datex, Debug, Clone)]
    #[datex(structural)]
    pub struct CapturedBlockData {
        block: Vec<u8>,
        socket_uuid: String,
        direction: BlockDirection,
    }

    #[derive(Datex, Debug, Clone)]
    #[datex(no_deserialize, builtin = 0x000001)]
    /// The Inspector is a built-in debug helper that provides access to the DATEX runtime and its properties.
    /// It is currently publicly accessible, but will be restricted with permission rules in the future.
    pub struct Inspector {
        // TODO: #[datex(readonly)]
        // TODO: distinguish between pub and private entity properties
        pub name: String,
        pub info: RuntimeInfo,
    }

    #[datex]
    impl Inspector {
        /// Returns a stream of all global log messages of the endpoint.
        pub fn logs(
            &self,
            #[runtime] runtime: &Runtime,
        ) -> SharedRef<Receiver<LogMessage>> {
            let (sender, receiver) = create_channel(runtime);
            register_log_sender(sender);
            receiver
        }

        /// Returns a stream of all incoming and outgoing blocks on the endpoint (excluding debug blocks).
        pub fn blocks(
            &self,
            #[runtime] runtime: &Runtime,
        ) -> SharedRef<Receiver<CapturedBlockData>> {
            let (mut sender, receiver) = create_channel(runtime);

            let uuid = Rc::new(RefCell::new(None));
            let uuid_clone = uuid.clone();
            let runtime_clone = runtime.clone();

            *uuid.borrow_mut() = Some(runtime.com_hub().register_block_interceptor(BlockInterceptor::new(
                move |block| {
                    info!("Inspector captured block: {:?}", block);
                    let captured_block = CapturedBlockData {
                        block: block.block.to_bytes(),
                        socket_uuid: block.socket_uuid.to_string(),
                        direction: block.direction,
                    };
                    if sender.send(captured_block).is_err() {
                        info!("Inspector blocks stream closed, unregistering block interceptor");
                        // remove interceptor
                        runtime_clone.com_hub().unregister_block_interceptor(uuid_clone.borrow().as_ref().unwrap());
                    }
                },
                false, // exclude debug blocks
            )));

            receiver.mark_debug(true);

            receiver
        }


        /// Returns the current metadata information (connected interfaces, sockets) of the endpoint's com hub.
        pub fn com_hub_metadata(
            &self,
            #[runtime] runtime: &Runtime
        ) -> ComHubMetadata {
            runtime.com_hub().get_metadata()
        }

        /// Returns the current runtime statistics of the endpoint.
        pub fn stats(
            &self,
            #[runtime] runtime: &Runtime,
        ) -> RuntimeStats {
            runtime.get_stats()
        }

        /// Attempts to reboot the endpoint associated with the inspector.
        /// Currently, this function is not implemented and will panic if called.
        pub fn try_reboot_endpoint(&self) {
            panic!("try_reboot_endpoint is not implemented yet");
        }
        // // TODO
        // pub fn method_for_shared_instance(self: SharedRef<Self>) -> String {
        //     self.borrow().name.clone()
        // }
    }

    /// Creates a new [Inspector] instance.
    /// TODO: this should return a unique temporary reference &mut Inspector that can be used by the caller.
    #[datex]
    pub fn create(
        name: String,
        #[runtime] runtime: &Runtime,
    ) -> SharedRef<Inspector> {
        let inspector = Inspector { name, info: runtime.info().clone() }.shared_ref(runtime);
        inspector.mark_debug(true);
        inspector
    }

    // pub async fn async_test(a: String) -> String {
    //     format!("a = {}", a)
    // }
}

/// Registers the `inspector` namespace in the runtime, allowing users to create Inspector instances.
pub fn register_inspector_namespace(runtime: &Runtime) {
    let mut memory = runtime.shared_references_cache_refcell().borrow_mut();
    let inspector_type = ValueContainer::from(
        datex_inspector::Inspector::datex_type(&mut memory),
    );

    runtime
        .endpoint_properties_mut()
        .insert("Inspector".to_string(), inspector_type);
}

#[cfg(test)]
mod tests {
    use crate::{
        inspector::datex_inspector::Inspector,
        traits::{
            apply::{Apply, ApplyArgument},
            convert_value_container::ConvertValueContainer,
            value_access::ValueAccess,
        },
        types::type_definition::callable::CallableKind,
        values::core_values::callable::{Callable, native_sync_callable},
    };
    use core::ops::DerefMut;

    #[test]
    fn ty() {
        let runtime = Runtime::stub();
        let inspector_type = Inspector::datex_type(
            runtime.shared_references_cache_mut().deref_mut(),
        );

        // create a new inspector instance via inspector.create()
        let inspector =
            datex_inspector::create("Test Inspector".to_string(), &runtime);
        let inspector_container = inspector.to_value_container();
        let inspector_container_clone = inspector_container.clone();

        // call inspector->stats()
        let name_fn = inspector_type
            .try_get_property(&ValueContainer::from("stats"))
            .unwrap()
            .try_as::<Callable>()
            .unwrap();
        let (result, mut borrows) = name_fn
            .try_apply_sync_checked(
                &runtime,
                vec![ApplyArgument::referenced(inspector_container)],
            )
            .unwrap();

        // TODO: check return value
        // borrowed inspector should be given back to the caller
        assert_eq!(borrows.remove(0), inspector_container_clone);
    }

    use super::*;
    #[test]
    fn test_function() {
        let runtime = Runtime::stub();
        let memory = runtime.shared_references_cache_refcell();
        // 1 arg
        let func = |x: u8| x + 1;
        let dx_func_1 = ValueContainer::from(native_sync_callable(
            func,
            None,
            CallableKind::Function,
            &mut memory.borrow_mut(),
        ));
        let res = dx_func_1
            .try_apply_sync(&runtime, vec![4u8.into()])
            .unwrap()
            .0
            .expect("Function should return a value");
        assert_eq!(res.try_as::<u8>(), Some(&5u8));

        // 2 args
        let func_2 = |x: u8, y: u8| x + y;
        let dx_func_2 = ValueContainer::from(native_sync_callable(
            func_2,
            None,
            CallableKind::Function,
            &mut memory.borrow_mut(),
        ));
        let res_2 = dx_func_2
            .try_apply_sync(&runtime, vec![3u8.into(), 4u8.into()])
            .unwrap()
            .0
            .expect("Function should return a value");
        assert_eq!(res_2.try_as::<u8>(), Some(&7u8));
    }
}
