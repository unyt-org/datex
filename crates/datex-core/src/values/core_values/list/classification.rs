use crate::{
    preludes::derive::List,
    traits::{
        classification::Classification,
        static_classification::StaticClassification,
    },
};

impl Classification for List {}
impl StaticClassification for List {}
