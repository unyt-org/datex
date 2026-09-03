use std::hash::{Hash, Hasher};
use crate::collections::default_hasher;
use crate::preludes::derive::CoreValue;
use crate::traits::datex_hash::{impl_datex_hash, DatexHash};

impl DatexHash for CoreValue {
    fn datex_hash(&self) -> u64 {
        match self {
            CoreValue::Uninitialized => state.write_u8(0),
            CoreValue::Null => state.write_u8(1),
            CoreValue::Boolean(b) => b.hash(&mut state),
            CoreValue::Integer(i) => i.hash(&mut state),
            CoreValue::TypedInteger(ti) => ti.hash(&mut state),
            CoreValue::Decimal(d) => d.hash(&mut state),
            CoreValue::TypedDecimal(td) => td.hash(&mut state),
            CoreValue::Text(t) => t.hash(&mut state),
            CoreValue::Endpoint(e) => e.hash(&mut state),
            CoreValue::List(l) => l.hash(&mut state),
            CoreValue::Map(m) => m.datex_hash(&mut state),
            CoreValue::Type(t) => t.datex_hash(),
            CoreValue::EntityTypeDefinition(etd) => etd.datex_hash(),
            CoreValue::Callable(c) => c.datex_hash(),
            CoreValue::Range(r) => r.datex_hash(),
            CoreValue::Box(b) => b.datex_hash(),
            CoreValue::Native(native) => native.datex_hash(),
        }
    }
}