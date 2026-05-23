/// Pretty-print DXB bytecode with recursive branch decoding
/// Usage:
///   dxb_pretty "1, 2, 0, 43, ..."              decode raw bytes
///   dxb_pretty -c "if (true) (42u8)"            compile source then decode( You will want to use this almost all time)
///   dxb_pretty -v "1, 2, 0, 43, ..."            verbose (hex dump + tree)
///   dxb_pretty -cv "if (true) (42u8)"           compile + verbose
use datex_core::disassembler::pretty_print_dxb_to_string;
use std::process::ExitCode;

fn hex_dump(bytes: &[u8]) {
    const WIDTH: usize = 16;
    for (i, chunk) in bytes.chunks(WIDTH).enumerate() {
        let offset = i * WIDTH;
        let hex: Vec<String> =
            chunk.iter().map(|b| format!("{:02x}", b)).collect();
        let ascii: String = chunk
            .iter()
            .map(|b| {
                if b.is_ascii_graphic() || *b == b' ' {
                    *b as char
                } else {
                    '.'
                }
            })
            .collect();
        println!(
            "{:04x}  {:<49}  {}",
            offset,
            hex.chunks(2)
                .map(|pair| pair.join(""))
                .collect::<Vec<_>>()
                .join(" "),
            ascii
        );
    }
}

fn parse_bytes(input: &str) -> Result<Vec<u8>, String> {
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
        Err("No valid byte values found in input".into())
    } else {
        Ok(bytes)
    }
}

fn print_help() {
    let name = std::env::args()
        .next()
        .unwrap_or_else(|| "dxb_pretty".into());
    eprintln!("Pretty-print DXB bytecode");
    eprintln!();
    eprintln!("Usage:");
    eprintln!(
        "  {name} <bytes>              decode raw bytes (comma/space-separated)"
    );
    eprintln!("  {name} -c <source>          compile Datex source then decode");
    eprintln!("  {name} -v <bytes>           verbose (hex dump + tree)");
    eprintln!("  {name} -cv <source>         compile + verbose");
    eprintln!("  {name} -h                   show this help");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  {name} \"4, 2, 0, 0, 0, 72, 0\"");
    eprintln!("  {name} -c \"if (true) (42u8)\"");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return ExitCode::FAILURE;
    }

    let mut verbose = false;
    let mut compile = false;
    let mut positional_start = 1;

    while positional_start < args.len()
        && args[positional_start].starts_with('-')
    {
        match args[positional_start].as_str() {
            "-v" | "--verbose" => verbose = true,
            "-c" | "--compile" => compile = true,
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-cv" | "-vc" => {
                compile = true;
                verbose = true;
            }
            flag => {
                eprintln!("Unknown flag: {flag}");
                eprintln!("Use -h for help.");
                return ExitCode::FAILURE;
            }
        }
        positional_start += 1;
    }

    let input = args[positional_start..].join(" ");

    let bytes: Vec<u8> = if compile {
        #[cfg(feature = "compiler")]
        {
            let runtime = datex_core::runtime::Runtime::stub();
            match datex_core::compiler::compile_script(
                &input,
                datex_core::compiler::CompileOptions::default(),
                runtime,
            ) {
                Ok((dxb, _scope)) => {
                    if verbose {
                        println!("--- Compiled {} bytes ---", dxb.len());
                        hex_dump(&dxb);
                        println!();
                    } else {
                        let hex: Vec<String> =
                            dxb.iter().map(|b| format!("{b}")).collect();
                        println!(
                            "--- Bytecode ({} bytes): {} ---",
                            dxb.len(),
                            hex.join(", ")
                        );
                        println!();
                    }
                    dxb
                }
                Err(e) => {
                    eprintln!("Compilation error: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
        #[cfg(not(feature = "compiler"))]
        {
            eprintln!(
                "Compilation not available (feature 'compiler' not enabled)."
            );
            eprintln!("Use --features compiler or use 'default' feature.");
            return ExitCode::FAILURE;
        }
    } else {
        match parse_bytes(&input) {
            Ok(bytes) => {
                if verbose {
                    println!("--- Input bytes ({}): ---", bytes.len());
                    hex_dump(&bytes);
                    println!();
                }
                bytes
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::FAILURE;
            }
        }
    };

    print!("{}", pretty_print_dxb_to_string(&bytes));
    ExitCode::SUCCESS
}
