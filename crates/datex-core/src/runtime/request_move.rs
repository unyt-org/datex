use crate::core_compiler::value_compiler::{append_instruction_code, append_local_pointer_address};
use crate::global::instruction_codes::InstructionCode;
use crate::global::protocol_structures::instructions::{RawLocalPointerAddress};
use crate::utils::buffers::{append_u32};
use crate::prelude::*;

/// Compiles a MOVE instruction with a list of pointer mappings
pub fn compile_request_move(
    mappings: Vec<(RawLocalPointerAddress, RawLocalPointerAddress)>,
) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1 + 5 + (mappings.len() * 2 * 5));

    append_instruction_code(&mut buffer, InstructionCode::MOVE);
    // number of pointer mappings
    append_u32(&mut buffer, mappings.len() as u32);

    for (original_address, new_address) in mappings {
        append_local_pointer_address(&mut buffer, original_address.id);
        append_local_pointer_address(&mut buffer, new_address.id);
    }

    buffer
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_request_empty_move() {
        assert_eq!(compile_request_move(vec![]), vec![
            InstructionCode::MOVE as u8,
            0,0,0,0
        ]);
    }

    #[test]
    fn test_compile_request_move() {
        let mappings = vec![
            (RawLocalPointerAddress {id: [1,1,1,1,1]}, RawLocalPointerAddress {id: [1,2,3,4,5]}),
            (RawLocalPointerAddress {id: [2,2,2,2,2]}, RawLocalPointerAddress {id: [1,2,3,4,6]}),
        ];
        assert_eq!(compile_request_move(mappings), vec![
            InstructionCode::MOVE as u8,
            2,0,0,0,
            1,1,1,1,1,
            1,2,3,4,5,
            2,2,2,2,2,
            1,2,3,4,6
        ])
    }
}
