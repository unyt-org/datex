use alloc::rc::Rc;
use core::cell::RefCell;
use crate::dxb_parser::body::{iterate_instructions, DXBParserError};
use core::fmt::Write;
use crate::global::protocol_structures::instructions::Instruction;

pub fn disassemble_body_to_string(body: &[u8]) -> Result<String, DXBParserError> {
    let mut output = "\n=== Disassembled DXB Body ===\n".to_string();
    write!(&mut output, "{}", disassemble_body_to_strings(body)?.join("\n"))?;
    write!(&mut output, "\n==== END ===\n")?;
    Ok(output)
}

/// Converts a raw DXB body into a list of disassembled human-readable instructions
pub fn disassemble_body_to_strings(body: &[u8]) -> Result<Vec<String>, DXBParserError> {
    Ok(disassemble_body(body)?.into_iter().map(|instr| instr.to_string()).collect::<Vec<String>>())
}

/// Converts a raw DXB body into a list of disassembled Instruction values
pub fn disassemble_body(body: &[u8]) -> Result<Vec<Instruction>, DXBParserError> {
    let mut instructions = Vec::new();

    // TODO: no to_vec clone of body
    for instruction in iterate_instructions(Rc::new(RefCell::new(body.to_vec()))) {
        let instruction = instruction?;
        instructions.push(instruction);
    }

    Ok(instructions)
}