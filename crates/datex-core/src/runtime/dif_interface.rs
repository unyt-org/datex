use crate::{
    dif::{
        interface::{
            DIFApplyError, DIFCreatePointerError, DIFInterface,
            DIFObserveError, DIFResolveReferenceError, DIFUpdateError,
        },
        reference::DIFReference,
        r#type::DIFTypeDefinition,
        update::{DIFKey, DIFUpdateData},
        value::{DIFReferenceNotFoundError, DIFValueContainer},
    },
    prelude::*,
    runtime::RuntimeInternal,
    shared_values::{
        observers::{ObserveOptions, Observer, TransceiverId},
        pointer_address::PointerAddress,
        shared_container::{SharedContainer, SharedContainerMutability},
    },
    values::value_container::ValueContainer,
};
use core::result::Result;
use core::cell::Ref;
use crate::shared_values::pointer_address::EndpointOwnedPointerAddress;

impl RuntimeInternal {
    fn resolve_in_memory_reference(
        &'_ self,
        address: &PointerAddress,
    ) -> Option<Ref<'_, SharedContainer>> {
        Ref::filter_map(self.memory.borrow(), |memory| {
            memory.get_reference(address)
        }).ok()
    }
    
    // FIXME #398 implement async resolution
    async fn resolve_reference(
        &self,
        address: &PointerAddress,
    ) -> Option<SharedContainer> {
        self.memory.borrow().get_reference(address).cloned()
    }
}

impl DIFInterface for RuntimeInternal {
    fn update(
        &self,
        source_id: TransceiverId,
        address: PointerAddress,
        update: &DIFUpdateData,
    ) -> Result<(), DIFUpdateError> {
        let reference = self
            .resolve_in_memory_reference(&address)
            .ok_or(DIFUpdateError::ReferenceNotFound)?;
        match update {
            DIFUpdateData::Set { key, value } => {
                let value_container = value.to_value_container(&self.memory)?;
                match key {
                    DIFKey::Text(key) => reference.try_set_property(
                        source_id,
                        Some(update),
                        key,
                        value_container,
                    )?,
                    DIFKey::Index(key) => reference.try_set_property(
                        source_id,
                        Some(update),
                        *key,
                        value_container,
                    )?,
                    DIFKey::Value(key) => {
                        let key = key.to_value_container(&self.memory)?;
                        reference.try_set_property(
                            source_id,
                            Some(update),
                            &key,
                            value_container,
                        )?
                    }
                }
            }
            DIFUpdateData::Replace { value } => reference.try_replace(
                source_id,
                Some(update),
                value.to_value_container(&self.memory)?,
            )?,
            DIFUpdateData::Append { value } => reference.try_append_value(
                source_id,
                Some(update),
                value.to_value_container(&self.memory)?,
            )?,
            DIFUpdateData::Clear => reference.try_clear(source_id)?,
            DIFUpdateData::Delete { key } => match key {
                DIFKey::Text(key) => reference.try_delete_property(
                    source_id,
                    Some(update),
                    key,
                )?,
                DIFKey::Index(key) => reference.try_delete_property(
                    source_id,
                    Some(update),
                    *key,
                )?,
                DIFKey::Value(key) => {
                    let key = key.to_value_container(&self.memory)?;
                    reference.try_delete_property(
                        source_id,
                        Some(update),
                        &key,
                    )?
                }
            },
            DIFUpdateData::ListSplice {
                start,
                delete_count,
                items,
            } => {
                reference.try_list_splice(
                    source_id,
                    Some(update),
                    *start..(start + delete_count),
                    items
                        .iter()
                        .map(|item| item.to_value_container(&self.memory))
                        .collect::<Result<
                            Vec<ValueContainer>,
                            DIFReferenceNotFoundError,
                        >>()?,
                )?
            }
        };

        Ok(())
    }

    fn apply(
        &self,
        _callee: DIFValueContainer,
        _value: DIFValueContainer,
    ) -> Result<DIFValueContainer, DIFApplyError> {
        core::todo!("#400 Undescribed by author.")
    }

    fn create_pointer(
        &self,
        value: DIFValueContainer,
        allowed_type: Option<DIFTypeDefinition>,
        mutability: SharedContainerMutability,
    ) -> Result<EndpointOwnedPointerAddress, DIFCreatePointerError> {
        let container = value.to_value_container(&self.memory)?;
        let type_container = if let Some(_allowed_type) = &allowed_type {
            core::todo!(
                "FIXME: Implement type_container creation from DIFTypeDefinition"
            )
        } else {
            None
        };

        let pointer = self.memory.borrow_mut().get_new_owned_local_pointer();
        let address = pointer.address().clone();

        let reference = SharedContainer::try_boxed_owned(
            container,
            type_container,
            pointer,
            mutability,
        )?;
        self.memory
            .borrow_mut()
            .register_shared_container(&reference);
        Ok(address)
    }

    async fn resolve_pointer_address_external(
        &self,
        address: PointerAddress,
    ) -> Result<DIFReference, DIFResolveReferenceError> {
        let reference = self.resolve_in_memory_reference(&address);
        match reference {
            Some(ptr) => Ok(DIFReference::from_reference(&ptr)),
            None => {
                core::todo!("#399 Implement async resolution of references")
            }
        }
    }

    fn resolve_pointer_address_in_memory(
        &self,
        address: PointerAddress,
    ) -> Result<DIFReference, DIFResolveReferenceError> {
        let reference = self.resolve_in_memory_reference(&address);
        match reference {
            Some(ptr) => Ok(DIFReference::from_reference(&ptr)),
            None => Err(DIFResolveReferenceError::ReferenceNotFound),
        }
    }

    fn observe_pointer(
        &self,
        transceiver_id: TransceiverId,
        address: PointerAddress,
        options: ObserveOptions,
        callback: impl Fn(&DIFUpdateData, TransceiverId) + 'static,
    ) -> Result<u32, DIFObserveError> {
        let reference = self
            .resolve_in_memory_reference(&address)
            .ok_or(DIFObserveError::ReferenceNotFound)?;
        Ok(reference.observe(Observer {
            transceiver_id,
            options,
            callback: Rc::new(callback),
        })?)
    }

    fn update_observer_options(
        &self,
        address: PointerAddress,
        observer_id: u32,
        options: ObserveOptions,
    ) -> Result<(), DIFObserveError> {
        let reference = self
            .resolve_in_memory_reference(&address)
            .ok_or(DIFObserveError::ReferenceNotFound)?;
        reference
            .update_observer_options(observer_id, options)
            .map_err(DIFObserveError::ObserveError)
    }

    fn unobserve_pointer(
        &self,
        address: PointerAddress,
        observer_id: u32,
    ) -> Result<(), DIFObserveError> {
        let reference = self
            .resolve_in_memory_reference(&address)
            .ok_or(DIFObserveError::ReferenceNotFound)?;
        reference
            .unobserve(observer_id)
            .map_err(DIFObserveError::ObserveError)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        dif::{
            interface::DIFInterface,
            representation::DIFValueRepresentation,
            update::DIFUpdateData,
            value::{DIFValue, DIFValueContainer},
        },
        prelude::*,
        runtime::{RuntimeConfig, RuntimeRunner},
        shared_values::{
            observers::ObserveOptions,
            shared_container::SharedContainerMutability,
        },
        values::{core_values::map::Map, value_container::ValueContainer},
    };
    use core::cell::RefCell;
    use crate::shared_values::pointer_address::PointerAddress;

    #[test]
    fn struct_serde() {
        let map = ValueContainer::from(Map::from(vec![
            ("a".to_string(), 1.into()),
            ("b".to_string(), "text".into()),
        ]));
        let dif_value = DIFValueContainer::from_value_container(&map);
        let _ = serde_json::to_string(&dif_value).unwrap();
    }

    #[tokio::test]
    async fn test_create_and_observe_pointer() {
        RuntimeRunner::new(RuntimeConfig::default())
            .run(async |runtime| {
                let runtime = runtime.internal;

                let pointer_address = runtime
                    .create_pointer(
                        DIFValueContainer::Value(DIFValue::from(
                            DIFValueRepresentation::String(
                                "Hello, world!".to_string(),
                            ),
                        )),
                        None,
                        SharedContainerMutability::Mutable,
                    )
                    .expect("Failed to create pointer");
                let pointer_address = PointerAddress::EndpointOwned(pointer_address);

                let observed = Rc::new(RefCell::new(None));
                let observed_clone = observed.clone();

                let observer_id = Rc::new(RefCell::new(None));
                let observer_id_clone = observer_id.clone();
                let runtime_clone = runtime.clone();
                let pointer_address_clone = pointer_address.clone();

                // Observe the pointer
                observer_id.replace(Some(
                    runtime
                        .observe_pointer(
                            0,
                            pointer_address_clone.clone(),
                            ObserveOptions::default(),
                            move |update, _| {
                                observed_clone.replace(Some(update.clone()));
                                // unobserve after first update
                                runtime_clone
                                    .unobserve_pointer(
                                        pointer_address_clone.clone(),
                                        observer_id_clone.borrow().unwrap(),
                                    )
                                    .unwrap();
                            },
                        )
                        .expect("Failed to observe pointer"),
                ));

                // Update the pointer value
                runtime
                    .update(
                        1,
                        pointer_address.clone(),
                        &DIFUpdateData::replace(DIFValue::from(
                            DIFValueRepresentation::String(
                                "Hello, Datex!".to_string(),
                            ),
                        )),
                    )
                    .expect("Failed to update pointer");

                // Check if the observed value matches the update
                let observed_value = observed.borrow();
                assert_eq!(
                    *observed_value,
                    Some(DIFUpdateData::replace(DIFValue::from(
                        DIFValueRepresentation::String(
                            "Hello, Datex!".to_string(),
                        )
                    )))
                );

                // try unobserve again, should fail
                assert!(
                    runtime
                        .unobserve_pointer(
                            pointer_address.clone(),
                            observer_id.borrow().unwrap()
                        )
                        .is_err()
                );
            })
            .await;
    }
}
