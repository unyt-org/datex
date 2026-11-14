use crate::traits::{structural_eq::StructuralEq, value_eq::ValueEq};

impl ValueEq for super::ValueClassification {
    fn value_eq(&self, other: &Self) -> bool {
        self.entity_type == other.entity_type
            && self.impls == other.impls
            && self.tag == other.tag
    }
}

impl StructuralEq for super::ValueClassification {
    fn structural_eq(&self, other: &Self) -> bool {
        // TODO is it different for tags and entities?
        // MyStruct::VariantA(1, 2) == MyStruct::VariantB(1, 2) ?
        // [1,2,3] + $123 == [1,2,3] + $456 ?

        self.value_eq(other)
    }
}
