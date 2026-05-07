use crate::{
    core_compiler::value_compiler::{
        append_instruction_code_new, append_local_pointer_address,
    },
    global::{
        instruction_codes::InstructionCode,
        protocol_structures::instruction_data::RawLocalPointerAddress,
    },
    prelude::*,
    utils::buffers::append_u32,
};
use binrw::io::Cursor;

/// Compiles a MOVE instruction with a list of pointer mappings
pub fn compile_request_move(
    mappings: &[(RawLocalPointerAddress, RawLocalPointerAddress)],
) -> Vec<u8> {
    let mut cursor =
        Cursor::new(Vec::with_capacity(1 + 5 + (mappings.len() * 2 * 5)));

    append_instruction_code_new(&mut cursor, InstructionCode::MOVE);
    // number of pointer mappings
    append_u32(&mut cursor, mappings.len() as u32);

    for (original_address, new_address) in mappings {
        append_local_pointer_address(&mut cursor, original_address.bytes);
        append_local_pointer_address(&mut cursor, new_address.bytes);
    }

    cursor.into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_request_empty_move() {
        assert_eq!(
            compile_request_move(&[]),
            vec![InstructionCode::MOVE as u8, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_compile_request_move() {
        let mappings = &[
            (
                RawLocalPointerAddress {
                    bytes: [1, 1, 1, 1, 1],
                },
                RawLocalPointerAddress {
                    bytes: [1, 2, 3, 4, 5],
                },
            ),
            (
                RawLocalPointerAddress {
                    bytes: [2, 2, 2, 2, 2],
                },
                RawLocalPointerAddress {
                    bytes: [1, 2, 3, 4, 6],
                },
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
