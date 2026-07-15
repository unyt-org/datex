use crate::{
    dif::serde_context::SerdeContext,
    prelude::*,
    shared_values::base_shared_value_container::BaseSharedValueContainer,
    utils::{freemap::NextKey, serde_serialize_seed::SerializeSeed},
    value_updates::update_data::Update,
    values::{core_value::CoreValue, core_values::endpoint::Endpoint},
};
use core::{
    fmt::{Debug, Display},
    result::Result,
    str::FromStr,
};
use num_traits::ToPrimitive;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{DeserializeSeed, Error, Visitor},
    forward_to_deserialize_any,
};
use serde_with::__private__::DeError;

#[derive(Debug)]
pub enum ObserverError {
    ObserverNotFound,
    ImmutableValue,
}

impl Display for ObserverError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ObserverError::ObserverNotFound => {
                write!(f, "Observer not found")
            }
            ObserverError::ImmutableValue => {
                write!(f, "Cannot observe an immutable reference")
            }
        }
    }
}

pub type ObserverCallback = Rc<dyn Fn(&Update)>;

/// unique identifier for a transceiver (source of updates)
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum TransceiverId {
    #[default]
    /// ID used for local transceivers
    Local,
    /// ID used for remote endpoint transceivers
    Remote(Endpoint),
    /// ID used for DIF clients
    Dif(u8),
}

impl From<&Endpoint> for TransceiverId {
    fn from(endpoint: &Endpoint) -> Self {
        if endpoint.is_local() {
            TransceiverId::Local
        } else {
            TransceiverId::Remote(endpoint.clone())
        }
    }
}

impl From<TransceiverId> for Endpoint {
    fn from(transceiver_id: TransceiverId) -> Self {
        match transceiver_id {
            TransceiverId::Local => Endpoint::LOCAL,
            TransceiverId::Remote(endpoint) => endpoint,
            TransceiverId::Dif(_) => Endpoint::LOCAL, // DIF clients are considered local for this conversion
        }
    }
}

impl Serialize for TransceiverId {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match self {
            TransceiverId::Local => serializer.serialize_str("local"),
            TransceiverId::Remote(endpoint) => {
                let endpoint_str = endpoint.to_string();
                serializer.serialize_str(&endpoint_str)
            }
            TransceiverId::Dif(id) => serializer.serialize_u8(*id),
        }
    }
}

impl<'de> Deserialize<'de> for TransceiverId {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        struct TransceiverIdVisitor;

        impl<'de> Visitor<'de> for TransceiverIdVisitor {
            type Value = TransceiverId;

            fn expecting(
                &self,
                formatter: &mut core::fmt::Formatter,
            ) -> core::fmt::Result {
                formatter.write_str("a valid transceiver id")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                Ok(TransceiverId::Dif(v.to_u8().ok_or_else(|| {
                    DeError::custom(format!(
                        "value {v} is too large to fit into u8"
                    ))
                })?))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if v == "local" {
                    Ok(TransceiverId::Local)
                } else {
                    Ok(TransceiverId::Remote(
                        Endpoint::from_str(v).map_err(Error::custom)?,
                    ))
                }
            }
        }
        deserializer.deserialize_any(TransceiverIdVisitor)
    }
}

impl Display for TransceiverId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TransceiverId::Local => write!(f, "local"),
            TransceiverId::Remote(endpoint) => write!(f, "{}", endpoint),
            TransceiverId::Dif(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct ObserveOptions {
    /// If true, the transceiver will be notified of changes that originated from itself
    pub relay_own_updates: bool,
}

impl<'de> DeserializeSeed<'de> for SerdeContext<'de, ObserveOptions> {
    type Value = ObserveOptions;

    fn deserialize<D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        ObserveOptions::deserialize(deserializer)
    }
}

impl<'ctx> SerializeSeed for SerdeContext<'ctx, ObserveOptions> {
    type Value = ObserveOptions;

    fn serialize<S: Serializer>(
        &mut self,
        value: &Self::Value,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.serialize(serializer)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
#[repr(transparent)]
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

impl Debug for Observer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Observer")
            .field("transceiver_id", &self.transceiver_id)
            .field("options", &self.options)
            .finish()
    }
}

impl Observer {
    /// Creates a new local observer with the given callback function,
    /// using default options and a transceiver ID of 0.
    pub fn new<F: Fn(&Update) + 'static>(callback: F) -> Self {
        Observer {
            transceiver_id: TransceiverId::Local,
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
            return Err(ObserverError::ImmutableValue);
        }
        Ok(())
    }

    /// Notifies all observers of a change represented by the given [Update].
    pub fn get_current_observers(
        &self,
        source_id: &TransceiverId,
    ) -> Vec<ObserverCallback> {
        self.observers
            .iter()
            .filter(|(_, observer)| {
                // Filter out bounced back transceiver updates if relay_own_updates not enabled
                observer.options.relay_own_updates
                    || &observer.transceiver_id != source_id
            })
            .map(|(_, f)| f.callback.clone())
            .collect()
    }

    /// Check if there are any observers registered
    pub fn has_observers(&self) -> bool {
        !self.observers.is_empty()
    }

    /// Calls all observers with the given update.
    pub fn call_observers(&self, update: &Update) {
        for observer in self.get_current_observers(&update.source_id) {
            observer(update);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::*,
        runtime::cache::shared_references_cache::SharedReferencesCache,
        shared_values::{
            SharedContainerMutability,
            base_shared_value_container::{
                BaseSharedValueContainer,
                observers::{
                    ObserveOptions, Observer, ObserverError, ObserverId,
                    TransceiverId,
                },
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
            value_container::{ValueContainer, value_key::ValueKey},
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
                        source_id: update.source_id.clone(),
                        data: update.data.clone(),
                    });
                }),
            })
            .expect("Failed to attach observer");
        update_collector
    }

    #[test]
    fn immutable_reference_observe_fails() {
        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Immutable,
        );
        assert_matches!(
            r.observe(Observer::new(|_| {})),
            Err(ObserverError::ImmutableValue)
        );

        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Mutable,
        );
        assert_matches!(r.observe(Observer::new(|_| {})), Ok(_));
    }

    #[test]
    fn observe_and_unobserve() {
        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Mutable,
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
        let mut r = BaseSharedValueContainer::new_with_inferred_allowed_type(
            42,
            SharedContainerMutability::Mutable,
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
}
