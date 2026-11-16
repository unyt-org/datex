use crate::types::type_definition_with_metadata::metadata::TypeMetadata;

use crate::{
    types::traits::type_match::{TypeSatisfiesValueContainer, TypeSuperset},
    values::value_container::ValueContainer,
};

impl TypeSuperset<TypeMetadata> for TypeMetadata {
    fn is_superset_of(&self, other: &Self) -> bool {
        match (self, other) {
            (
                TypeMetadata::Local {
                    mutability: mutability1,
                    ownership: reference_mutability1,
                },
                TypeMetadata::Local {
                    mutability: mutability2,
                    ownership: reference_mutability2,
                },
            ) => {
                mutability1 == mutability2
                    && reference_mutability1 == reference_mutability2
            }
            (
                TypeMetadata::Shared {
                    mutability: mutability1,
                    ownership: ownership1,
                },
                TypeMetadata::Shared {
                    mutability: mutability2,
                    ownership: ownership2,
                },
            ) => mutability1 == mutability2 && ownership1 == ownership2,
            _ => false,
        }
    }
}

impl TypeSatisfiesValueContainer for TypeMetadata {
    fn satisfies_value_container(&self, _value: &ValueContainer) -> bool {
        unimplemented!()
    }
}

// type y = [text, integer];
// var x = [text, integer];
// x matches y --> false
// ['xxx', 3] matches y --> true
// y matches x --> false
// integer/u8 matches integer --> true
// 10u8 matches integer/u8 --> true
// 10u8 matches integer --> ?
// 10u8 matches 10u8 --> true

// var y: integer[] = [4]
// procedure(x: 4 | 5) { ... }

// prodedure(y[0]) // <--- y[0] : integer !!!! Incorrect function agrs for 4 | 5, can not pass integer
// if y[0] matches 4 | 5 then prodedure(y[0]) // <--- y[0] : 4 | 5 | 7, can pass 4 | 5
