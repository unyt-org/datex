use crate::{
    shared_values::{SharedContainer, traits::SharedContainerCommon},
    value_updates::{
        errors::UpdateError,
        update_data::Update,
        update_handler::{UpdateHandler, UpdateResult},
    },
};

impl UpdateHandler for SharedContainer {
    fn try_handle_update(&mut self, update: Update) -> UpdateResult {
        if let SharedContainer::Referenced(referenced) = self
            && !referenced.can_mutate()
        {
            return Err(UpdateError::ImmutableReference);
        }

        let update_clone = update.clone();
        let (source_id, operation, path) = update.into_parts();

        let observers = self
            .base_shared_container()
            .get_current_observers(&source_id);

        let result = self
            .base_shared_container_mut()
            .try_handle_update(operation, path)?;

        // call observers
        for observer in observers {
            observer(&update_clone);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        prelude::*,
        runtime::pointer_address_provider::SelfOwnedPointerAddressProvider,
        shared_values::{
            SharedContainerMutability,
            base_shared_value_container::observers::{
                ObserveOptions, Observer, TransceiverId,
            },
        },
        value_updates::update_data::{UpdateData, UpdateOperation},
        values::{core_values::list::List, value_container::ValueContainer},
    };
    use alloc::rc::Rc;
    use core::cell::RefCell;

    fn get_shared_container_with_observer(
        val: impl Into<ValueContainer>,
    ) -> (SharedContainer, Rc<RefCell<Vec<Update>>>) {
        let address_provider = &mut SelfOwnedPointerAddressProvider::default();

        let mut shared = SharedContainer::new_owned_with_inferred_allowed_type(
            val,
            SharedContainerMutability::Mutable,
            address_provider,
        );

        let updates = Rc::new(RefCell::new(Vec::new()));
        let updates_clone = updates.clone();
        shared
            .observe(Observer::new_with_options(
                move |update| {
                    updates_clone.borrow_mut().push(update.clone());
                },
                ObserveOptions {
                    relay_own_updates: true,
                },
            ))
            .unwrap();

        (shared, updates)
    }

    #[test]
    fn update_top_down_trigger_observer() {
        let (mut shared, updates) =
            get_shared_container_with_observer(List::from(vec![
                ValueContainer::from(1),
            ]));

        let update = Update::new(
            TransceiverId::Local,
            UpdateData::new(UpdateOperation::append_entry(
                ValueContainer::from(2),
            )),
        );

        shared.try_handle_update(update.clone()).unwrap();

        {
            let updates = updates.borrow();
            assert_eq!(updates.len(), 1);
            assert_eq!(updates.get(0).unwrap(), &update)
        }
    }

    #[test]
    fn update_bottom_up_trigger_observer() {
        let (shared, updates) =
            get_shared_container_with_observer(List::from(vec![
                ValueContainer::from(1),
            ]));

        {
            let mut list = shared.try_as_mut::<List>().unwrap();
            list.push(2); // FIXME: only trigger observers after ref drop
        }

        {
            let updates = updates.borrow();
            assert_eq!(updates.len(), 1);
            assert_eq!(
                updates.get(0).unwrap(),
                &Update::new(
                    TransceiverId::Local,
                    UpdateData::new(UpdateOperation::append_entry(
                        ValueContainer::from(2)
                    ))
                )
            )
        }
    }
}
