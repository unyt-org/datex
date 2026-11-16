use crate::traits::datex_hash::DatexHash;
use core::hash::{Hash, Hasher};
use ordered_float::OrderedFloat;

impl DatexHash for f32 {
    fn datex_hash(&self, mut state: &mut dyn Hasher) {
        let ordered = OrderedFloat::from(*self);
        ordered.hash(&mut state);
    }
}

impl DatexHash for f64 {
    fn datex_hash(&self, mut state: &mut dyn Hasher) {
        let ordered = OrderedFloat::from(*self);
        ordered.hash(&mut state);
    }
}
