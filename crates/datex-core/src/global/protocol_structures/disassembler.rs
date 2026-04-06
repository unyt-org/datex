use alloc::rc::Rc;
use core::cell::RefCell;
use crate::dxb_parser::body::{iterate_instructions, DXBParserError};
use core::fmt::{Debug, Write};
use std::io::Write as StdWrite;
use binrw::{BinRead, BinWrite};
use termcolor::{Buffer, Color, ColorSpec, WriteColor};
use crate::global::instruction_codes::InstructionCode;
use crate::global::protocol_structures::instruction_data::{StatementsData};
use crate::global::protocol_structures::instructions::{CountOrUnbounded, Instruction, NestedInstructionResolutionStrategy};
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::global::type_instruction_codes::TypeInstructionCode;
use crate::prelude::*;


#[derive(Debug, Clone, PartialEq)]
pub struct Tree<T> where T: Debug + Clone {
    instruction: Box<T>,
    children: Vec<Tree<T>>,
}

impl<T> Tree<T> where T: Debug + Clone {
    pub fn new(instruction: T) -> Self {
        Self {
            instruction: Box::new(instruction),
            children: Vec::new(),
        }
    }
    pub fn flatten(&self) -> Vec<T> {
        let mut result = vec![*self.instruction.clone()];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }

    pub fn map<N: Debug + Clone>(self, f: impl Fn(T) -> N + Clone) -> Tree<N> {
        Tree {
            instruction: Box::new(f(*self.instruction)),
            children: self.children.into_iter().map(|child| child.map(f.clone())).collect(),
        }
    }
}

pub struct DisassemblerOptions {
    pub tree: bool,
    pub colorized: bool,
    pub recursive: bool,
}

impl DisassemblerOptions {
    pub fn simple() -> DisassemblerOptions{
        DisassemblerOptions {
            tree: false,
            colorized: false,
            recursive: false,
        }
    }

    fn nested_instructions_resolution_strategy(&self) -> NestedInstructionResolutionStrategy {
        if self.recursive {
            NestedInstructionResolutionStrategy::ResolveNestedScopesTree // always resolve as tree, collapse later if needed for string display
        }
        else {
            NestedInstructionResolutionStrategy::None
        }
    }
}

impl Default for DisassemblerOptions {
    fn default() -> DisassemblerOptions {
        DisassemblerOptions {
            tree: true,
            colorized: true,
            recursive: true,
        }
    }
}

/// Converts a raw DXB body in to human-readable disassembled instructions string
pub fn disassemble_body_to_string(body: &[u8], options: DisassemblerOptions) -> String {
    let (instructions, err) = disassemble_body(body, options.nested_instructions_resolution_strategy());
    let instructions = instruction_tree_to_detailed_tree(instructions);

    let mut output = "\n\n=== Disassembled DXB Body ===\n".to_string();

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
    writeln!(&mut output, "==== END ===\n").unwrap();

    output
}

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

fn write_instruction(
    output: &mut String,
    instruction: &Instruction,
    level: u32,
    colorized: bool
) {
    let mut buffer = Buffer::ansi();
    if colorized {
        let color = color_for_level(level);
        buffer.set_color(ColorSpec::new().set_fg(Some(color))).unwrap();
    }

    write!(&mut buffer, "{}",
        match instruction {
            Instruction::Regular(instr) => InstructionCode::from(instr).to_string(),
            Instruction::Type(instr) => TypeInstructionCode::from(instr).to_string(),
        }
    ).unwrap();

    if colorized {
        buffer.set_color(&ColorSpec::new()).unwrap();
    }

    if let Some(metadata_string) = instruction.metadata_string() {
        write!(&mut buffer, " {}", metadata_string).unwrap();
    }

    writeln!(output, "{}", String::from_utf8_lossy(&buffer.into_inner())).unwrap();
}


fn color_for_level(level: u32) -> Color {
    match level % 10 {
        0 => Color::Rgb(0, 153, 204),    // deep sky blue
        1 => Color::Rgb(0, 204, 153),    // teal
        2 => Color::Rgb(51, 204, 102),   // green
        3 => Color::Rgb(153, 255, 51),   // lime green
        4 => Color::Rgb(255, 221, 51),   // golden yellow
        5 => Color::Rgb(204, 204, 255),  // light periwinkle
        6 => Color::Rgb(153, 153, 255),  // soft purple
        7 => Color::Rgb(153, 102, 204),  // medium purple
        8 => Color::Rgb(255, 153, 204),  // pink
        9 => Color::Rgb(255, 204, 229),  // soft pink / rose
        _ => Color::Rgb(200, 200, 200),  // neutral fallback
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
        if is_root_child || is_inner {""} else if is_last_child || children_len > 0 { "└─ " } else { "├─ " },
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
            indent_width + 1,
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

#[derive(Debug, Clone)]
struct DetailedInstructionTree(
    pub Tree<(Instruction, Option<Box<DetailedInstructionTree>>)>
);


fn instruction_tree_to_detailed_tree(instructions: Tree<Instruction>) -> DetailedInstructionTree {
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

/// Converts a raw DXB body into a list of disassembled Instruction values
pub fn disassemble_body(body: &[u8], nested_instruction_resolution_strategy: NestedInstructionResolutionStrategy) -> (Tree<Instruction>, Option<DXBParserError>) {
    let mut iterator = iterate_instructions(Rc::new(RefCell::new(body.to_vec())), nested_instruction_resolution_strategy);
    let mut tree = Tree::new(Instruction::Regular(RegularInstruction::UnboundedStatements)); // initial tree root, gets overridden
    let err = disassemble_body_inner(&mut iterator, &mut tree, CountOrUnbounded::UnboundedStart, true);
    (tree, err)
}

fn disassemble_body_inner(
    iterator: &mut impl Iterator<Item = Result<Instruction, DXBParserError>>,
    parent: &mut Tree<Instruction>,
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
                        let mut tree = Tree::new(instruction);

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


#[macro_export]
macro_rules! assert_instructions_equal {
    ($dxb:expr, $expected:expr) => {{
        use crate::global::protocol_structures::instructions::NestedInstructionResolutionStrategy;
        use crate::global::protocol_structures::disassembler::disassemble_body;

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

#[macro_export]
macro_rules! assert_regular_instructions_equal {
    ($dxb:expr, $expected:expr) => {{
        use crate::global::protocol_structures::instructions::NestedInstructionResolutionStrategy;
        use crate::global::protocol_structures::disassembler::disassemble_body;

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
    use super::*;

    fn instructions_to_bytes(instructions: Vec<Instruction>) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        for instruction in instructions {
            append_instruction(&mut cursor, instruction).unwrap();
        }
        cursor.into_inner()
    }

    #[rstest]

    /// empty dxb
    #[case(
        &[],
        Tree::new(Instruction::Regular(RegularInstruction::UnboundedStatements)),
        Some(DXBParserError::ExpectingMoreInstructions)
    )]

    /// multiple root nodes
    #[case(
        &[
            Instruction::Regular(RegularInstruction::True),
            Instruction::Regular(RegularInstruction::False),
        ],
        Tree::new(Instruction::Regular(RegularInstruction::True)),
        Some(DXBParserError::UnexpectedBytesAfterEndOfInstructions)
    )]


    /// simple statements
    #[case(
        &[
            Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true})),
            Instruction::Regular(RegularInstruction::True),
            Instruction::Regular(RegularInstruction::False),
        ],
        Tree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                Tree::new(Instruction::Regular(RegularInstruction::True)),
                Tree::new(Instruction::Regular(RegularInstruction::False)),
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
        Tree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::UnboundedStatements)),
            children: vec![
                Tree::new(Instruction::Regular(RegularInstruction::True)),
                Tree::new(Instruction::Regular(RegularInstruction::False)),
                Tree::new(Instruction::Regular(RegularInstruction::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})))
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
        Tree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                Tree {
                    instruction: Box::new(Instruction::Regular(RegularInstruction::UnboundedStatements)),
                    children: vec![
                        Tree::new(Instruction::Regular(RegularInstruction::True)),
                        Tree::new(Instruction::Regular(RegularInstruction::False)),
                        Tree::new(Instruction::Regular(RegularInstruction::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})))
                    ]
                },
                Tree::new(Instruction::Regular(RegularInstruction::Null)),
            ]
        },
        None
    )]

    /// single instruction
    #[case(
        &[
            Instruction::Regular(RegularInstruction::True),
        ],
        Tree::new(Instruction::Regular(RegularInstruction::True)),
        None
    )]

    /// statements with missing instructions
    #[case(
        &[
            Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true})),
            Instruction::Regular(RegularInstruction::True),
        ],
        Tree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                Tree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        },
        Some(DXBParserError::ExpectingMoreInstructions)
    )]

    /// remote execution
    #[case(
        &[
            Instruction::Regular(RegularInstruction::RemoteExecution(InstructionBlockData {
                length: 2,
                injected_variable_count: 0,
                injected_variables: vec![],
                body: vec![
                    InstructionCode::UINT_8 as u8,
                    42,
                ]
            })),
            Instruction::Regular(RegularInstruction::True)
        ],
        Tree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::RemoteExecution(InstructionBlockData {
                length: 2,
                injected_variable_count: 0,
                injected_variables: vec![],
                body: vec![
                    InstructionCode::UINT_8 as u8,
                    42,
                ]
            }))),
            children: vec![
                Tree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        },
        None
    )]

    fn disassemble_statements(
        #[case] instructions: &[Instruction],
        #[case] expected_tree: Tree<Instruction>,
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
                injected_variable_count: 0,
                injected_variables: vec![],
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
        assert_eq!(tree, Tree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::_RemoteExecutionDebugFlat(InstructionBlockDataDebugFlat {
                length: 5,
                injected_variable_count: 0,
                injected_variables: vec![],
                body: vec![
                    Instruction::Regular(RegularInstruction::Add),
                    Instruction::Regular(RegularInstruction::UInt8(UInt8Data(42))),
                    Instruction::Regular(RegularInstruction::UInt8(UInt8Data(43))),
                ]
            }))),
            children: vec![
                Tree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        });
    }

    #[test]
    fn disassemble_nested_tree() {
        let instructions = vec![
            Instruction::Regular(RegularInstruction::RemoteExecution(InstructionBlockData {
                length: 5,
                injected_variable_count: 0,
                injected_variables: vec![],
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

        assert_eq!(err, None);
        assert_eq!(tree, Tree {
            instruction: Box::new(Instruction::Regular(RegularInstruction::_RemoteExecutionDebugTree(InstructionBlockDataDebugTree {
                length: 5,
                injected_variable_count: 0,
                injected_variables: vec![],
                body: Tree {
                    instruction: Box::new(Instruction::Regular(RegularInstruction::Add)),
                    children: vec![
                        Tree::new(Instruction::Regular(RegularInstruction::UInt8(UInt8Data(42)))),
                        Tree::new(Instruction::Regular(RegularInstruction::UInt8(UInt8Data(43)))),
                    ]
                }
            }))),
            children: vec![
                Tree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        });
    }

}