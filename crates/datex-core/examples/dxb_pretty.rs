use datex_core::disassembler::{
    get_disassembled_with_options, options::DisassemblerOptions,
};
/// Pretty-print DXB bytecode
/// Usage: cargo run --example dxb_pretty -- "1, 2, 0, 43, 72, 10, 4, 5, 0, 0, 0, 45, 0, 0, 0, 0, 2, 0, 0, 0, 72, 0, 83"
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: dxb_pretty \"1, 2, 0, 43, ...\"");
        eprintln!(
            "Paste the comma-separated byte values as a single argument."
        );
        return ExitCode::FAILURE;
    }

    let input = args[1..].join(" ");
    let bytes: Vec<u8> = input
        .split(|c: char| c == ',' || c == '[' || c == ']' || c == ' ')
        .filter_map(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                s.parse::<u8>().ok()
            }
        })
        .collect();

    if bytes.is_empty() {
        eprintln!("No valid byte values found in input");
        return ExitCode::FAILURE;
    }

    let disassembly = get_disassembled_with_options(
        &bytes,
        DisassemblerOptions {
            tree: false,
            colorized: false,
            recursive: true,
        },
    );

    println!("{}", disassembly);
    ExitCode::SUCCESS
}
