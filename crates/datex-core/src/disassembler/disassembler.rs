use crate::{
    disassembler::options::DisassemblerOptions,
    dxb_parser::body::{DXBParserError, iterate_instructions},
    global::{
        instruction_codes::InstructionCode,
        protocol_structures::{
            instruction_data::InstructionBlockDataDebugFlat,
            instructions::{
                CountOrUnbounded, Instruction,
                NestedInstructionResolutionStrategy,
            },
            regular_instructions::RegularInstructionData,
        },
    },
    prelude::*,
    utils::ansi_colors::{AnsiColor, AnsiWrite},
};
use alloc::{rc::Rc, vec::IntoIter};
use core::{
    cell::RefCell,
    fmt::{Debug, Write},
};
use serde::Serialize;

/// A generic tree structure for instructions with child instructions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InstructionTree<T>
where
    T: Debug + Clone,
{
    instruction: Box<T>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<InstructionTree<T>>,
}
impl Default for InstructionTree<Instruction> {
    fn default() -> Self {
        InstructionTree::new(Instruction::Regular(
            RegularInstructionData::UnboundedStatements,
        ))
    }
}

impl<T> From<T> for InstructionTree<T>
where
    T: Debug + Clone,
{
    fn from(instruction: T) -> Self {
        InstructionTree::new(instruction)
    }
}

impl From<RegularInstructionData> for InstructionTree<Instruction> {
    fn from(instruction: RegularInstructionData) -> Self {
        InstructionTree::new(Instruction::Regular(instruction))
    }
}

impl From<Vec<InstructionTree<Instruction>>> for InstructionTree<Instruction> {
    fn from(mut instruction_trees: Vec<InstructionTree<Instruction>>) -> Self {
        if instruction_trees.len() == 1 {
            instruction_trees.remove(0)
        } else {
            fn visit_next_child(
                parent: &mut InstructionTree<Instruction>,
                iterator: &mut IntoIter<InstructionTree<Instruction>>,
            ) -> bool {
                let mut current = match iterator.next() {
                    Some(next) => next,
                    None => return false,
                };

                // if instruction with next expected instructions, skip the next n instructions
                if current.children().is_empty()
                    && let Some(child_count) = current
                        .instruction()
                        .get_next_expected_instructions()
                        .total_count()
                    && let CountOrUnbounded::Count(child_count) = child_count
                {
                    for _ in 0..child_count {
                        if !visit_next_child(&mut current, iterator) {
                            panic!(
                                "Expected {} children for instruction {:?}, but got fewer",
                                child_count,
                                current.instruction()
                            );
                        }
                    }
                }

                parent.children.push(current);

                true
            }

            let mut iterator = instruction_trees.into_iter();

            let mut root: InstructionTree<Instruction> =
                iterator.next().unwrap();
            if !root.children().is_empty() {
                panic!("Multiple root nodes found in instruction tree.");
            }

            while visit_next_child(&mut root, &mut iterator) {}

            root
        }
    }
}

impl InstructionTree<Instruction> {
    /// Flattens the tree into a list of instructions,
    /// also recursively flattens [RegularInstruction::_RemoteExecutionDebugTree]
    /// into [RegularInstruction::_RemoteExecutionDebugFlat]
    pub fn flatten_instructions(self) -> Vec<Instruction> {
        if let Instruction::Regular(
            RegularInstructionData::_RemoteExecutionDebugTree(tree),
        ) = *self.instruction
        {
            vec![Instruction::Regular(
                RegularInstructionData::_RemoteExecutionDebugFlat(
                    InstructionBlockDataDebugFlat {
                        length: tree.length,
                        injected_variable_count: tree.injected_variable_count,
                        injected_values: tree.injected_values,
                        body: tree.body.flatten_instructions(),
                    },
                ),
            )]
        } else {
            let mut result = vec![*self.instruction];
            for child in self.children {
                result.extend(child.flatten_instructions());
            }
            result
        }
    }
}

impl<T> InstructionTree<T>
where
    T: Debug + Clone,
{
    /// Create a new tree with a root instruction
    pub fn new(instruction: T) -> Self {
        Self {
            instruction: Box::new(instruction),
            children: Vec::new(),
        }
    }

    /// Create a new tree with a root instruction and children
    pub fn new_with_children(
        instruction: T,
        children: Vec<InstructionTree<T>>,
    ) -> Self {
        Self {
            instruction: Box::new(instruction),
            children,
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
    pub fn map<N: Debug + Clone>(
        self,
        f: impl Fn(T) -> N + Clone,
    ) -> InstructionTree<N> {
        InstructionTree {
            instruction: Box::new(f(*self.instruction)),
            children: self
                .children
                .into_iter()
                .map(|child| child.map(f.clone()))
                .collect(),
        }
    }

    pub fn children(&self) -> &Vec<InstructionTree<T>> {
        &self.children
    }

    pub fn instruction(&self) -> &T {
        &self.instruction
    }
}

/// An instruction tree containing an optional detailed instruction tree inside each node
#[derive(Debug, Clone)]
struct DetailedInstructionTree(
    pub InstructionTree<(Instruction, Option<Box<DetailedInstructionTree>>)>,
);

#[derive(Default, Clone, Debug, PartialEq)]
pub enum InnerInstructions<'a> {
    #[default]
    None,
    Flat(&'a Vec<Instruction>),
    Tree(&'a InstructionTree<Instruction>),
}

/// Converts a raw DXB body in to human-readable disassembled instructions string
pub(super) fn disassemble_body_to_string(
    body: &[u8],
    options: DisassemblerOptions,
) -> String {
    let (instructions, err) = disassemble_body(
        body,
        options.nested_instructions_resolution_strategy(),
    );
    disassemble_instruction_tree_to_string(instructions, err, options)
}

/// Converts an instruction tree in to human-readable disassembled instructions string
pub fn disassemble_instruction_tree_to_string(
    instruction_tree: InstructionTree<Instruction>,
    error: Option<DXBParserError>,
    options: DisassemblerOptions,
) -> String {
    let instructions = instruction_tree_to_detailed_tree(instruction_tree);

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
            0,
        );
    } else {
        write_flat_instructions(
            &mut output,
            instructions,
            options.colorized,
            0,
        );
    }

    if let Some(err) = error {
        if options.colorized {
            write!(
                &mut output,
                "\x1b[38;2;245;39;60m\n[!] Parser Error: {}\x1b[0m",
                err
            )
            .unwrap();
        } else {
            write!(&mut output, "[!] Parser Error: {}", err).unwrap();
        }
    }

    output
}

/// Converts a raw DXB body into a list of disassembled Instruction values
pub fn disassemble_body(
    body: &[u8],
    nested_instruction_resolution_strategy: NestedInstructionResolutionStrategy,
) -> (InstructionTree<Instruction>, Option<DXBParserError>) {
    let mut iterator = iterate_instructions(
        Rc::new(RefCell::new(body.to_vec())),
        nested_instruction_resolution_strategy,
    );
    get_instruction_tree(&mut iterator)
}

/// Converts a list of Instruction values into an instruction tree
pub fn get_instruction_tree_from_list(
    instructions: Vec<Instruction>,
) -> (InstructionTree<Instruction>, Option<DXBParserError>) {
    let mut iterator = instructions.into_iter().map(Ok);
    get_instruction_tree(&mut iterator)
}

/// Converts an instruction iterator into a list of disassembled Instruction values
pub fn get_instruction_tree(
    instructions: impl Iterator<Item = Result<Instruction, DXBParserError>>,
) -> (InstructionTree<Instruction>, Option<DXBParserError>) {
    let mut tree = InstructionTree::new(Instruction::Regular(
        RegularInstructionData::UnboundedStatements,
    )); // initial tree root, gets overridden
    let err = disassemble_body_inner(
        &mut instructions.into_iter(),
        &mut tree,
        CountOrUnbounded::UnboundedStart,
        true,
    );
    (tree, err)
}

/// Writes a detailed instruction tree to an output string recursively with optional colorization and indentation
fn write_flat_instructions(
    output: &mut String,
    instructions: DetailedInstructionTree,
    colorized: bool,
    level: u32,
) {
    for (instruction, inner_instructions) in instructions.0.flatten() {
        write!(output, "{}", " ".repeat(level as usize * 2),).unwrap();
        write_instruction(output, &instruction, level, colorized);
        if let Some(inner_instructions) = inner_instructions {
            write_flat_instructions(
                output,
                *inner_instructions,
                colorized,
                level + 1,
            );
        }
    }
}

/// Writes a single instruction to an output string with optional colorization and indentation
fn write_instruction(
    output: &mut String,
    instruction: &Instruction,
    level: u32,
    colorized: bool,
) {
    if colorized {
        let color = color_for_level(level);
        output.write_fg(color);
    }

    write!(
        output,
        "{}",
        match instruction {
            Instruction::Regular(instr) => instr.instruction_code().to_string(),
            Instruction::Type(instr) => instr.as_ref().to_string(),
        }
    )
    .unwrap();

    if colorized {
        output.write_reset();
    }

    if let Some(metadata_string) = instruction.metadata_string() {
        write!(output, " {}", metadata_string).unwrap();
    }

    writeln!(output).unwrap();
}

/// Returns an instruction text color for a given level
fn color_for_level(level: u32) -> AnsiColor {
    match level % 10 {
        0 => AnsiColor::Rgb(0, 153, 204), // deep sky blue
        1 => AnsiColor::Rgb(0, 204, 153), // teal
        2 => AnsiColor::Rgb(51, 204, 102), // green
        3 => AnsiColor::Rgb(153, 255, 51), // lime green
        4 => AnsiColor::Rgb(255, 221, 51), // golden yellow
        5 => AnsiColor::Rgb(204, 204, 255), // light periwinkle
        6 => AnsiColor::Rgb(153, 153, 255), // soft purple
        7 => AnsiColor::Rgb(153, 102, 204), // medium purple
        8 => AnsiColor::Rgb(255, 153, 204), // pink
        9 => AnsiColor::Rgb(255, 204, 229), // soft pink / rose
        _ => AnsiColor::Rgb(200, 200, 200), // neutral fallback
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
        if is_root_child {
            ""
        } else if is_inner {
            "↳  "
        } else if is_last_child || children_len > 0 {
            "└─ "
        } else {
            "├─ "
        },
    )
    .unwrap();

    write_instruction(output, &main, level, colorized);

    if let Some(inner) = inner {
        disassemble_body_to_string_inner(
            output,
            *inner,
            if is_root_child { 0 } else { indent_width + 1 },
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
            if is_root_child { 0 } else { indent_width + 1 },
            false,
            i == children_len - 1,
            false,
            colorized,
            level,
        );
    }
}

fn instruction_tree_to_detailed_tree(
    instructions: InstructionTree<Instruction>,
) -> DetailedInstructionTree {
    DetailedInstructionTree(instructions.map(|i| {
        let inner = get_inner_instructions_as_detailed_tree(&i).map(Box::new);
        (i, inner)
    }))
}

fn get_inner_instructions_as_detailed_tree(
    instruction: &Instruction,
) -> Option<DetailedInstructionTree> {
    match instruction {
        Instruction::Regular(
            RegularInstructionData::_RemoteExecutionDebugFlat(data),
        ) => {
            let (tree, err) = get_instruction_tree_from_list(data.body.clone());
            if let Some(err) = err {
                panic!("{}", err);
            }
            Some(instruction_tree_to_detailed_tree(tree))
        }
        Instruction::Regular(
            RegularInstructionData::_RemoteExecutionDebugTree(data),
        ) => Some(instruction_tree_to_detailed_tree(data.body.clone())),
        _ => None,
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
                    }
                    Ok(instruction) => {
                        // get next expected children
                        let next_expected_count = instruction
                            .get_next_expected_instructions()
                            .total_count();
                        let mut tree = InstructionTree::new(instruction);

                        let err = match next_expected_count {
                            Some(next_expected_count) => {
                                match next_expected_count {
                                    CountOrUnbounded::UnboundedEnd => {
                                        parent.children.push(tree);
                                        return None;
                                    }
                                    _ => disassemble_body_inner(
                                        iterator,
                                        &mut tree,
                                        next_expected_count,
                                        false,
                                    ),
                                }
                            }
                            None => None,
                        };

                        // if root node, replace parent with first instruction
                        if is_root {
                            *parent = tree;
                        } else {
                            parent.children.push(tree);
                        }

                        if let Some(err) = err {
                            return Some(err);
                        }

                        // all expected children collected
                        if let CountOrUnbounded::Count(expected_count) =
                            count_or_unbounded
                            && parent.children.len() as u32 >= expected_count
                        {
                            break;
                        }
                    }
                }
            }
            None => break,
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core_compiler::value_compiler::append_instruction,
        global::protocol_structures::{
            instruction_data::{
                InstructionBlockData, InstructionBlockDataDebugFlat,
                InstructionBlockDataDebugTree, StatementsData, UInt8Data,
                UnboundedStatementsData,
            },
            regular_instructions::RegularInstructionData,
        },
        runtime::{Runtime, RuntimeConfig, RuntimeRunner},
    };
    use binrw::io::Cursor;
    use test_case::test_case;

    fn instructions_to_bytes(instructions: Vec<Instruction>) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        for instruction in instructions {
            append_instruction(&mut cursor, instruction);
        }
        cursor.into_inner()
    }

    #[test_case(
        &[],
        InstructionTree::new(Instruction::Regular(RegularInstructionData::UnboundedStatements)),
        Some(DXBParserError::ExpectingMoreInstructions)
         ; "empty dxb")]
    #[test_case(
        &[
            Instruction::Regular(RegularInstructionData::True),
            Instruction::Regular(RegularInstructionData::False),
        ],
        InstructionTree::new(Instruction::Regular(RegularInstructionData::True)),
        Some(DXBParserError::UnexpectedBytesAfterEndOfInstructions)
         ; "multiple root nodes")]
    #[test_case(
        &[
            Instruction::Regular(RegularInstructionData::Statements(StatementsData {statements_count: 2, terminated: true})),
            Instruction::Regular(RegularInstructionData::True),
            Instruction::Regular(RegularInstructionData::False),
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstructionData::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstructionData::True)),
                InstructionTree::new(Instruction::Regular(RegularInstructionData::False)),
            ]
        },
        None
         ; "simple statements"
    )]
    #[test_case(
        &[
            Instruction::Regular(RegularInstructionData::UnboundedStatements),
            Instruction::Regular(RegularInstructionData::True),
            Instruction::Regular(RegularInstructionData::False),
            Instruction::Regular(RegularInstructionData::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})),
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstructionData::UnboundedStatements)),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstructionData::True)),
                InstructionTree::new(Instruction::Regular(RegularInstructionData::False)),
                InstructionTree::new(Instruction::Regular(RegularInstructionData::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})))
            ]
        },
        None
         ; "unbounded statements"
    )]
    #[test_case(
        &[
            Instruction::Regular(RegularInstructionData::Statements(StatementsData {statements_count: 2, terminated: true})),
            Instruction::Regular(RegularInstructionData::UnboundedStatements),
            Instruction::Regular(RegularInstructionData::True),
            Instruction::Regular(RegularInstructionData::False),
            Instruction::Regular(RegularInstructionData::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})),
            Instruction::Regular(RegularInstructionData::Null),
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstructionData::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                InstructionTree {
                    instruction: Box::new(Instruction::Regular(RegularInstructionData::UnboundedStatements)),
                    children: vec![
                        InstructionTree::new(Instruction::Regular(RegularInstructionData::True)),
                        InstructionTree::new(Instruction::Regular(RegularInstructionData::False)),
                        InstructionTree::new(Instruction::Regular(RegularInstructionData::UnboundedStatementsEnd(UnboundedStatementsData {terminated: false})))
                    ]
                },
                InstructionTree::new(Instruction::Regular(RegularInstructionData::Null)),
            ]
        },
        None
        ; "normal and unbounded statements"
    )]
    #[test_case(
        &[
            Instruction::Regular(RegularInstructionData::True),
        ],
        InstructionTree::new(Instruction::Regular(RegularInstructionData::True)),
        None
        ; "single instruction"
    )]
    #[test_case(
        &[
            Instruction::Regular(RegularInstructionData::Statements(StatementsData {statements_count: 2, terminated: true})),
            Instruction::Regular(RegularInstructionData::True),
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstructionData::Statements(StatementsData {statements_count: 2, terminated: true}))),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstructionData::True)),
            ]
        },
        Some(DXBParserError::ExpectingMoreInstructions)
        ; "statements with missing instructions"
    )]
    #[test_case(
        &[
            Instruction::Regular(RegularInstructionData::RemoteExecution(InstructionBlockData {
                length: 2,
                injected_value_count: 0,
                injected_values: vec![],
                body: vec![
                    InstructionCode::UINT_8 as u8,
                    42,
                ]
            })),
            Instruction::Regular(RegularInstructionData::True)
        ],
        InstructionTree {
            instruction: Box::new(Instruction::Regular(RegularInstructionData::RemoteExecution(InstructionBlockData {
                length: 2,
                injected_value_count: 0,
                injected_values: vec![],
                body: vec![
                    InstructionCode::UINT_8 as u8,
                    42,
                ]
            }))),
            children: vec![
                InstructionTree::new(Instruction::Regular(RegularInstructionData::True)),
            ]
        },
        None
        ; "remote execution"
    )]

    fn disassemble_statements(
        instructions: &[Instruction],
        expected_tree: InstructionTree<Instruction>,
        expected_err: Option<DXBParserError>,
    ) {
        let dxb = instructions_to_bytes(instructions.to_vec());
        let (tree, err) = disassemble_body(
            &dxb,
            NestedInstructionResolutionStrategy::default(),
        );

        assert_eq!(err, expected_err);
        assert_eq!(tree, expected_tree,)
    }

    #[test]
    fn disassemble_nested_flat() {
        let instructions = vec![
            Instruction::Regular(RegularInstructionData::RemoteExecution(
                InstructionBlockData {
                    length: 5,
                    injected_value_count: 0,
                    injected_values: vec![],
                    body: vec![
                        InstructionCode::ADD as u8,
                        InstructionCode::UINT_8 as u8,
                        42,
                        InstructionCode::UINT_8 as u8,
                        43,
                    ],
                },
            )),
            Instruction::Regular(RegularInstructionData::True),
        ];
        let dxb = instructions_to_bytes(instructions.to_vec());
        let (tree, err) = disassemble_body(
            &dxb,
            NestedInstructionResolutionStrategy::ResolveNestedScopesFlat,
        );

        assert_eq!(err, None);
        assert_eq!(
            tree,
            InstructionTree {
                instruction: Box::new(Instruction::Regular(
                    RegularInstructionData::_RemoteExecutionDebugFlat(
                        InstructionBlockDataDebugFlat {
                            length: 5,
                            injected_variable_count: 0,
                            injected_values: vec![],
                            body: vec![
                                Instruction::Regular(
                                    RegularInstructionData::Add
                                ),
                                Instruction::Regular(
                                    RegularInstructionData::UInt8(UInt8Data(
                                        42
                                    ))
                                ),
                                Instruction::Regular(
                                    RegularInstructionData::UInt8(UInt8Data(
                                        43
                                    ))
                                ),
                            ]
                        }
                    )
                )),
                children: vec![InstructionTree::new(Instruction::Regular(
                    RegularInstructionData::True
                )),]
            }
        );
    }

    #[test]
    fn disassemble_nested_tree() {
        let instructions = vec![
            Instruction::Regular(RegularInstructionData::RemoteExecution(
                InstructionBlockData {
                    length: 5,
                    injected_value_count: 0,
                    injected_values: vec![],
                    body: vec![
                        InstructionCode::ADD as u8,
                        InstructionCode::UINT_8 as u8,
                        42,
                        InstructionCode::UINT_8 as u8,
                        43,
                    ],
                },
            )),
            Instruction::Regular(RegularInstructionData::True),
        ];
        let dxb = instructions_to_bytes(instructions.to_vec());
        let (tree, err) = disassemble_body(
            &dxb,
            NestedInstructionResolutionStrategy::ResolveNestedScopesTree,
        );

        assert_eq!(err, None);
        assert_eq!(
            tree,
            InstructionTree {
                instruction: Box::new(Instruction::Regular(
                    RegularInstructionData::_RemoteExecutionDebugTree(
                        InstructionBlockDataDebugTree {
                            length: 5,
                            injected_variable_count: 0,
                            injected_values: vec![],
                            body: InstructionTree {
                                instruction: Box::new(Instruction::Regular(
                                    RegularInstructionData::Add
                                )),
                                children: vec![
                                    InstructionTree::new(Instruction::Regular(
                                        RegularInstructionData::UInt8(
                                            UInt8Data(42)
                                        )
                                    )),
                                    InstructionTree::new(Instruction::Regular(
                                        RegularInstructionData::UInt8(
                                            UInt8Data(43)
                                        )
                                    )),
                                ]
                            }
                        }
                    )
                )),
                children: vec![InstructionTree::new(Instruction::Regular(
                    RegularInstructionData::True
                )),]
            }
        );
    }

    #[ignore]
    #[cfg(all(feature = "std", feature = "compiler"))]
    #[test]
    fn disassemble_string_test() {
        use crate::compiler::{CompileOptions, compile_script};

        let script = r#"
            var x = 5;
            var y = 42;
            @example :: (
                1;2;3;
                @test :: (1 + 2);
            )
        "#;
        let dxb =
            compile_script(script, CompileOptions::default(), Runtime::stub())
                .unwrap()
                .0;
        println!(
            "{}",
            disassemble_body_to_string(
                &dxb,
                DisassemblerOptions {
                    tree: true,
                    colorized: true,
                    recursive: true,
                }
            )
        );

        println!(
            "{}",
            disassemble_body_to_string(
                &dxb,
                DisassemblerOptions {
                    tree: false,
                    colorized: true,
                    recursive: true,
                }
            )
        );

        println!(
            "{}",
            disassemble_body_to_string(
                &dxb,
                DisassemblerOptions {
                    tree: true,
                    colorized: true,
                    recursive: false,
                }
            )
        );
    }
}
