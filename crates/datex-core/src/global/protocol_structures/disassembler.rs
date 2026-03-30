use alloc::rc::Rc;
use core::cell::RefCell;
use crate::dxb_parser::body::{iterate_instructions, DXBParserError};
use core::fmt::{Debug, Write};
use crate::global::protocol_structures::instruction_data::StatementsData;
use crate::global::protocol_structures::instructions::{CountOrUnbounded, Instruction};
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Tree<T> where T: Debug + Clone {
    instruction: T,
    children: Vec<Tree<T>>,
}

impl<T> Tree<T> where T: Debug + Clone {
    pub fn new(instruction: T) -> Self {
        Self {
            instruction,
            children: Vec::new(),
        }
    }
    pub fn flatten(&self) -> Vec<T> {
        let mut result = vec![self.instruction.clone()];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }

    pub fn map<N: Debug + Clone>(self, f: impl Fn(T) -> N + Clone) -> Tree<N> {
        Tree {
            instruction: f(self.instruction),
            children: self.children.into_iter().map(|child| child.map(f.clone())).collect(),
        }
    }
}

pub struct DisassemblerOptions {
    pub tree: bool,
    pub colorized: bool,
}

impl DisassemblerOptions {
    pub fn simple() -> DisassemblerOptions{
        DisassemblerOptions {
            tree: false,
            colorized: false,
        }
    }
}

impl Default for DisassemblerOptions {
    fn default() -> DisassemblerOptions {
        DisassemblerOptions {
            tree: true,
            colorized: true,
        }
    }
}

/// Converts a raw DXB body in to human-readable disassembled instructions string
pub fn disassemble_body_to_string(body: &[u8], options: DisassemblerOptions) -> String {
    let (instructions, err) = disassemble_body_to_strings(body, options.colorized);

    let mut output = "\n\n=== Disassembled DXB Body ===\n".to_string();

    if options.tree {
        disassemble_body_to_string_inner(&mut output, instructions, 0, true, true);
    }
    else {
        for instruction in instructions.flatten() {
            write!(&mut output, "{}\n", instruction).unwrap();
        }
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

fn disassemble_body_to_string_inner(
    output: &mut String,
    instructions: Tree<String>,
    indent_width: usize,
    is_root_child: bool,
    is_last_child: bool,
) {
    let indent = " ".repeat(indent_width * 3);
    let children_len = instructions.children.len();

    writeln!(
        output,
        "{}{}{}",
        indent,
        if is_root_child {""} else if is_last_child || children_len > 0 { "└─ " } else { "├─ " },
        instructions.instruction
    ).unwrap();

    for (i, child) in instructions.children.into_iter().enumerate() {
        disassemble_body_to_string_inner(output, child, if is_root_child {0} else {indent_width + 1}, false, i == children_len - 1);
    }
}


/// Converts a raw DXB body into a list of disassembled human-readable instructions
pub fn disassemble_body_to_strings(body: &[u8], colorized: bool) -> (Tree<String>, Option<DXBParserError>) {
    let (instructions, err) = disassemble_body(body);
    let instructions = instructions.map(|i| if colorized { i.to_formatted_string() } else { i.to_string() });
    (instructions, err)
}

/// Converts a raw DXB body into a list of disassembled Instruction values
pub fn disassemble_body(body: &[u8]) -> (Tree<Instruction>, Option<DXBParserError>) {
    let mut iterator = iterate_instructions(Rc::new(RefCell::new(body.to_vec())));
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

                        // println!("iknstruction {:#?}, expecting {} children", tree.instruction, match &next_expected_count {
                        //     Some(count) => count.to_string(),
                        //     None => "".to_string(),
                        // });

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
        let (instructions, err) = disassemble_body($dxb);
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
        let (instructions, err) = disassemble_body($dxb);
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
    use crate::global::protocol_structures::instruction_data::{StatementsData, UnboundedStatementsData};
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
            instruction: Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true})),
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
            instruction: Instruction::Regular(RegularInstruction::UnboundedStatements),
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
            instruction: Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true})),
            children: vec![
                Tree {
                    instruction: Instruction::Regular(RegularInstruction::UnboundedStatements),
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
            instruction: Instruction::Regular(RegularInstruction::Statements(StatementsData {statements_count: 2, terminated: true})),
            children: vec![
                Tree::new(Instruction::Regular(RegularInstruction::True)),
            ]
        },
        Some(DXBParserError::ExpectingMoreInstructions)
    )]

    fn test_disassemble_statements(
        #[case] instructions: &[Instruction],
        #[case] expected_tree: Tree<Instruction>,
        #[case] expected_err: Option<DXBParserError>,
    ) {
        let dxb = instructions_to_bytes(instructions.to_vec());
        let (tree, err) = disassemble_body(&dxb);

        assert_eq!(err, expected_err);
        assert_eq!(tree, expected_tree,)
    }

}