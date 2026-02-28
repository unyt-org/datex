use crate::values::pointer::PointerAddress;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pointer {
    address: PointerAddress,
    // TODO: additional fields will probably be added later, e.g. previous owners
    // subscribers: Vec<(Endpoint, Permissions)>,
}

impl Pointer {
    pub const NULL: Pointer = Pointer {
        address: PointerAddress::NULL,
    };

    pub fn address(&self) -> &PointerAddress {
        &self.address
    }

    pub(crate) fn new(address: PointerAddress) -> Self {
        Pointer { address }
    }
}