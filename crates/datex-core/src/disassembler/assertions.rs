use core::fmt::Debug;
use core::slice::Iter;
use crate::disassembler::{disassemble_body, disassemble_body_to_string, disassemble_instruction_tree_to_string, get_instruction_tree_from_list, InstructionTree};
use crate::disassembler::options::DisassemblerOptions;
use crate::global::protocol_structures::instructions::{CountOrUnbounded, Instruction, NestedInstructionResolutionStrategy};
use crate::global::protocol_structures::regular_instructions::RegularInstruction;
use crate::prelude::*;

#[derive(Debug)]
pub enum InstructionAssertionNode {
    Parent(Vec<InstructionAssertionNode>),
    Leaf(Instruction),
}

impl InstructionAssertionNode
where {
    pub fn flatten(self) -> Vec<Instruction> {
        match self {
            InstructionAssertionNode::Parent(children) => {
                let mut result = Vec::new();
                for child in children {
                    result.extend(child.flatten());
                }
                result
            }
            InstructionAssertionNode::Leaf(value) => vec![value],
        }
    }
}

impl From<Vec<InstructionAssertionNode>> for InstructionAssertionNode {
    fn from(children: Vec<InstructionAssertionNode>) -> Self {
        InstructionAssertionNode::Parent(children)
    }
}

impl From<Instruction> for InstructionAssertionNode {
    fn from(value: Instruction) -> Self {
        InstructionAssertionNode::Leaf(value)
    }
}

impl From<RegularInstruction> for InstructionAssertionNode {
    fn from(value: RegularInstruction) -> Self {
        InstructionAssertionNode::Leaf(Instruction::Regular(value))
    }
}

#[cfg(feature = "disassembler")]
#[macro_export]
macro_rules! assert_instructions_equal {
    ($dxb:expr, $expected:expr) => {{
        use $crate::global::protocol_structures::instructions::NestedInstructionResolutionStrategy;
        use $crate::disassembler::disassemble_body;

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
    ($dxb:expr, ($($expr:expr),* $(,)?)) => {{
        use $crate::disassembler::assertions::{resolve_instructions, InstructionAssertionNode, assert_instruction_lists_eq};
        let dxb = $dxb;
        assert_instruction_lists_eq(
            resolve_instructions(dxb),
            InstructionAssertionNode::Parent(vec![$($expr.into(),)*]).flatten(),
            dxb,
        );
    }};
    ($dxb:expr, $vec:expr $(,)?) => {{
        use $crate::disassembler::assertions::{resolve_instructions, assert_instruction_lists_eq};
        let dxb = $dxb;
        assert_instruction_lists_eq(
            resolve_instructions(dxb),
            $vec.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
            dxb,
        );
    }}
}

pub fn assert_instruction_lists_eq(
    output_instructions: Vec<Instruction>,
    expected_instructions: Vec<Instruction>,
    output_dxb: &[u8],
) {
    if output_instructions != expected_instructions {
        let (expected_tree, expected_err) = get_instruction_tree_from_list(expected_instructions);
        panic!(
            "Output did not match expected instructions:\n\nOutput:\n{}\n\nExpected:\n{}\n",
            disassemble_body_to_string(output_dxb, DisassemblerOptions::default()),
            disassemble_instruction_tree_to_string(expected_tree, expected_err, DisassemblerOptions::default()),
        );
    }
}

pub fn resolve_instructions(dxb: &[u8]) -> Vec<Instruction> {
    let (instructions, err) = disassemble_body(
        dxb,
        NestedInstructionResolutionStrategy::ResolveNestedScopesFlat,
    );
    if let Some(err) = err {
        panic!("Parser error: {}", err);
    }
    instructions.flatten()
}

impl RegularInstruction {
    pub fn with_children(self, children: Vec<InstructionAssertionNode>) -> InstructionAssertionNode {
        // assert that children count matches expected count
        if children.is_empty() &&
            let CountOrUnbounded::Count(count) =
                self.get_next_expected_instructions().total_count().expect("Expected count should be set for this instruction") &&
            count as usize != children.len() {
            panic!("Expected {} children for instruction {:?}, but got {}", count, self, children.len());
        }

        InstructionAssertionNode::Parent(
            vec![self.into()]
                .into_iter()
                .chain(children)
                .collect()
        )
    }

    /// Calculates the actual child count of a list of instructions, skipping nested child instructions
    fn calculate_children_count(children: &[InstructionAssertionNode]) -> u32 {
        fn visit_next_child(
            count: &mut u32,
            skip: bool,
            iterator: &mut Iter<InstructionAssertionNode>,
        ) -> Option<u32> {
            let child = match iterator.next() {
                Some(next) => next,
                None => return Some(*count),
            };

            if !skip {
                *count += 1;
            }

            // if instruction with next expected instructions, skip the next n instructions
            if let InstructionAssertionNode::Leaf(instruction) = child &&
                let Some(child_count) = instruction.get_next_expected_instructions().total_count() &&
                let CountOrUnbounded::Count(child_count) = child_count {

                for _ in 0..child_count {
                    visit_next_child(count, true, iterator);
                }
            }

            None
        }
        
        let iterator = &mut children.iter();
        let mut count = 0;
        loop {
            if let Some(count) = visit_next_child(&mut count, false, iterator) {
                return count;
            }
        }
    }

    pub fn statements_with_children(terminated: bool, children: Vec<InstructionAssertionNode>) -> InstructionAssertionNode {
         RegularInstruction::statements(RegularInstruction::calculate_children_count(&children), terminated)
             .with_children(children)
    }

    pub fn list_with_children(children: Vec<InstructionAssertionNode>) -> InstructionAssertionNode {
        RegularInstruction::list(RegularInstruction::calculate_children_count(&children))
            .with_children(children)
    }
}


#[cfg(feature = "disassembler")]
#[macro_export]
macro_rules! instructions {
    ($($expr:expr),* $(,)?) => {vec![
        $($expr.into(),)*
    ]};
}