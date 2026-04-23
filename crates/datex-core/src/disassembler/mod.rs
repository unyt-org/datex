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
