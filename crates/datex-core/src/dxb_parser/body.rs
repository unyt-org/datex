use crate::dxb_parser::next_instructions_stack::{
    NextInstructionType, NextInstructionsStack, NotInUnboundedRegularScopeError,
};

use crate::{
    global::protocol_structures::{
        instructions::{
            Instruction, NestedInstructionResolutionStrategy,
            NextExpectedInstructions,
        },
        regular_instructions::RegularInstruction,
        type_instructions::TypeInstruction,
    },
    libs::core::core_lib_id::CoreLibIdIndex,
    prelude::*,
};
use alloc::string::FromUtf8Error;
use binrw::{BinRead, io::Cursor};
use core::{cell::RefCell, fmt, fmt::Display, result::Result};

// This is needed to avoid using "UNBOUNDED_STATEMENTS" and still know the amount of commands
/// A relative program counter request and the number of direct counted
/// children bypassed while moving forward to the destination
#[derive(Default)]
pub struct SeekState {
    offset: Option<i32>,
    skipped_instruction_count: u32,
}

impl SeekState {
    pub fn request(&mut self, offset: i32) {
        self.offset = Some(offset);
    }
    fn take_offset(&mut self) -> Option<i32> {
        self.offset.take()
    }
    pub fn take_skipped_instruction_count(&mut self) -> u32 {
        core::mem::take(&mut self.skipped_instruction_count)
    }
}

pub type SeekRequest = Rc<RefCell<SeekState>>;

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
    InvalidCoreLibId(CoreLibIdIndex),
    InvalidJumpTarget(i64),
}

// custom impl required because binrw::Error does not implement PartialEq
impl PartialEq for DXBParserError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                DXBParserError::InvalidEndpoint(a),
                DXBParserError::InvalidEndpoint(b),
            ) => a == b,
            (
                DXBParserError::InvalidBinaryCode(a),
                DXBParserError::InvalidBinaryCode(b),
            ) => a == b,
            (
                DXBParserError::FailedToReadInstructionCode,
                DXBParserError::FailedToReadInstructionCode,
            ) => true,
            (
                DXBParserError::InvalidInstructionCode(a),
                DXBParserError::InvalidInstructionCode(b),
            ) => a == b,
            (
                DXBParserError::ExpectingMoreInstructions,
                DXBParserError::ExpectingMoreInstructions,
            ) => true,
            (
                DXBParserError::UnexpectedBytesAfterEndOfInstructions,
                DXBParserError::UnexpectedBytesAfterEndOfInstructions,
            ) => true,
            (DXBParserError::FmtError(a), DXBParserError::FmtError(b)) => {
                a.to_string() == b.to_string()
            }
            (DXBParserError::BinRwError(a), DXBParserError::BinRwError(b)) => {
                a.to_string() == b.to_string()
            }
            (
                DXBParserError::FromUtf8Error(a),
                DXBParserError::FromUtf8Error(b),
            ) => a.to_string() == b.to_string(),
            (
                DXBParserError::NotInUnboundedRegularScopeError,
                DXBParserError::NotInUnboundedRegularScopeError,
            ) => true,
            (
                DXBParserError::InvalidJumpTarget(a),
                DXBParserError::InvalidJumpTarget(b),
            ) => a == b,
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
            DXBParserError::InvalidCoreLibId(id) => {
                core::write!(f, "Invalid Core Lib Id: {}", id.0)
            }
            DXBParserError::InvalidJumpTarget(target) => {
                core::write!(
                    f,
                    "Jump target is not a valid instruction boundary: {target}"
                )
            }
        }
    }
}

fn consume_expected_subtrees(
    reader: &mut Cursor<Vec<u8>>,
    expected: NextExpectedInstructions,
) -> Result<(), DXBParserError> {
    match expected {
        NextExpectedInstructions::None => Ok(()),
        NextExpectedInstructions::Regular(count) => {
            for _ in 0..count {
                consume_subtree(reader, NextInstructionType::Regular)?;
            }
            Ok(())
        }
        NextExpectedInstructions::Type(count) => {
            for _ in 0..count {
                consume_subtree(reader, NextInstructionType::Type)?;
            }
            Ok(())
        }
        NextExpectedInstructions::RegularAndType(regular, ty) => {
            for _ in 0..ty {
                consume_subtree(reader, NextInstructionType::Type)?;
            }
            for _ in 0..regular {
                consume_subtree(reader, NextInstructionType::Regular)?;
            }
            Ok(())
        }
        NextExpectedInstructions::UnboundedStart
        | NextExpectedInstructions::UnboundedEnd => {
            Err(DXBParserError::NotInUnboundedRegularScopeError)
        }
    }
}

fn consume_subtree(
    reader: &mut Cursor<Vec<u8>>,
    kind: NextInstructionType,
) -> Result<(), DXBParserError> {
    match kind {
        NextInstructionType::Regular => {
            let instruction = RegularInstruction::read(reader)
                .map_err(DXBParserError::BinRwError)?;
            consume_expected_subtrees(
                reader,
                instruction.get_next_expected_instructions(),
            )
        }
        NextInstructionType::Type => {
            let instruction = TypeInstruction::read(reader)
                .map_err(DXBParserError::BinRwError)?;
            consume_expected_subtrees(
                reader,
                instruction.get_next_expected_instructions(),
            )
        }
        NextInstructionType::End => {
            Err(DXBParserError::UnexpectedBytesAfterEndOfInstructions)
        }
    }
}

fn consume_one_instruction(
    reader: &mut Cursor<Vec<u8>>,
    stack: &mut NextInstructionsStack,
) -> Result<(), DXBParserError> {
    match stack.pop() {
        NextInstructionType::Regular => {
            let instruction = RegularInstruction::read(reader)
                .map_err(DXBParserError::BinRwError)?;
            stack.handle_next_expected_instructions(
                instruction.get_next_expected_instructions(),
            )?;
            Ok(())
        }
        NextInstructionType::Type => {
            let instruction = TypeInstruction::read(reader)
                .map_err(DXBParserError::BinRwError)?;
            stack.handle_next_expected_instructions(
                instruction.get_next_expected_instructions(),
            )?;
            Ok(())
        }
        NextInstructionType::End => {
            Err(DXBParserError::UnexpectedBytesAfterEndOfInstructions)
        }
    }
}

// TODO #676: we must ensure while an execution for a block runs, no other executions run using the same next_instructions_stack - maybe also find a solution without Rc<RefCell>
pub gen fn iterate_instructions(
    dxb_body_ref: Rc<RefCell<Vec<u8>>>,
    nested_instruction_resolution_strategy: NestedInstructionResolutionStrategy,
) -> Result<Instruction, DXBParserError> {
    let mut next_instructions_stack = NextInstructionsStack::default();
    let mut dxb_body = core::mem::take(&mut *dxb_body_ref.borrow_mut());
    let mut len = dxb_body.len();
    let mut reader = Cursor::new(dxb_body);

    loop {
        if reader.position() as usize >= len {
            if !next_instructions_stack.is_end() {
                yield Err(DXBParserError::ExpectingMoreInstructions);

                dxb_body = core::mem::take(&mut *dxb_body_ref.borrow_mut());
                len = dxb_body.len();
                reader = Cursor::new(dxb_body);

                continue;
            }

            return;
        }

        let next_instruction_type = next_instructions_stack.pop();

        let instruction_result: Result<Instruction, DXBParserError> = try {
            match next_instruction_type {
                NextInstructionType::End => {
                    if len > reader.position() as usize {
                        yield Err(
                            DXBParserError::
                                UnexpectedBytesAfterEndOfInstructions,
                        );
                    }

                    return;
                }

                NextInstructionType::Regular => {
                    let instruction = RegularInstruction::read(&mut reader)
                        .map_err(DXBParserError::BinRwError)?;

                    let instruction =
                        if let RegularInstruction::RemoteExecution(
                            instruction_block_data,
                        ) = instruction
                        {
                            match nested_instruction_resolution_strategy {
                                #[cfg(feature = "disassembler")]
                                NestedInstructionResolutionStrategy::
                                    ResolveNestedScopesFlat
                                | NestedInstructionResolutionStrategy::
                                    ResolveNestedScopesTree => {
                                    use crate::global::
                                        protocol_structures::
                                        instruction_data::{
                                            InstructionBlockDataDebugFlat,
                                            InstructionBlockDataDebugTree,
                                        };

                                    let (inner_instructions, err) =
                                        crate::disassembler::disassemble_body(
                                            &instruction_block_data.body,
                                            nested_instruction_resolution_strategy,
                                        );

                                    if let Some(err) = err {
                                        Err(err)?;
                                    }

                                    if nested_instruction_resolution_strategy
                                        == NestedInstructionResolutionStrategy::
                                            ResolveNestedScopesFlat
                                    {
                                        RegularInstruction::
                                            remote_execution_debug_flat(
                                                InstructionBlockDataDebugFlat {
                                                    length:
                                                        instruction_block_data
                                                            .length,
                                                    injected_variable_count:
                                                        instruction_block_data
                                                            .injected_value_count,
                                                    injected_values:
                                                        instruction_block_data
                                                            .injected_values.clone(),
                                                    body: inner_instructions
                                                        .flatten(),
                                                },
                                            )
                                    } else {
                                        RegularInstruction::
                                            remote_execution_debug_tree(
                                                InstructionBlockDataDebugTree {
                                                    length:
                                                        instruction_block_data
                                                            .length,
                                                    injected_variable_count:
                                                        instruction_block_data
                                                            .injected_value_count,
                                                    injected_values:
                                                        instruction_block_data
                                                            .injected_values.clone(),
                                                    body: inner_instructions,
                                                },
                                            )
                                    }
                                }

                                _ => {
                                    RegularInstruction::remote_execution(
                                        instruction_block_data.clone(),
                                    )
                                }
                            }
                        } else {
                            instruction
                        };

                    next_instructions_stack
                        .handle_next_expected_instructions(
                            instruction.get_next_expected_instructions(),
                        )
                        .map_err(|_| {
                            DXBParserError::NotInUnboundedRegularScopeError
                        })?;

                    instruction.into()
                }

                NextInstructionType::Type => {
                    let instruction = TypeInstruction::read(&mut reader)
                        .map_err(DXBParserError::BinRwError)?;

                    next_instructions_stack
                        .handle_next_expected_instructions(
                            instruction.get_next_expected_instructions(),
                        )
                        .map_err(|_| {
                            DXBParserError::NotInUnboundedRegularScopeError
                        })?;

                    instruction.into()
                }
            }
        };

        let instruction = match instruction_result {
            Ok(instruction) => instruction,
            Err(error) => return yield Err(error),
        };

        yield Ok(instruction);
    }
}

pub gen fn iterate_instructions_with_seek(
    dxb_body_ref: Rc<RefCell<Vec<u8>>>,
    nested_instruction_resolution_strategy: NestedInstructionResolutionStrategy,
    seek_request: SeekRequest,
) -> Result<Instruction, DXBParserError> {
    let mut next_instructions_stack = NextInstructionsStack::default();
    // Backward jumps will restore Instructions with snapshots
    let mut expectation_snapshots: HashMap<u64, NextInstructionsStack> =
        HashMap::new();

    let mut dxb_body = core::mem::take(&mut *dxb_body_ref.borrow_mut());
    let mut len = dxb_body.len();
    let mut reader = Cursor::new(dxb_body);

    loop {
        // here we check if there are any pending jump requests, if there are, we jump there before doing anything else
        let seek_offset = { seek_request.borrow_mut().take_offset() };
        if let Some(seek_offset) = seek_offset {
            let new_pos = reader.position() as i64 + seek_offset as i64;
            if new_pos < 0 || new_pos as usize > len {
                return yield Err(DXBParserError::InvalidJumpTarget(new_pos));
            }
            let target = new_pos as u64;
            // If PC must jump backwards, we will restore instructions from snapshot
            if target < reader.position() {
                let Some(snapshot) = expectation_snapshots.get(&target) else {
                    return yield Err(DXBParserError::InvalidJumpTarget(
                        new_pos,
                    ));
                };
                next_instructions_stack = snapshot.clone();
                reader.set_position(target);
                continue;
            }

            // We jump forward and read instructions without executing it, so we can create snapshot and avoid any structure problems
            let mut skipped_roots = 0u32;
            while reader.position() < target {
                let mut probe_reader = reader.clone();
                let mut probe_stack = next_instructions_stack.clone();
                let kind = probe_stack.pop();
                if let Err(error) = consume_subtree(&mut probe_reader, kind) {
                    return yield Err(error);
                }
                let subtree_end = probe_reader.position();
                if subtree_end > target {
                    return yield Err(DXBParserError::InvalidJumpTarget(
                        new_pos,
                    ));
                }
                while reader.position() < subtree_end {
                    expectation_snapshots.insert(
                        reader.position(),
                        next_instructions_stack.clone(),
                    );
                    if let Err(error) = consume_one_instruction(
                        &mut reader,
                        &mut next_instructions_stack,
                    ) {
                        return yield Err(error);
                    }
                }
                skipped_roots += 1;
            }
            if reader.position() != target {
                return yield Err(DXBParserError::InvalidJumpTarget(new_pos));
            }
            {
                seek_request.borrow_mut().skipped_instruction_count +=
                    skipped_roots;
            }
            if skipped_roots != 0 {
                // This does nothing, just help parser to understand what to do,
                // in future must be replaced on something that make sense, not just useless Conditional
                yield Ok(RegularInstruction::Conditional.into());
            }
            continue;
        }

        if reader.position() as usize >= len {
            if !next_instructions_stack.is_end() {
                yield Err(DXBParserError::ExpectingMoreInstructions);

                dxb_body = core::mem::take(&mut *dxb_body_ref.borrow_mut());
                len = dxb_body.len();
                reader = Cursor::new(dxb_body);

                continue;
            }

            return;
        }

        expectation_snapshots
            .insert(reader.position(), next_instructions_stack.clone());
        let next_instruction_type = next_instructions_stack.pop();

        let instruction_result: Result<Instruction, DXBParserError> = try {
            match next_instruction_type {
                NextInstructionType::End => {
                    if len > reader.position() as usize {
                        yield Err(
                            DXBParserError::
                                UnexpectedBytesAfterEndOfInstructions,
                        );
                    }

                    return;
                }

                NextInstructionType::Regular => {
                    let instruction = RegularInstruction::read(&mut reader)
                        .map_err(DXBParserError::BinRwError)?;

                    let instruction =
                        if let RegularInstruction::RemoteExecution(
                            instruction_block_data,
                        ) = instruction
                        {
                            match nested_instruction_resolution_strategy {
                                #[cfg(feature = "disassembler")]
                                NestedInstructionResolutionStrategy::
                                    ResolveNestedScopesFlat
                                | NestedInstructionResolutionStrategy::
                                    ResolveNestedScopesTree => {
                                    use crate::global::
                                        protocol_structures::
                                        instruction_data::{
                                            InstructionBlockDataDebugFlat,
                                            InstructionBlockDataDebugTree,
                                        };

                                    let (inner_instructions, err) =
                                        crate::disassembler::disassemble_body(
                                            &instruction_block_data.body,
                                            nested_instruction_resolution_strategy,
                                        );

                                    if let Some(err) = err {
                                        Err(err)?;
                                    }

                                    if nested_instruction_resolution_strategy
                                        == NestedInstructionResolutionStrategy::
                                            ResolveNestedScopesFlat
                                    {
                                        RegularInstruction::
                                            remote_execution_debug_flat(
                                                InstructionBlockDataDebugFlat {
                                                    length:
                                                        instruction_block_data
                                                            .length,
                                                    injected_variable_count:
                                                        instruction_block_data
                                                            .injected_value_count,
                                                    injected_values:
                                                        instruction_block_data
                                                            .injected_values.clone(),
                                                    body: inner_instructions
                                                        .flatten(),
                                                },
                                            )
                                    } else {
                                        RegularInstruction::
                                            remote_execution_debug_tree(
                                                InstructionBlockDataDebugTree {
                                                    length:
                                                        instruction_block_data
                                                            .length,
                                                    injected_variable_count:
                                                        instruction_block_data
                                                            .injected_value_count,
                                                    injected_values:
                                                        instruction_block_data
                                                            .injected_values.clone(),
                                                    body: inner_instructions,
                                                },
                                            )
                                    }
                                }

                                _ => {
                                    RegularInstruction::remote_execution(
                                        instruction_block_data.clone(),
                                    )
                                }
                            }
                        } else {
                            instruction
                        };

                    next_instructions_stack
                        .handle_next_expected_instructions(
                            instruction.get_next_expected_instructions(),
                        )
                        .map_err(|_| {
                            DXBParserError::NotInUnboundedRegularScopeError
                        })?;

                    instruction.into()
                }

                NextInstructionType::Type => {
                    let instruction = TypeInstruction::read(&mut reader)
                        .map_err(DXBParserError::BinRwError)?;

                    next_instructions_stack
                        .handle_next_expected_instructions(
                            instruction.get_next_expected_instructions(),
                        )
                        .map_err(|_| {
                            DXBParserError::NotInUnboundedRegularScopeError
                        })?;

                    instruction.into()
                }
            }
        };

        let instruction = match instruction_result {
            Ok(instruction) => instruction,
            Err(error) => return yield Err(error),
        };

        yield Ok(instruction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global::instruction_codes::InstructionCode;
    use core::assert_matches;

    fn iterate_dxb(
        data: Vec<u8>,
    ) -> impl Iterator<Item = Result<Instruction, DXBParserError>> {
        iterate_instructions(
            Rc::new(RefCell::new(data)),
            NestedInstructionResolutionStrategy::default(),
        )
    }

    #[test]
    fn invalid_instruction_code() {
        let data = vec![0xFF]; // Invalid instruction code
        let mut iterator = iterate_dxb(data);
        let result = iterator.next().unwrap();
        assert_matches!(result, Err(DXBParserError::BinRwError(_)));
    }

    #[test]
    fn empty_expect_more_instructions() {
        let data = vec![]; // Empty data
        let mut iterator = iterate_dxb(data);
        let result = iterator.next().unwrap();
        assert_matches!(result, Err(DXBParserError::ExpectingMoreInstructions));
    }

    #[test]
    fn valid_uint8_instruction() {
        let data = vec![InstructionCode::UINT_8 as u8, 42];
        let mut iterator = iterate_dxb(data);
        let result = iterator.next().unwrap();
        match result.expect("Expected a valid instruction") {
            Instruction::Regular(instr) => {
                assert_eq!(instr, RegularInstruction::uint8(42));
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
        let result = iterator
            .next()
            .unwrap()
            .expect("Expected a valid instruction");
        match result {
            Instruction::Regular(instr) => {
                assert_eq!(
                    instr,
                    RegularInstruction::short_text("Hello".to_string())
                );
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
        assert_eq!(
            iterator.next().unwrap(),
            Ok(Instruction::Regular(RegularInstruction::add()))
        );
        // next instruction should be first UINT_8
        assert_eq!(
            iterator.next().unwrap(),
            Ok(Instruction::Regular(RegularInstruction::uint8(10)))
        );
        // next instruction should be second UINT_8
        assert_eq!(
            iterator.next().unwrap(),
            Ok(Instruction::Regular(RegularInstruction::uint8(20)))
        );
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
        let mut iterator = iterate_instructions(
            data_ref.clone(),
            NestedInstructionResolutionStrategy::default(),
        );
        // first instruction should be LIST
        let result = iterator.next().unwrap();
        assert_eq!(
            result,
            Ok(Instruction::Regular(RegularInstruction::list_default(2)))
        );
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
        assert_eq!(
            result,
            Ok(Instruction::Regular(RegularInstruction::uint8(10)))
        );
        // next instruction should be second UINT_8
        let result = iterator.next().unwrap();
        assert_eq!(
            result,
            Ok(Instruction::Regular(RegularInstruction::uint8(20)))
        );
        // ensure no more instructions
        assert!(iterator.next().is_none());
    }

    #[test]
    fn unbounded_expect_more_instructions() {
        let data = vec![InstructionCode::UNBOUNDED_STATEMENTS as u8]; // Start unbounded statements
        let data_ref = Rc::new(RefCell::new(data));
        let mut iterator = iterate_instructions(
            data_ref.clone(),
            NestedInstructionResolutionStrategy::default(),
        );
        // first instruction should be UNBOUNDED_STATEMENTS
        let result = iterator.next().unwrap();
        assert_eq!(
            result.unwrap(),
            Instruction::Regular(RegularInstruction::unbounded_statements())
        );
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
        assert_eq!(
            result.unwrap(),
            Instruction::Regular(RegularInstruction::uint8(42))
        );
        // next instruction should be UNBOUNDED_STATEMENTS_END
        let result = iterator.next().unwrap();
        assert_eq!(
            result.unwrap(),
            Instruction::Regular(RegularInstruction::unbounded_statements_end(
                false
            ))
        );
        // ensure no more instructions
        assert!(iterator.next().is_none());
    }
}
