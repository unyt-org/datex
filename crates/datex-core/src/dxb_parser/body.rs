use crate::{
    dxb_parser::next_instructions_stack::{
        NextInstructionType, NextInstructionsStack,
        NotInUnboundedRegularScopeError,
    },
    global::{
        protocol_structures::instruction_data::{ UInt8Data },
    },
    runtime::execution::macros::yield_unwrap,
};

use crate::prelude::*;
use alloc::string::FromUtf8Error;
use binrw::{BinRead, io::Cursor};
use core::{
    cell::RefCell, convert::TryFrom, fmt, fmt::Display, result::Result,
};
use crate::global::protocol_structures::instructions::{Instruction, NestedInstructionResolutionStrategy};
use crate::global::protocol_structures::regular_instructions::{RegularInstruction};
use crate::global::protocol_structures::type_instructions::TypeInstruction;

#[derive(Debug)]
pub enum DXBParserError {
    InvalidEndpoint(String),
    InvalidBinaryCode(u8),
    FailedToReadInstructionCode,
    InvalidInstructionCode(u8),
    /// Returned when the end of the DXB body is reached, but further instructions are expected.
    ExpectingMoreInstructions,
    UnexpectedBytesAfterEndOfInstructions,
    FmtError(fmt::Error),
    BinRwError(binrw::Error),
    FromUtf8Error(FromUtf8Error),
    NotInUnboundedRegularScopeError,
    InvalidInternalSlotAddress(u32),
}

// custom impl required because binrw::Error does not implement PartialEq
impl PartialEq for DXBParserError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DXBParserError::InvalidEndpoint(a), DXBParserError::InvalidEndpoint(b)) => a == b,
            (DXBParserError::InvalidBinaryCode(a), DXBParserError::InvalidBinaryCode(b)) => a == b,
            (DXBParserError::FailedToReadInstructionCode, DXBParserError::FailedToReadInstructionCode) => true,
            (DXBParserError::InvalidInstructionCode(a), DXBParserError::InvalidInstructionCode(b)) => a == b,
            (DXBParserError::ExpectingMoreInstructions, DXBParserError::ExpectingMoreInstructions) => true,
            (DXBParserError::UnexpectedBytesAfterEndOfInstructions, DXBParserError::UnexpectedBytesAfterEndOfInstructions) => true,
            (DXBParserError::FmtError(a), DXBParserError::FmtError(b)) => a.to_string() == b.to_string(),
            (DXBParserError::BinRwError(a), DXBParserError::BinRwError(b)) => a.to_string() == b.to_string(),
            (DXBParserError::FromUtf8Error(a), DXBParserError::FromUtf8Error(b)) => a.to_string() == b.to_string(),
            (DXBParserError::NotInUnboundedRegularScopeError, DXBParserError::NotInUnboundedRegularScopeError) => true,
            (DXBParserError::InvalidInternalSlotAddress(a), DXBParserError::InvalidInternalSlotAddress(b)) => a == b,
            _ => false,
        }
    }
}

impl From<fmt::Error> for DXBParserError {
    fn from(error: fmt::Error) -> Self {
        DXBParserError::FmtError(error)
    }
}

impl From<binrw::Error> for DXBParserError {
    fn from(error: binrw::Error) -> Self {
        DXBParserError::BinRwError(error)
    }
}

impl From<FromUtf8Error> for DXBParserError {
    fn from(error: FromUtf8Error) -> Self {
        DXBParserError::FromUtf8Error(error)
    }
}

impl From<NotInUnboundedRegularScopeError> for DXBParserError {
    fn from(_: NotInUnboundedRegularScopeError) -> Self {
        DXBParserError::NotInUnboundedRegularScopeError
    }
}

impl Display for DXBParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DXBParserError::InvalidBinaryCode(code) => {
                core::write!(f, "Invalid binary code: {code}")
            }
            DXBParserError::InvalidEndpoint(endpoint) => {
                core::write!(f, "Invalid endpoint: {endpoint}")
            }
            DXBParserError::FailedToReadInstructionCode => {
                core::write!(f, "Failed to read instruction code")
            }
            DXBParserError::InvalidInstructionCode(code) => {
                core::write!(
                    f,
                    "Encountered an invalid instruction code: {:2X}",
                    code
                )
            }
            DXBParserError::FmtError(err) => {
                core::write!(f, "Formatting error: {err}")
            }
            DXBParserError::BinRwError(err) => {
                core::write!(f, "Binary read/write error: {err}")
            }
            DXBParserError::FromUtf8Error(err) => {
                core::write!(f, "UTF-8 conversion error: {err}")
            }
            DXBParserError::ExpectingMoreInstructions => {
                core::write!(f, "Expecting more instructions")
            }
            DXBParserError::UnexpectedBytesAfterEndOfInstructions => {
                core::write!(f, "Unexpected bytes after end of instructions")
            }
            DXBParserError::NotInUnboundedRegularScopeError => {
                core::write!(f, "Not in unbounded regular scope error")
            }
            DXBParserError::InvalidInternalSlotAddress(addr) => {
                core::write!(f, "Invalid internal slot address: {}", addr)
            }
        }
    }
}

// TODO #676: we must ensure while an execution for a block runs, no other executions run using the same next_instructions_stack - maybe also find a solution without Rc<RefCell>
pub fn iterate_instructions(
    dxb_body_ref: Rc<RefCell<Vec<u8>>>,
    nested_instruction_resolution_strategy: NestedInstructionResolutionStrategy,
) -> impl Iterator<Item = Result<Instruction, DXBParserError>> {
    gen move {
        // create a stack to track next instructions
        let mut next_instructions_stack = NextInstructionsStack::default();

        // get reader for dxb_body
        let mut dxb_body = core::mem::take(&mut *dxb_body_ref.borrow_mut());
        let mut len = dxb_body.len();
        let mut reader = Cursor::new(dxb_body);

        loop {
            // if cursor is at the end, check if more instructions are expected, else end iteration
            if reader.position() as usize >= len {
                // indicates that more instructions need to be read
                if !next_instructions_stack.is_end() {
                    yield Err(DXBParserError::ExpectingMoreInstructions);
                    // assume that more instructions are loaded into dxb_body externally after this yield
                    // so we just reload the dxb_body from the Rc<RefCell>
                    dxb_body = core::mem::take(&mut *dxb_body_ref.borrow_mut());
                    len = dxb_body.len();
                    reader = Cursor::new(dxb_body);
                    continue;
                }
                return;
            }

            let next_instruction_type = next_instructions_stack.pop();

            // parse instruction based on its type
            let instruction = match next_instruction_type {
                NextInstructionType::End => {
                    // if cursor
                    if len > reader.position() as usize {
                        yield Err(DXBParserError::UnexpectedBytesAfterEndOfInstructions);
                    }
                    return
                }, // end of instructions

                NextInstructionType::Regular => {
                    let instruction = yield_unwrap!(RegularInstruction::read(&mut reader));
                    let instruction = if let RegularInstruction::RemoteExecution(instruction_block_data) = instruction {
                        match nested_instruction_resolution_strategy {
                            #[cfg(feature = "disassembler")]
                            NestedInstructionResolutionStrategy::ResolveNestedScopesFlat | NestedInstructionResolutionStrategy::ResolveNestedScopesTree => {
                                use crate::global::protocol_structures::instruction_data::{InstructionBlockDataDebugFlat, InstructionBlockDataDebugTree};

                                let (inner_instructions, err) = crate::disassembler::disassemble_body(
                                    &instruction_block_data.body,
                                    nested_instruction_resolution_strategy
                                );

                                if let Some(err) = err {
                                    return yield Err(err);
                                }
                                if nested_instruction_resolution_strategy == NestedInstructionResolutionStrategy::ResolveNestedScopesFlat {
                                    RegularInstruction::_RemoteExecutionDebugFlat(InstructionBlockDataDebugFlat {
                                        length: instruction_block_data.length,
                                        injected_variable_count: instruction_block_data.injected_variable_count,
                                        injected_variables: instruction_block_data.injected_variables,
                                        body: inner_instructions.flatten(),
                                    })
                                }
                                else {
                                    RegularInstruction::_RemoteExecutionDebugTree(InstructionBlockDataDebugTree {
                                        length: instruction_block_data.length,
                                        injected_variable_count: instruction_block_data.injected_variable_count,
                                        injected_variables: instruction_block_data.injected_variables,
                                        body: inner_instructions,
                                    })
                                }
                            }
                            _ => RegularInstruction::RemoteExecution(instruction_block_data)
                        }
                    }
                    else {
                        instruction
                    };


                    yield_unwrap!(next_instructions_stack.handle_next_expected_instructions(
                        instruction.get_next_expected_instructions()
                    ));

                    instruction
                }
                .into(),

                NextInstructionType::Type => {
                    let instruction = yield_unwrap!(TypeInstruction::read(&mut reader));

                    yield_unwrap!(next_instructions_stack.handle_next_expected_instructions(
                        instruction.get_next_expected_instructions()
                    ));

                    instruction
                }
                .into(),
            };

            // println!("instruction {}", instruction);

            yield Ok(instruction);
        }
    }
}

#[cfg(test)]
mod tests {
    use core::assert_matches;
    use crate::global::instruction_codes::InstructionCode;
    use super::*;

    fn iterate_dxb(
        data: Vec<u8>,
    ) -> impl Iterator<Item = Result<Instruction, DXBParserError>> {
        iterate_instructions(Rc::new(RefCell::new(data)), NestedInstructionResolutionStrategy::default())
    }

    #[test]
    fn invalid_instruction_code() {
        let data = vec![0xFF]; // Invalid instruction code
        let mut iterator = iterate_dxb(data);
        let result = iterator.next().unwrap();
        assert_matches!(
            result,
            Err(err @ DXBParserError::BinRwError(_)) if err.to_string().contains("invalid instruction code")
        );
    }

    #[test]
    fn empty_expect_more_instructions() {
        let data = vec![]; // Empty data
        let mut iterator = iterate_dxb(data);
        let result = iterator.next().unwrap();
        assert_matches!(
            result,
            Err(DXBParserError::ExpectingMoreInstructions)
        );
    }

    #[test]
    fn valid_uint8_instruction() {
        let data = vec![InstructionCode::UINT_8 as u8, 42];
        let mut iterator = iterate_dxb(data);
        let result = iterator.next().unwrap();
        match result {
            Ok(Instruction::Regular(RegularInstruction::UInt8(
                value,
            ))) => {
                assert_eq!(value.0, 42);
            }
            _ => panic!("Expected UINT_8 instruction"),
        }
        // Ensure no more instructions
        assert!(iterator.next().is_none());
    }

    #[test]
    fn valid_short_text_instruction() {
        let text = "Hello";
        let text_bytes = text.as_bytes();
        let mut data =
            vec![InstructionCode::SHORT_TEXT as u8, text_bytes.len() as u8];
        data.extend_from_slice(text_bytes);
        let mut iterator = iterate_dxb(data);
        let result = iterator.next().unwrap();
        match result {
            Ok(Instruction::Regular(
                RegularInstruction::ShortText(value),
            )) => {
                assert_eq!(value.0, "Hello");
            }
            _ => panic!("Expected SHORT_TEXT instruction"),
        }
        // Ensure no more instructions
        assert!(iterator.next().is_none());
    }

    #[test]
    fn valid_add_instruction() {
        let data = vec![
            InstructionCode::ADD as u8,
            // first operand (UINT_8)
            InstructionCode::UINT_8 as u8,
            10,
            // second operand (UINT_8)
            InstructionCode::UINT_8 as u8,
            20,
        ];
        let mut iterator = iterate_dxb(data);
        // first instruction should be ADD
        assert!(matches!(
            iterator.next().unwrap(),
            Ok(Instruction::Regular(RegularInstruction::Add))
        ));
        // next instruction should be first UINT_8
        assert!(matches!(
            iterator.next().unwrap(),
            Ok(Instruction::Regular(RegularInstruction::UInt8(
                UInt8Data(10)
            )))
        ));
        // next instruction should be second UINT_8
        assert!(matches!(
            iterator.next().unwrap(),
            Ok(Instruction::Regular(RegularInstruction::UInt8(
                UInt8Data(20)
            )))
        ));
        // ensure no more instructions
        assert!(iterator.next().is_none());
    }

    #[test]
    fn error_for_partial_instruction() {
        let data = vec![InstructionCode::UINT_16 as u8, 0x34]; // Incomplete UINT_16 data
        let mut iterator = iterate_dxb(data);
        let result = iterator.next().unwrap();
        assert!(matches!(result, Err(DXBParserError::BinRwError(_))));
    }

    #[test]
    fn expect_more_instructions_after_partial() {
        let data = vec![InstructionCode::LIST as u8, 0x02, 0x00, 0x00, 0x00]; // LIST with 2 elements but no elements provided
        let data_ref = Rc::new(RefCell::new(data));
        let mut iterator = iterate_instructions(data_ref.clone(), NestedInstructionResolutionStrategy::default());
        // first instruction should be LIST
        let result = iterator.next().unwrap();
        assert!(matches!(
            result,
            Ok(Instruction::Regular(RegularInstruction::List(_)))
        ));
        // next instruction should error expecting more instructions
        let result = iterator.next().unwrap();
        assert!(matches!(
            result,
            Err(DXBParserError::ExpectingMoreInstructions)
        ));

        // now provide more data for the two elements
        let new_data = vec![
            InstructionCode::UINT_8 as u8, // first element
            10,
            InstructionCode::UINT_8 as u8, // second element
            20,
        ];

        *data_ref.borrow_mut() = new_data;

        // next instruction should be first UINT_8
        let result = iterator.next().unwrap();
        assert!(matches!(
            result,
            Ok(Instruction::Regular(RegularInstruction::UInt8(
                _
            )))
        ));
        // next instruction should be second UINT_8
        let result = iterator.next().unwrap();
        assert!(matches!(
            result,
            Ok(Instruction::Regular(RegularInstruction::UInt8(
                _
            )))
        ));
        // ensure no more instructions
        assert!(iterator.next().is_none());
    }

    #[test]
    fn unbounded_expect_more_instructions() {
        let data = vec![InstructionCode::UNBOUNDED_STATEMENTS as u8]; // Start unbounded statements
        let data_ref = Rc::new(RefCell::new(data));
        let mut iterator = iterate_instructions(data_ref.clone(), NestedInstructionResolutionStrategy::default());
        // first instruction should be UNBOUNDED_STATEMENTS
        let result = iterator.next().unwrap();
        assert!(matches!(
            result,
            Ok(Instruction::Regular(
                RegularInstruction::UnboundedStatements
            ))
        ));
        // next instruction should error expecting more instructions
        let result = iterator.next().unwrap();
        assert!(matches!(
            result,
            Err(DXBParserError::ExpectingMoreInstructions)
        ));

        // now provide more data for the statements
        let new_data = vec![
            InstructionCode::UINT_8 as u8, // first statement
            42,
            InstructionCode::UNBOUNDED_STATEMENTS_END as u8, // end unbounded statements
            0x00,
        ];

        *data_ref.borrow_mut() = new_data;

        // next instruction should be first UINT_8
        let result = iterator.next().unwrap();
        assert!(matches!(
            result,
            Ok(Instruction::Regular(RegularInstruction::UInt8(
                _
            )))
        ));
        // next instruction should be UNBOUNDED_STATEMENTS_END
        let result = iterator.next().unwrap();
        assert!(matches!(
            result,
            Ok(Instruction::Regular(
                RegularInstruction::UnboundedStatementsEnd(_)
            ))
        ));
        // ensure no more instructions
        assert!(iterator.next().is_none());
    }
}
