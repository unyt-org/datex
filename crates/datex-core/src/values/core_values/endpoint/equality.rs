use core::cmp::Ordering;

use crate::{
    traits::structural_eq::StructuralEq,
    values::core_values::endpoint::{Endpoint, EndpointInstance, EndpointType},
};

impl StructuralEq for Endpoint {
    fn structural_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl PartialOrd for EndpointInstance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EndpointInstance {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (EndpointInstance::Any, EndpointInstance::Any)
            | (EndpointInstance::All, EndpointInstance::All) => Ordering::Equal,
            (EndpointInstance::Any, _) => Ordering::Less,
            (_, EndpointInstance::Any) => Ordering::Greater,
            (EndpointInstance::All, _) => Ordering::Greater,
            (_, EndpointInstance::All) => Ordering::Less,

            (EndpointInstance::Instance(a), EndpointInstance::Instance(b)) => {
                a.cmp(b)
            }
        }
    }
}

impl PartialOrd for EndpointType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EndpointType {
    fn cmp(&self, other: &Self) -> Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl PartialOrd for Endpoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Endpoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ty
            .cmp(&other.ty)
            .then_with(|| self.identifier.cmp(&other.identifier))
            .then_with(|| self.instance.cmp(&other.instance))
    }
}
