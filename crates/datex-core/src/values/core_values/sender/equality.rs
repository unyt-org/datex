use crate::{
    traits::structural_eq::StructuralEq,
};
use crate::values::core_values::sender::Sender;

impl<T> PartialEq for Sender<T> {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl<T> Eq for Sender<T> {}

impl<T> StructuralEq for Sender<T> {
    fn structural_eq(&self, other: &Self) -> bool {
        false
    }
}