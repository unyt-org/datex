use crate::{
    traits::structural_eq::StructuralEq,
    values::{
        core_value::CoreValue,
        core_values::map::{BorrowedMapKey, Map},
        value::Value,
        value_container::ValueContainer,
    },
};
use core::hash::{Hash, Hasher};

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl Eq for Map {}

impl Hash for Map {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (k, v) in self.iter() {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl StructuralEq for BorrowedMapKey<'_> {
    fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (BorrowedMapKey::Text(a), BorrowedMapKey::Text(b)) => a == b,
            (BorrowedMapKey::Value(a), BorrowedMapKey::Value(b)) => {
                a.structural_eq(b)
            }
            (BorrowedMapKey::Text(a), BorrowedMapKey::Value(b))
            | (BorrowedMapKey::Value(b), BorrowedMapKey::Text(a)) => {
                if let ValueContainer::Local(Value {
                    inner: CoreValue::Text(text),
                    ..
                }) = b
                {
                    a == &text.0
                } else {
                    false
                }
            }
        }
    }
}

impl StructuralEq for Map {
    fn structural_eq(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }
        for ((key, value), (other_key, other_value)) in
            self.iter().zip(other.iter())
        {
            if !key.structural_eq(&other_key)
                || !value.structural_eq(other_value)
            {
                return false;
            }
        }
        true
    }
}
