use crate::{
    collections::HashMap,
    core_compiler::value_compiler::{
        append_instruction_code,
    },
    global::instruction_codes::InstructionCode,
    runtime::execution::context::ExecutionMode,
    utils::buffers::append_u32,
    values::value_container::ValueContainer,
};

use crate::prelude::*;
use core::cmp::PartialEq;
use core::hash::{Hash, Hasher};
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedSlotType {
    // shared x
    Move,
    // 'shared x
    Ref,
    // 'mut shared mut x
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalSlotType {
    /// The value is moved into the external slot and no longer used afterward
    Move,
    /// The value is moved into the external slot but still used afterward (clone or immutable ref (&x))
    Copy,
    /// The value is temporarily borrowed in the external slot - the changed value must be written back to the local slot afterward
    RefMut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExternalSlotType {
    Local(LocalSlotType),
    Shared(SharedSlotType),
}

impl From<ExternalSlotType> for u8 {
    fn from(slot_type: ExternalSlotType) -> Self {
        match slot_type {
            ExternalSlotType::Local(local_type) => match local_type {
                LocalSlotType::Move => 0,
                LocalSlotType::Copy => 1,
                LocalSlotType::RefMut => 2,
            },
            ExternalSlotType::Shared(shared_type) => match shared_type {
                SharedSlotType::Move => 3,
                SharedSlotType::Ref => 4,
                SharedSlotType::RefMut => 5,
            },
        }
    }
}

impl TryFrom<u8> for ExternalSlotType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ExternalSlotType::Local(LocalSlotType::Move)),
            1 => Ok(ExternalSlotType::Local(LocalSlotType::Copy)),
            2 => Ok(ExternalSlotType::Local(LocalSlotType::RefMut)),
            3 => Ok(ExternalSlotType::Shared(SharedSlotType::Move)),
            4 => Ok(ExternalSlotType::Shared(SharedSlotType::Ref)),
            5 => Ok(ExternalSlotType::Shared(SharedSlotType::RefMut)),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct VirtualSlot {
    /// parent scope level if exists, otherwise 0
    pub level: u8,
    /// local slot address of scope with level
    pub virtual_address: u32,
    pub external_slot_type: Option<ExternalSlotType>
}

impl Hash for VirtualSlot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.level.hash(state);
        self.virtual_address.hash(state);
    }
}

impl PartialEq for VirtualSlot {
    fn eq(&self, other: &Self) -> bool {
        self.level == other.level && self.virtual_address == other.virtual_address
    }
}

impl VirtualSlot {
    pub fn local(virtual_address: u32) -> Self {
        VirtualSlot {
            level: 0,
            virtual_address,
            external_slot_type: None,
        }
    }
    pub fn is_external(&self) -> bool {
        let is_external = self.level > 0;
        if is_external && self.external_slot_type.is_none() {
            unreachable!()
        }
        is_external
    }

    pub fn external(level: u8, virtual_address: u32, external_slot_type: ExternalSlotType) -> Self {
        VirtualSlot {
            level,
            virtual_address,
            external_slot_type: Some(external_slot_type),
        }
    }

    pub fn downgrade(&self, external_slot_type: ExternalSlotType) -> Self {
        VirtualSlot {
            level: self.level + 1,
            virtual_address: self.virtual_address,
            external_slot_type: Some(external_slot_type),
        }
    }

    pub fn upgrade(&self) -> Self {
        if self.level > 0 {
            VirtualSlot {
                level: self.level - 1,
                virtual_address: self.virtual_address,
                external_slot_type: self.external_slot_type,
            }
        } else {
            core::panic!("Cannot upgrade a local slot");
        }
    }
}

/// compilation context, created for each compiler call, even if compiling a script for the same scope
pub struct CompilationContext {
    pub inserted_value_index: usize,
    pub buffer: Vec<u8>,
    pub inserted_values: Vec<Option<ValueContainer>>,
    /// this flag is set to true if any non-static value is encountered
    pub has_non_static_value: bool,
    pub execution_mode: ExecutionMode,

    // mapping for temporary scope slot resolution
    slot_indices: HashMap<VirtualSlot, Vec<u32>>,
}

impl CompilationContext {
    const MAX_INT_32: i64 = 2_147_483_647;
    const MIN_INT_32: i64 = -2_147_483_648;

    const MAX_INT_8: i64 = 127;
    const MIN_INT_8: i64 = -128;

    const MAX_INT_16: i64 = 32_767;
    const MIN_INT_16: i64 = -32_768;

    const MAX_UINT_16: i64 = 65_535;

    const INT_8_BYTES: u8 = 1;
    const INT_16_BYTES: u8 = 2;
    const INT_32_BYTES: u8 = 4;
    const INT_64_BYTES: u8 = 8;
    const INT_128_BYTES: u8 = 16;

    const FLOAT_32_BYTES: u8 = 4;
    const FLOAT_64_BYTES: u8 = 8;

    pub fn new(
        buffer: Vec<u8>,
        inserted_values: Vec<Option<ValueContainer>>,
        execution_mode: ExecutionMode,
    ) -> Self {
        CompilationContext {
            inserted_value_index: 0,
            buffer,
            inserted_values,
            has_non_static_value: false,
            slot_indices: HashMap::new(),
            execution_mode,
        }
    }

    pub fn buffer_index(&self) -> usize {
        self.buffer.len()
    }

    pub fn external_slots(&self) -> Vec<VirtualSlot> {
        self.slot_indices
            .iter()
            .filter(|(slot, _)| slot.is_external())
            .sorted_by(|a, b| a.0.virtual_address.cmp(&b.0.virtual_address))
            .map(|(slot, _)| *slot)
            .collect()
    }

    /// Gets all slots for either local or external slots depending on the value of external
    pub fn get_slot_byte_indices(
        &self,
        match_externals: bool,
    ) -> Vec<Vec<u32>> {
        self.slot_indices
            .iter()
            .filter(|(slot, _)| slot.is_external() == match_externals)
            .sorted_by(|a, b| a.0.virtual_address.cmp(&b.0.virtual_address))
            .map(|(_, indices)| indices.clone())
            .collect()
    }

    pub fn remap_virtual_slots(&mut self) {
        let mut slot_address = 0;

        // parent slots
        for byte_indices in self.get_slot_byte_indices(true) {
            for byte_index in byte_indices {
                self.set_u32_at_index(slot_address, byte_index as usize);
            }
            slot_address += 1;
        }

        // local slots
        for byte_indices in self.get_slot_byte_indices(false) {
            for byte_index in byte_indices {
                self.set_u32_at_index(slot_address, byte_index as usize);
            }
            slot_address += 1;
        }
    }

    // This method writes a placeholder value for the slot
    // since the slot address is not known yet and just temporary.
    pub fn insert_virtual_slot_address(&mut self, virtual_slot: VirtualSlot) {
        let buffer_index = self.buffer_index() as u32;
        if let Some(indices) = self.slot_indices.get_mut(&virtual_slot) {
            indices.push(buffer_index);
        } else {
            self.slot_indices.insert(virtual_slot, vec![buffer_index]);
        }
        append_u32(&mut self.buffer, 0); // placeholder for the slot address
    }

    pub fn set_u32_at_index(&mut self, u32: u32, index: usize) {
        self.buffer[index..index + CompilationContext::INT_32_BYTES as usize]
            .copy_from_slice(&u32.to_le_bytes());
    }

    pub fn mark_has_non_static_value(&mut self) {
        self.has_non_static_value = true;
    }

    pub fn append_instruction_code(&mut self, code: InstructionCode) {
        append_instruction_code(&mut self.buffer, code);
    }
}
