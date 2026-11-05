use crate::{
    traits::structural_eq::StructuralEq,
    values::core_values::receiver::Receiver,
};

impl<T> PartialEq for Receiver<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl<T> Eq for Receiver<T> {}

impl<T> StructuralEq for Receiver<T> {
    fn structural_eq(&self, _other: &Self) -> bool {
        false
    }
}
