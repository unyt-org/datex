use crate::{
    core_compiler::value_compiler::append_regular_instruction,
    global::protocol_structures::{
        instruction_data::ConfirmMoves,
        regular_instructions::RegularInstruction,
    },
    prelude::*,
    shared_values::SelfOwnedPointerAddress,
};
use binrw::io::Cursor;

/// Compiles a CONFIRM_MOVES instruction with a list of pointer mappings
pub fn compile_request_moves(
    mappings: Vec<(SelfOwnedPointerAddress, SelfOwnedPointerAddress)>,
) -> Vec<u8> {
    let mut cursor =
        Cursor::new(Vec::with_capacity(1 + 5 + (mappings.len() * 2 * 5)));

    append_regular_instruction(
        &mut cursor,
        RegularInstruction::ConfirmMoves(ConfirmMoves {
            pointer_count: mappings.len() as u32,
            address_mappings: mappings,
        }),
    );

    cursor.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global::instruction_codes::InstructionCode;

    #[test]
    fn compile_request_empty_move() {
        assert_eq!(
            compile_request_moves(vec![]),
            vec![InstructionCode::CONFIRM_MOVES as u8, 0, 0, 0, 0]
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
            compile_request_moves(mappings),
            vec![
                InstructionCode::CONFIRM_MOVES as u8,
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
