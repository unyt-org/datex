use crate::{
    traits::structural_eq::StructuralEq,
};
use crate::values::core_values::receiver::Receiver;

impl<T> PartialEq for Receiver<T> {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl<T> Eq for Receiver<T> {}

impl<T> StructuralEq for Receiver<T> {
    fn structural_eq(&self, other: &Self) -> bool {
        false
    }
}