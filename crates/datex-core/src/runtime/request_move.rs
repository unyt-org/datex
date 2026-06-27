use binrw::BinWrite;
use crate::{
    core_compiler::value_compiler::{
        append_instruction_code, append_local_pointer_address,
    },
    global::{
        instruction_codes::InstructionCode,
    },
    prelude::*,
    utils::buffers::append_u32,
};
use binrw::io::Cursor;
use crate::core_compiler::value_compiler::append_regular_instruction;
use crate::global::protocol_structures::instruction_data::Move;
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::shared_values::SelfOwnedPointerAddress;

/// Compiles a MOVE instruction with a list of pointer mappings
pub fn compile_request_move(
    mappings: Vec<(SelfOwnedPointerAddress, SelfOwnedPointerAddress)>,
) -> Vec<u8> {
    let mut cursor =
        Cursor::new(Vec::with_capacity(1 + 5 + (mappings.len() * 2 * 5)));
    
    append_regular_instruction(&mut cursor, RegularInstruction::Move(Move {
        pointer_count: mappings.len() as u32,
        address_mappings: mappings,
    }));

    cursor.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_request_empty_move() {
        assert_eq!(
            compile_request_move(vec![]),
            vec![InstructionCode::MOVE as u8, 0, 0, 0, 0]
        );
    }

    #[test]
    fn compile_request_move_default() {
        let mappings = vec![
            (
                SelfOwnedPointerAddress::new([1, 1, 1, 1, 1]),
                SelfOwnedPointerAddress::new([1, 2, 3, 4, 5]),
            ),
            (
                SelfOwnedPointerAddress::new([2, 2, 2, 2, 2]),
                SelfOwnedPointerAddress::new([1, 2, 3, 4, 6]),
            ),
        ];
        assert_eq!(
            compile_request_move(mappings),
            vec![
                InstructionCode::MOVE as u8,
                2,
                0,
                0,
                0,
                1,
                1,
                1,
                1,
                1,
                1,
                2,
                3,
                4,
                5,
                2,
                2,
                2,
                2,
                2,
                1,
                2,
                3,
                4,
                6
            ]
        )
    }
}
