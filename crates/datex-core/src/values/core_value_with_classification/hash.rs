use core::hash::{Hash, Hasher};

use crate::values::core_value_with_classification::CoreValueWithClassification;

impl Hash for CoreValueWithClassification {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
        self.classification.hash(state);
    }
}
