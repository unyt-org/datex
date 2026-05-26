//! This module contains the disassembler for DATEX, which converts DXB bytecode into a human-readable assembly-like string representation e.g.
//! from `4, 2, 0, 0, 0, 72, 42, 0, 0, 0, 0, 83` to
//! ```asm
//! CONDITIONAL
//!     condition: TRUE true
//!     then:
//!         UINT_8 42
//! ```

#[cfg(feature = "disassembler")]
mod disassembler;
pub mod options;
use crate::{disassembler::options::DisassemblerOptions, prelude::*};
use cfg_if::cfg_if;
#[cfg(feature = "disassembler")]
pub use disassembler::*;
use log::info;

/// Converts a DXB block to a human-readable assembly string representation and prints it to stdout
pub fn print_disassembled(dxb: &[u8]) {
    print_disassembled_with_options(dxb, DisassemblerOptions::default());
}

/// Converts a DXB block to a human-readable assembly string representation and prints it to stdout
pub fn print_disassembled_with_options(
    dxb: &[u8],
    options: DisassemblerOptions,
) {
    info!(
        "\n\n=== Disassembled DXB Body ===\n{}==== END ===\n",
        get_disassembled_with_options(dxb, options)
    );
}

/// Converts a DXB block to a human-readable assembly string representation
pub fn get_disassembled_with_options(
    dxb: &[u8],
    options: DisassemblerOptions,
) -> String {
    cfg_if! {
        if #[cfg(feature = "disassembler")] {
            disassemble_body_to_string(dxb, options)
        }
        else {
            "[feature 'disassembler' is not enabled]".to_string()
        }
    }
}

/// Pretty-prints DXB bytecode as a tree with nested branch decoding and returns a String
/// Unlike `get_disassembled_with_options`, this produces a compact tree view
/// that recursively resolves CONDITIONAL branches inline
#[cfg(feature = "disassembler")]
pub fn pretty_print_dxb_to_string(bytes: &[u8]) -> String {
    use crate::global::{
        instruction_codes::InstructionCode,
        protocol_structures::{
            instructions::{Instruction, NestedInstructionResolutionStrategy},
            regular_instructions::RegularInstruction,
        },
        type_instruction_codes::TypeInstructionCode,
    };
    use alloc::format;
    use core::fmt::Write;

    fn tree_to_string(
        node: &disassembler::InstructionTree<Instruction>,
        depth: usize,
        output: &mut String,
    ) {
        let indent = "  ".repeat(depth);
        let instruction = &*node.instruction;

        match instruction {
            Instruction::Regular(RegularInstruction::Conditional(data)) => {
                writeln!(output, "{}CONDITIONAL", indent).unwrap();
                if let Some(cond_child) = node.children.first() {
                    write!(output, "{}  condition: ", indent).unwrap();
                    instr_inline(cond_child, "", output);
                } else {
                    writeln!(output, "{}  condition: (none)", indent).unwrap();
                }
                if !data.then_branch.branch.is_empty() {
                    writeln!(output, "{}  then:", indent).unwrap();
                    let then_bytes = &data.then_branch.branch;
                    let (then_tree, _) = disassembler::disassemble_body(
                        then_bytes,
                        NestedInstructionResolutionStrategy::ResolveNestedScopesFlat,
                    );
                    tree_to_string(&then_tree, depth + 2, output);
                } else {
                    writeln!(output, "{}  then: (empty)", indent).unwrap();
                }
                if !data.else_branch.branch.is_empty() {
                    writeln!(output, "{}  else:", indent).unwrap();
                    let else_bytes = &data.else_branch.branch;
                    let (else_tree, _) = disassembler::disassemble_body(
                        else_bytes,
                        NestedInstructionResolutionStrategy::ResolveNestedScopesFlat,
                    );
                    tree_to_string(&else_tree, depth + 2, output);
                }
            }
            inst => {
                instr_inline(node, &indent, output);
                for child in &node.children {
                    tree_to_string(child, depth + 1, output);
                }
            }
        }
    }

    fn instr_inline(
        node: &disassembler::InstructionTree<Instruction>,
        indent: &str,
        output: &mut String,
    ) {
        let instruction = &*node.instruction;
        let name = match instruction {
            Instruction::Regular(reg) => InstructionCode::from(reg).to_string(),
            Instruction::Type(ty) => TypeInstructionCode::from(ty).to_string(),
        };
        let meta = match instruction {
            Instruction::Regular(RegularInstruction::UInt8(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::UInt16(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::UInt32(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::UInt64(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::UInt128(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::Int8(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::Int16(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::Int32(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::Int64(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::Integer(data)) => {
                format!("{}", data.0)
            }
            Instruction::Regular(RegularInstruction::True) => "true".into(),
            Instruction::Regular(RegularInstruction::False) => "false".into(),
            Instruction::Regular(RegularInstruction::Null) => "null".into(),
            Instruction::Regular(RegularInstruction::ShortText(data)) => {
                format!("\"{}\"", data.0)
            }
            Instruction::Regular(RegularInstruction::TakeStackValue(idx))
            | Instruction::Regular(RegularInstruction::CloneStackValue(idx))
            | Instruction::Regular(
                RegularInstruction::GetStackValueSharedRef(idx),
            )
            | Instruction::Regular(
                RegularInstruction::GetStackValueSharedRefMut(idx),
            )
            | Instruction::Regular(RegularInstruction::BorrowStackValue(idx)) =>
            {
                format!("slot {}", idx.0)
            }
            Instruction::Regular(RegularInstruction::Jump(data)) => {
                format!("{}", data.offset)
            }
            _ => String::new(),
        };
        if meta.is_empty() {
            writeln!(output, "{}{}", indent, name).unwrap();
        } else {
            writeln!(output, "{}{} {}", indent, name, meta).unwrap();
        }
    }

    let (tree, _err) = disassembler::disassemble_body(
        bytes,
        NestedInstructionResolutionStrategy::ResolveNestedScopesFlat,
    );
    let mut output = String::new();
    tree_to_string(&tree, 0, &mut output);
    output
}
