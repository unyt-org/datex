use alloc::rc::Rc;
use core::cell::RefCell;
use crate::dxb_parser::body::{iterate_instructions, DXBParserError};
use core::fmt::{Debug, Write};
use binrw::{BinRead, BinWrite};
use serde::Serialize;
use crate::global::instruction_codes::InstructionCode;
use crate::disassembler::options::DisassemblerOptions;
use crate::global::protocol_structures::instruction_data::{StatementsData};
use crate::global::protocol_structures::instructions::{CountOrUnbounded, Instruction, NestedInstructionResolutionStrategy};
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::global::type_instruction_codes::TypeInstructionCode;
use crate::prelude::*;
use crate::utils::ansi_colors::{AnsiColor, AnsiWrite};


/// A generic tree structure for instructions with child instructions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InstructionTree<T> where T: Debug + Clone {
    instruction: Box<T>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<InstructionTree<T>>,
}

impl<T> InstructionTree<T> where T: Debug + Clone {
    /// Create a new tree with a root instruction
    pub fn new(instruction: T) -> Self {
        Self {
            instruction: Box::new(instruction),
            children: Vec::new(),
        }
    }

    /// Flattens the tree into a list of instructions
    pub fn flatten(&self) -> Vec<T> {
        let mut result = vec![*self.instruction.clone()];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }

    /// Maps a tree to an instruction tree with a different generic type with a mapping function, preserving the structure
    pub fn map<N: Debug + Clone>(self, f: impl Fn(T) -> N + Clone) -> InstructionTree<N> {
        InstructionTree {
            instruction: Box::new(f(*self.instruction)),
            children: self.children.into_iter().map(|child| child.map(f.clone())).collect(),
        }
    }
}

/// An instruction tree containing an optional detailed instruction tree inside each node
#[derive(Debug, Clone)]
struct DetailedInstructionTree(
    pub InstructionTree<(Instruction, Option<Box<DetailedInstructionTree>>)>
);

#[derive(Default, Clone, Debug, PartialEq)]
pub enum InnerInstructions<'a> {
    #[default]
    None,
    Flat(&'a Vec<Instruction>),
    Tree(&'a InstructionTree<Instruction>),
}

/// Converts a raw DXB body in to human-readable disassembled instructions string
pub fn disassemble_body_to_string(body: &[u8], options: DisassemblerOptions) -> String {
    let (instructions, err) = disassemble_body(body, options.nested_instructions_resolution_strategy());
    let instructions = instruction_tree_to_detailed_tree(instructions);

    let mut output = String::new();

    if options.tree {
        disassemble_body_to_string_inner(
            &mut output,
            instructions,
            0,
            true,
            true,
            false,
            options.colorized,
            0
        );
    }
    else {
        write_flat_instructions(&mut output, instructions, options.colorized, 0);
    }

    if let Some(err) = err {
        if options.colorized {
            write!(&mut output, "\x1b[38;2;245;39;60m\n[!] Parser Error: {}\x1b[0m", err).unwrap();
        }
        else {
            write!(&mut output, "[!] Parser Error: {}", err).unwrap();
        }
    }

    output
}

/// Converts a raw DXB body into a list of disassembled Instruction values
pub fn disassemble_body(body: &[u8], nested_instruction_resolution_strategy: NestedInstructionResolutionStrategy) -> (InstructionTree<Instruction>, Option<DXBParserError>) {
    let mut iterator = iterate_instructions(Rc::new(RefCell::new(body.to_vec())), nested_instruction_resolution_strategy);
    let mut tree = InstructionTree::new(Instruction::Regular(RegularInstruction::UnboundedStatements)); // initial tree root, gets overridden
    let err = disassemble_body_inner(&mut iterator, &mut tree, CountOrUnbounded::UnboundedStart, true);
    (tree, err)
}


/// Writes a detailed instruction tree to an output string recursively with optional colorization and indentation
fn write_flat_instructions(output: &mut String, instructions: DetailedInstructionTree, colorized: bool, level: u32) {
    for (instruction, inner_instructions) in instructions.0.flatten() {
        write!(
            output,
            "{}",
            " ".repeat(level as usize * 2),
        ).unwrap();
        write_instruction(
            output,
            &instruction,
            level,
            colorized,
        );
        if let Some(inner_instructions) = inner_instructions {
            write_flat_instructions(output, *inner_instructions, colorized, level + 1);
        }
    }
}

/// Writes a single instruction to an output string with optional colorization and indentation
fn write_instruction(
    output: &mut String,
    instruction: &Instruction,
    level: u32,
    colorized: bool
) {
    if colorized {
        let color = color_for_level(level);
        output.write_fg(color);
    }

    write!(output, "{}",
        match instruction {
            Instruction::Regular(instr) => InstructionCode::from(instr).to_string(),
            Instruction::Type(instr) => TypeInstructionCode::from(instr).to_string(),
        }
    ).unwrap();

    if colorized {
        output.write_reset();
    }

    if let Some(metadata_string) = instruction.metadata_string() {
        write!(output, " {}", metadata_string).unwrap();
    }

    writeln!(output, "").unwrap();
}


/// Returns an instruction text color for a given level
fn color_for_level(level: u32) -> AnsiColor {
    match level % 10 {
        0 => AnsiColor::Rgb(0, 153, 204),    // deep sky blue
        1 => AnsiColor::Rgb(0, 204, 153),    // teal
        2 => AnsiColor::Rgb(51, 204, 102),   // green
        3 => AnsiColor::Rgb(153, 255, 51),   // lime green
        4 => AnsiColor::Rgb(255, 221, 51),   // golden yellow
        5 => AnsiColor::Rgb(204, 204, 255),  // light periwinkle
        6 => AnsiColor::Rgb(153, 153, 255),  // soft purple
        7 => AnsiColor::Rgb(153, 102, 204),  // medium purple
        8 => AnsiColor::Rgb(255, 153, 204),  // pink
        9 => AnsiColor::Rgb(255, 204, 229),  // soft pink / rose
        _ => AnsiColor::Rgb(200, 200, 200),  // neutral fallback
    }
}


fn disassemble_body_to_string_inner(
    output: &mut String,
    instructions: DetailedInstructionTree,
    indent_width: usize,
    is_root_child: bool,
    is_last_child: bool,
    is_inner: bool,
    colorized: bool,
    level: u32,
) {
    let indent = " ".repeat(indent_width * 3);
    let children_len = instructions.0.children.len();

    let (main, inner) = *instructions.0.instruction;

    write!(
        output,
        "{}{}",
        indent,
        if is_root_child {""} else if is_inner {"↳  "} else if is_last_child || children_len > 0 { "└─ " } else { "├─ " },
    ).unwrap();

    write_instruction(
        output,
        &main,
        level,
        colorized,
    );

    if let Some(inner) = inner {
        disassemble_body_to_string_inner(
            output,
            *inner,
            if is_root_child {0} else {indent_width + 1},
            false,
            false,
            true,
            colorized,
            level + 1,
        );
    }

    for (i, child) in instructions.0.children.into_iter().enumerate() {
        disassemble_body_to_string_inner(
            output,
            DetailedInstructionTree(child),
            if is_root_child {0} else {indent_width + 1},
            false,
            i == children_len - 1,
            false,
            colorized,
            level,
        );
    }
}


fn instruction_tree_to_detailed_tree(instructions: InstructionTree<Instruction>) -> DetailedInstructionTree {
    DetailedInstructionTree(instructions.map(|i| {
        let inner = get_inner_instructions_as_detailed_tree(&i).map(Box::new);
        (i, inner)
    }))
}

fn get_inner_instructions_as_detailed_tree(instruction: &Instruction) -> Option<DetailedInstructionTree> {
    match instruction {
        Instruction::Regular(RegularInstruction::_RemoteExecutionDebugFlat(data)) => {
            unreachable!()
        }
        Instruction::Regular(RegularInstruction::_RemoteExecutionDebugTree(data)) => {
            Some(instruction_tree_to_detailed_tree(data.body.clone()))
        }
        _ => None
    }
}
fn disassemble_body_inner(
    iterator: &mut impl Iterator<Item = Result<Instruction, DXBParserError>>,
    parent: &mut InstructionTree<Instruction>,
    count_or_unbounded: CountOrUnbounded,
    is_root: bool,
) -> Option<DXBParserError> {
    loop {
        let next = iterator.next();
        match next {
            Some(instruction) => {
                match instruction {
                    Err(e) => {
                        return Some(e);
                    },
                    Ok(instruction) => {
                        // get next expected children
                        let next_expected_count = instruction.get_next_expected_instructions().total_count();
                        let mut tree = InstructionTree::new(instruction);

                        let err = match next_expected_count {
                            Some(next_expected_count) => {
                                match next_expected_count {
                                    CountOrUnbounded::UnboundedEnd => {
                                        parent.children.push(tree);
                                        return None
                                    },
                                    _ => disassemble_body_inner(iterator, &mut tree, next_expected_count, false),
                                }
                            },
                            None => None
                        };

                        // if root node, replace parent with first instruction
                        if is_root {
                            *parent = tree;
                        }
                        else {
                            parent.children.push(tree);
                        }

                        if let Some(err) = err {
                            return Some(err);
                        }

                        // all expected children collected
                        if let CountOrUnbounded::Count(expected_count) = count_or_unbounded && parent.children.len() as u32 >= expected_count {
                            break;
                        }
                    }
                }
            }
            None => break
        }

    }

    None
}


#[cfg(feature = "disassembler")]
#[macro_export]
macro_rules! assert_instructions_equal {
    ($dxb:expr, $expected:expr) => {{
        use crate::global::protocol_structures::instructions::NestedInstructionResolutionStrategy;
        use crate::disassembler::disassemble_body;

        let (instructions, err) = disassemble_body($dxb, NestedInstructionResolutionStrategy::ResolveNestedScopesFlat);
        if let Some(err) = err {
            panic!("Parser error: {}", err);
        }
        assert_eq!(
            &instructions.flatten(),
            &$expected
        );
    }}
}

#[cfg(feature = "disassembler")]
#[macro_export]
macro_rules! assert_regular_instructions_equal {
    ($dxb:expr, $expected:expr) => {{
        use crate::global::protocol_structures::instructions::NestedInstructionResolutionStrategy;
        use crate::disassembler::disassemble_body;

        let (instructions, err) = disassemble_body($dxb, NestedInstructionResolutionStrategy::ResolveNestedScopesFlat);
        if let Some(err) = err {
            panic!("Parser error: {}", err);
        }
        assert_eq!(
            &instructions.flatten(),
            &($expected.into_iter().map(Instruction::Regular).collect::<Vec<_>>())
        );
    }}
}



#[cfg(test)]
mod tests {
    use crate::core_compiler::value_compiler::{append_instruction};
    use binrw::io::{Cursor};
    use rstest::rstest;
    use crate::global::protocol_structures::instruction_data::{InstructionBlockData, InstructionBlockDataDebugFlat, InstructionBlockDataDebugTree, StatementsData, UInt8Data, UnboundedStatementsData};
    use crate::global::protocol_structures::regular_instructions::RegularInstruction;
    use crate::runtime::{Runtime, RuntimeConfig, RuntimeRunner};
    use super::*;

    fn instructions_to_bytes(instructions: Vec<Instruction>) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        for instruction in instructions {
            append_instruction(&mut cursor, instruction);
        }
        cursor.into_inner()
    }

    #[rstest]

    /// empty dxb
    #[case(
        &[],
        InstructionTree::new(Instruction::Regular(RegularInstruction::UnboundedStatements)),
        Some(DXBParserError::ExpectingMoreInstructions)
    )]

    /// multiple root nodes
    #[case(
        &[
            Instruction::Regular(RegularInstruction::True),
            Instruction::Regular(RegularInstruction::False),
        ],
        InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
        Some(DXBParserError::UnexpectedBytesAfterEndOfInstructions)
    )]


    /// simple statements
    #[case(
        &[
            Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true})),
            Instruction::Regular(RegularInstruction::True),
            Instruction::Regular(RegularInstruction::False),
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
                InstructionTree::new(Instruction::Regular(RegularInstruction::False)),
            ]
        },
        None
    )]

    /// unbounded statements
    #[case(
        &[
            Instruction::Regular(RegularInstruction::UnboundedStatements),
            Instruction::Regular(RegularInstruction::True),
            Instruction::Regular(RegularInstruction::False),
            Instruction::Regular(RegularInstruction::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})),
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::UnboundedStatements)),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
                InstructionTree::new(Instruction::Regular(RegularInstruction::False)),
                InstructionTree::new(Instruction::Regular(RegularInstruction::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})))
            ]
        },
        None
    )]

    /// normal and unbounded statements
    #[case(
        &[
            Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true})),
            Instruction::Regular(RegularInstruction::UnboundedStatements),
            Instruction::Regular(RegularInstruction::True),
            Instruction::Regular(RegularInstruction::False),
            Instruction::Regular(RegularInstruction::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})),
            Instruction::Regular(RegularInstruction::Null),
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                InstructionTree {
                    instruction: Box::new(Instruction::Regular(RegularInstruction::UnboundedStatements)),
                    children: vec![
                        InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
                        InstructionTree::new(Instruction::Regular(RegularInstruction::False)),
                        InstructionTree::new(Instruction::Regular(RegularInstruction::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})))
                    ]
                },
                InstructionTree::new(Instruction::Regular(RegularInstruction::Null)),
            ]
        },
        None
    )]

    /// single instruction
    #[case(
        &[
            Instruction::Regular(RegularInstruction::True),
        ],
        InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
        None
    )]

    /// statements with missing instructions
    #[case(
        &[
            Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true})),
            Instruction::Regular(RegularInstruction::True),
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        },
        Some(DXBParserError::ExpectingMoreInstructions)
    )]

    /// remote execution
    #[case(
        &[
            Instruction::Regular(RegularInstruction::RemoteExecution(InstructionBlockData {
                length: 2,
                injected_value_count: 0,
                injected_values: vec![],
                body: vec![
                    InstructionCode::UINT_8 as u8,
                    42,
                ]
            })),
            Instruction::Regular(RegularInstruction::True)
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::RemoteExecution(InstructionBlockData {
                length: 2,
                injected_value_count: 0,
                injected_values: vec![],
                body: vec![
                    InstructionCode::UINT_8 as u8,
                    42,
                ]
            }))),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        },
        None
    )]

    fn disassemble_statements(
        #[case] instructions: &[Instruction],
        #[case] expected_tree: InstructionTree<Instruction>,
        #[case] expected_err: Option<DXBParserError>,
    ) {
        let dxb = instructions_to_bytes(instructions.to_vec());
        let (tree, err) = disassemble_body(&dxb, NestedInstructionResolutionStrategy::default());

        assert_eq!(err, expected_err);
        assert_eq!(tree, expected_tree,)
    }

    #[test]
    fn disassemble_nested_flat() {
        let instructions = vec![
            Instruction::Regular(RegularInstruction::RemoteExecution(InstructionBlockData {
                length: 5,
                injected_value_count: 0,
                injected_values: vec![],
                body: vec![
                    InstructionCode::ADD as u8,
                    InstructionCode::UINT_8 as u8,
                    42,
                    InstructionCode::UINT_8 as u8,
                    43,
                ]
            })),
            Instruction::Regular(RegularInstruction::True),
        ];
        let dxb = instructions_to_bytes(instructions.to_vec());
        let (tree, err) = disassemble_body(&dxb, NestedInstructionResolutionStrategy::ResolveNestedScopesFlat);

        assert_eq!(err, None);
        assert_eq!(tree, InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::_RemoteExecutionDebugFlat(InstructionBlockDataDebugFlat {
                length: 5,
                injected_variable_count: 0,
                injected_values: vec![],
                body: vec![
                    Instruction::Regular(RegularInstruction::Add),
                    Instruction::Regular(RegularInstruction::UInt8(UInt8Data(42))),
                    Instruction::Regular(RegularInstruction::UInt8(UInt8Data(43))),
                ]
            }))),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        });
    }

    #[test]
    fn disassemble_nested_tree() {
        let instructions = vec![
            Instruction::Regular(RegularInstruction::RemoteExecution(InstructionBlockData {
                length: 5,
                injected_value_count: 0,
                injected_values: vec![],
                body: vec![
                    InstructionCode::ADD as u8,
                    InstructionCode::UINT_8 as u8,
                    42,
                    InstructionCode::UINT_8 as u8,
                    43,
                ]
            })),
            Instruction::Regular(RegularInstruction::True),
        ];
        let dxb = instructions_to_bytes(instructions.to_vec());
        let (tree, err) = disassemble_body(&dxb, NestedInstructionResolutionStrategy::ResolveNestedScopesTree);


        assert_eq!(err, None);
        assert_eq!(tree, InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::_RemoteExecutionDebugTree(InstructionBlockDataDebugTree {
                length: 5,
                injected_variable_count: 0,
                injected_values: vec![],
                body: InstructionTree {
                    instruction: Box::new(Instruction::Regular(RegularInstruction::Add)),
                    children: vec![
                        InstructionTree::new(Instruction::Regular(RegularInstruction::UInt8(UInt8Data(42)))),
                        InstructionTree::new(Instruction::Regular(RegularInstruction::UInt8(UInt8Data(43)))),
                    ]
                }
            }))),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        });
    }

    #[ignore]
    #[cfg(all(feature = "std", feature = "compiler"))]
    #[test]
    fn disassemble_string_test() {
        use crate::compiler::{compile_script, CompileOptions};

        let script = r#"
            var x = 5;
            var y = 42;
            @example :: (
                1;2;3;
                @test :: (1 + 2);
            )
        "#;
        let (dxb, _) = compile_script(script, CompileOptions::default(), Runtime::stub()).unwrap();
        println!("{}", disassemble_body_to_string(&dxb, DisassemblerOptions {
            tree: true,
            colorized: true,
            recursive: true,
        }));

        println!("{}", disassemble_body_to_string(&dxb, DisassemblerOptions {
            tree: false,
            colorized: true,
            recursive: true,
        }));

        println!("{}", disassemble_body_to_string(&dxb, DisassemblerOptions {
            tree: true,
            colorized: true,
            recursive: false,
        }));
    }
}