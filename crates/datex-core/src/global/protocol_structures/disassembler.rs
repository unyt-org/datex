use alloc::rc::Rc;
use core::cell::RefCell;
use crate::dxb_parser::body::{iterate_instructions, DXBParserError};
use core::fmt::Write;
use crate::global::protocol_structures::instructions::Instruction;
use crate::prelude::*;

pub fn disassemble_body_to_string(body: &[u8]) -> String {
    let (instructions, err) = disassemble_body_to_strings(body);

    let mut output = "\n=== Disassembled DXB Body ===\n".to_string();
    write!(&mut output, "{}", instructions.join("\n")).unwrap();
    if let Some(err) = err {
        write!(&mut output, "\n[!] Parser Error: {}", err).unwrap();
    }
    write!(&mut output, "\n==== END ===\n").unwrap();

    output
}

/// Converts a raw DXB body into a list of disassembled human-readable instructions
pub fn disassemble_body_to_strings(body: &[u8]) -> (Vec<String>, Option<DXBParserError>) {
    let (instructions, err) = disassemble_body(body);
    let instructions = instructions.iter().map(ToString::to_string).collect();
    (instructions, err)
}

/// Converts a raw DXB body into a list of disassembled Instruction values
pub fn disassemble_body(body: &[u8]) -> (Vec<Instruction>, Option<DXBParserError>) {
    let mut instructions = Vec::new();

    // TODO: no to_vec clone of body
    for instruction in iterate_instructions(Rc::new(RefCell::new(body.to_vec()))) {
        match instruction {
            Err(e) => {
                return (instructions, Some(e));
            },
            Ok(instruction) => {
                instructions.push(instruction);
            }
        }
    }

    (instructions, None)
}

#[macro_export]
macro_rules! assert_instructions_equal {
    ($dxb:expr, $expected:expr) => {{
        let (instructions, err) = disassemble_body($dxb);
        if let Some(err) = err {
            panic!("Parser error: {}", err);
        }
        assert_eq!(
            &instructions,
            &$expected
        );
    }}
}