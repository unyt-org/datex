use crate::{
    prelude::*,
    shared_values::base_shared_value_container::BaseSharedValueContainer,
    utils::freemap::NextKey, value_updates::update_data::Update,
};
use core::{fmt::Display, result::Result};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ObserverError {
    ObserverNotFound,
    ImmutableReference,
}

impl Display for ObserverError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ObserverError::ObserverNotFound => {
                core::write!(f, "Observer not found")
            }
            ObserverError::ImmutableReference => {
                core::write!(f, "Cannot observe an immutable reference")
            }
        }
    }
}

pub type ObserverCallback = Rc<dyn Fn(&Update)>;

/// unique identifier for a transceiver (source of updates)
/// 0-255 are reserved for DIF clients
#[derive(
    Debug, Default, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash,
)]
pub struct TransceiverId(pub u32);

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ObserveOptions {
    /// If true, the transceiver will be notified of changes that originated from itself
    pub relay_own_updates: bool,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
pub struct ObserverId(pub u32);

impl NextKey for ObserverId {
    fn next_key(&mut self) -> Self {
        ObserverId(self.0.next_key())
    }
}

#[derive(Clone)]
pub struct Observer {
    pub transceiver_id: TransceiverId,
    pub options: ObserveOptions,
    pub callback: ObserverCallback,
}

impl Observer {
    /// Creates a new observer with the given callback function,
    /// using default options and a transceiver ID of 0.
    pub fn new<F: Fn(&Update) + 'static>(callback: F) -> Self {
        Observer {
            transceiver_id: TransceiverId(0),
            options: ObserveOptions::default(),
            callback: Rc::new(callback),
        }
    }
}

impl BaseSharedValueContainer {
    /// Adds an observer to this reference that will be notified on value changes.
    /// Returns an error if the reference is immutable.
    /// The returned u32 is an observer ID that can be used to remove the observer later.
    pub fn observe(
        &mut self,
        observer: Observer,
    ) -> Result<ObserverId, ObserverError> {
        self.ensure_mutable_container()?;
        // Add the observer to the list of observers
        // TODO #299: also set observers on child references if not yet active, keep track of active observers
        Ok(self.observers.add(observer))
    }

    /// Removes an observer by its ID.
    /// Returns an error if the observer ID is not found or the reference is immutable.
    pub fn unobserve(
        &mut self,
        observer_id: ObserverId,
    ) -> Result<(), ObserverError> {
        self.ensure_mutable_container()?;
        let removed = self.observers.remove(observer_id);
        if removed.is_some() {
            Ok(())
        } else {
            Err(ObserverError::ObserverNotFound)
        }
    }

    /// Updates the options for an existing observer by its ID.
    /// Returns an error if the observer ID is not found or the reference is immutable.
    pub fn update_observer_options(
        &mut self,
        observer_id: ObserverId,
        options: ObserveOptions,
    ) -> Result<(), ObserverError> {
        self.ensure_mutable_container()?;
        if let Some(observer) = self.observers.get_mut(&observer_id) {
            observer.options = options;
            Ok(())
        } else {
            Err(ObserverError::ObserverNotFound)
        }
    }

    /// Returns a list of all observer IDs currently registered to this reference.
    /// A type reference or immutable reference will always return an empty list.
    pub fn observers_ids(&self) -> Vec<ObserverId> {
        self.observers.keys().cloned().collect()
    }

    /// Removes all observers from this reference.
    /// Returns an error if the reference is immutable.
    pub fn unobserve_all(&mut self) -> Result<(), ObserverError> {
        self.ensure_mutable_container()?;
        for id in self.observers_ids() {
            let _ = self.unobserve(id);
        }
        Ok(())
    }

    /// Ensures that the shared container is mutable and returns it.
    /// Returns an ObserverError if the reference is immutable (or a type container).
    fn ensure_mutable_container(&self) -> Result<(), ObserverError> {
        if !self.is_mutable() {
            return Err(ObserverError::ImmutableReference);
        }
        Ok(())
    }

    /// Notifies all observers of a change represented by the given [Update].
    pub fn notify_observers(&self, dif: &Update) {
        let observer_callbacks: Vec<ObserverCallback> = self
            .observers
            .iter()
            .filter(|(_, f)| {
                // Filter out bounced back transceiver updates if relay_own_updates not enabled
                f.options.relay_own_updates || f.transceiver_id != dif.source_id
            })
            .map(|(_, f)| f.callback.clone())
            .collect();

        // Call each observer synchronously
        for callback in observer_callbacks {
            callback(dif);
        }
    }

    /// Check if there are any observers registered
    pub fn has_observers(&self) -> bool {
        !self.observers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        runtime::memory::Memory,
        shared_values::{
            SharedContainerMutability,
            base_shared_value_container::BaseSharedValueContainer,
            observers::{
                ObserveOptions, Observer, ObserverError, ObserverId,
                TransceiverId,
            },
        },
        value_updates::{
            update_data::{
                ReplaceUpdateData, SetEntryUpdateData, Update, UpdateData,
            },
            update_handler::UpdateHandler,
        },
        values::{
            core_values::map::Map,
            value_container::{ValueContainer, ValueKey},
        },
    };
    use core::{assert_matches, cell::RefCell};

    /// Helper function to record DIF updates observed on a reference
    /// Returns a Rc<RefCell<Vec<DIFUpdate>>> that contains all observed updates
    /// The caller can borrow this to inspect the updates after performing operations on the reference
    fn record_dif_updates(
        reference: &mut BaseSharedValueContainer,
        transceiver_id: TransceiverId,
        observe_options: ObserveOptions,
    ) -> Rc<RefCell<Vec<Update>>> {
        let update_collector = Rc::new(RefCell::new(Vec::new()));
        let update_collector_clone = update_collector.clone();
        reference
            .observe(Observer {
                transceiver_id,
                options: observe_options,
                callback: Rc::new(move |update| {
                    update_collector_clone.borrow_mut().push(Update {
                        source_id: update.source_id,
                        data: update.data.clone(),
                    });
                }),
            })
            .expect("Failed to attach observer");
        update_collector
    }

    #[test]
    fn immutable_reference_observe_fails() {
        let memory = &Memory::new();

        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Immutable,
            memory,
        );
        assert_matches!(
            r.observe(Observer::new(|_| {})),
            Err(ObserverError::ImmutableReference)
        );

        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Mutable,
            memory,
        );
        assert_matches!(r.observe(Observer::new(|_| {})), Ok(_));
    }

    #[test]
    fn observe_and_unobserve() {
        let memory = &Memory::new();

        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Mutable,
            memory,
        );
        assert!(!r.has_observers());
        let observer_id = r.observe(Observer::new(|_| {})).unwrap();
        assert_eq!(observer_id, ObserverId(0));
        assert!(r.has_observers());
        assert!(r.unobserve(observer_id).is_ok());
        assert!(!r.has_observers());
        assert_matches!(
            r.unobserve(observer_id),
            Err(ObserverError::ObserverNotFound)
        );
    }

    #[test]
    fn observer_ids_incremental() {
        let memory = &Memory::new();

        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Mutable,
            memory,
        );
        let id1 = r.observe(Observer::new(|_| {})).unwrap();
        let id2 = r.observe(Observer::new(|_| {})).unwrap();
        assert_eq!(id1, ObserverId(0));
        assert_eq!(id2, ObserverId(1));
        assert!(r.unobserve(id1).is_ok());
        let id3 = r.observe(Observer::new(|_| {})).unwrap();
        assert_eq!(id3, ObserverId(0));
        let id4 = r.observe(Observer::new(|_| {})).unwrap();
        assert_eq!(id4, ObserverId(2));
    }

    #[test]
    fn observe_replace() {
        let memory = &Memory::new();

        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
                memory,
            );
        let observed_updates = record_dif_updates(
            &mut int_ref,
            TransceiverId(0),
            ObserveOptions::default(),
        );

        // Update the value of the reference
        int_ref
            .try_set_value_container(ValueContainer::from(43))
            .expect("Failed to set value");

        // Verify the observed update matches the expected change
        let expected_update = Update {
            source_id: TransceiverId(1),
            data: UpdateData::Replace(ReplaceUpdateData {
                value: ValueContainer::from(43),
            }),
        };

        assert_eq!(*observed_updates.borrow(), vec![expected_update]);
    }

    #[test]
    fn observe_replace_same_transceiver() {
        let memory = &Memory::new();

        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
                memory,
            );
        let observed_update = record_dif_updates(
            &mut int_ref,
            TransceiverId(0),
            ObserveOptions::default(),
        );

        // Update the value of the reference
        int_ref
            .try_set_value_container(ValueContainer::from(43))
            .expect("Failed to set value");

        // No update triggered, same transceiver id
        assert_eq!(*observed_update.borrow(), vec![]);
    }

    #[test]
    fn observe_replace_same_transceiver_relay_own_updates() {
        let memory = &Memory::new();

        let mut int_ref =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                42,
                SharedContainerMutability::Mutable,
                memory,
            );
        let observed_update = record_dif_updates(
            &mut int_ref,
            TransceiverId(0),
            ObserveOptions {
                relay_own_updates: true,
            },
        );

        // Update the value of the reference
        int_ref
            .try_replace(
                ReplaceUpdateData {
                    value: ValueContainer::from(43),
                },
                TransceiverId(0),
            )
            .expect("Failed to set value");

        // update triggered, same transceiver id but relay_own_updates enabled
        let expected_update = Update {
            source_id: TransceiverId(0),
            data: UpdateData::Replace(ReplaceUpdateData {
                value: ValueContainer::from(43),
            }),
        };

        assert_eq!(*observed_update.borrow(), vec![expected_update]);
    }

    #[test]
    fn observe_update_property() {
        let memory = &Memory::new();

        let mut reference =
            BaseSharedValueContainer::new_with_inferred_allowed_type(
                Map::from(vec![
                    ("a".to_string(), ValueContainer::from(1)),
                    ("b".to_string(), ValueContainer::from(2)),
                ]),
                SharedContainerMutability::Mutable,
                memory,
            );
        let observed_updates = record_dif_updates(
            &mut reference,
            TransceiverId(0),
            ObserveOptions::default(),
        );
        // Update a property
        reference
            .try_set_entry(
                SetEntryUpdateData {
                    key: "a".into(),
                    value: ValueContainer::from("val"),
                },
                TransceiverId(1),
            )
            .expect("Failed to set property");
        // Verify the observed update matches the expected change
        let expected_update = Update {
            source_id: TransceiverId(1),
            data: UpdateData::SetEntry(SetEntryUpdateData {
                key: ValueKey::Text("a".to_string()),
                value: ValueContainer::from("val"),
            }),
        };
        assert_eq!(*observed_updates.borrow(), vec![expected_update]);
    }
}
