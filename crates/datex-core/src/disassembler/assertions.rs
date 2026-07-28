use crate::{
    disassembler::{
        InstructionTree, disassemble_body, disassemble_body_to_string,
        disassemble_instruction_tree_to_string, get_instruction_tree_from_list,
        options::DisassemblerOptions,
    },
    global::protocol_structures::{
        instructions::{
            CountOrUnbounded, Instruction, NestedInstructionResolutionStrategy,
        },
        regular_instructions::RegularInstruction,
    },
    prelude::*,
};
use core::slice::Iter;

pub macro assert_instructions_equal {
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

pub macro assert_regular_instructions_equal {
    ($dxb:expr, ($($expr:expr),* $(,)?)) => {{
        use $crate::disassembler::assertions::{resolve_instructions, assert_instruction_lists_eq};
        use $crate::disassembler::{InstructionTree};
        use $crate::global::protocol_structures::instructions::Instruction;

        let dxb = $dxb;
        assert_instruction_lists_eq(
            resolve_instructions(dxb),
            InstructionTree::<Instruction>::from(vec![$(InstructionTree::<Instruction>::from($expr),)*]).flatten_instructions(),
            dxb,
        );
    }},
    ($dxb:expr, $vec:expr $(,)?) => {{
        use $crate::disassembler::assertions::{resolve_instructions, assert_instruction_lists_eq};
        use $crate::disassembler::{InstructionTree};
        use $crate::global::protocol_structures::instructions::Instruction;

        let dxb = $dxb;
        assert_instruction_lists_eq(
            resolve_instructions(dxb),
            InstructionTree::<Instruction>::from($vec.into_iter().map(|i| i.into()).collect::<Vec<_>>()).flatten_instructions(),
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
        let (expected_tree, expected_err) =
            get_instruction_tree_from_list(expected_instructions);

        panic!(
            "Output did not match expected instructions:\n\nOutput:\n{}\n\nExpected:\n{}\n",
            disassemble_body_to_string(
                output_dxb,
                DisassemblerOptions::default()
            ),
            disassemble_instruction_tree_to_string(
                expected_tree,
                expected_err,
                DisassemblerOptions::default()
            ),
        );
    }
}

/// Resolves the instructions from a DXB byte slice, panicking if there is an error
/// This is called by the [assert_regular_instructions_equal!] macro to resolve the instructions from the DXB and compare them to the expected instructions
pub fn resolve_instructions(dxb: &[u8]) -> Vec<Instruction> {
    let (instructions, err) = disassemble_body(
        dxb,
        NestedInstructionResolutionStrategy::ResolveNestedScopesFlat,
    );
    if let Some(err) = err {
        panic!(
            "Disassembly error: {}:\n{}",
            err,
            disassemble_body_to_string(dxb, DisassemblerOptions::default())
        );
    }
    instructions.flatten()
}

impl RegularInstruction {
    pub fn with_children(
        self,
        children: Vec<InstructionTree<Instruction>>,
    ) -> InstructionTree<Instruction> {
        // assert that children count matches expected count
        if children.is_empty()
            && let CountOrUnbounded::Count(count) = self
                .get_next_expected_instructions()
                .total_count()
                .expect("Expected count should be set for this instruction")
            && count as usize != children.len()
        {
            panic!(
                "Expected {} children for instruction {:?}, but got {}",
                count,
                self,
                children.len()
            );
        }

        InstructionTree::new_with_children(self.into(), children)
    }

    /// Calculates the actual child count of a list of instructions, skipping nested child instructions
    fn calculate_children_count(
        children: &[InstructionTree<Instruction>],
    ) -> u32 {
        fn visit_next_child(
            count: &mut u32,
            skip: bool,
            iterator: &mut Iter<InstructionTree<Instruction>>,
        ) -> Option<u32> {
            let current = match iterator.next() {
                Some(next) => next,
                None => return Some(*count),
            };

            if !skip {
                *count += 1;
            }

            // if instruction with next expected instructions, skip the next n instructions
            if current.children().is_empty()
                && let Some(child_count) = current
                    .instruction()
                    .get_next_expected_instructions()
                    .total_count()
                && let CountOrUnbounded::Count(child_count) = child_count
            {
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

    pub fn statements_with_children(
        terminated: bool,
        children: Vec<InstructionTree<Instruction>>,
    ) -> InstructionTree<Instruction> {
        RegularInstruction::statements(
            RegularInstruction::calculate_children_count(&children),
            terminated,
        )
        .with_children(children)
    }

    pub fn list_with_children(
        children: Vec<InstructionTree<Instruction>>,
    ) -> InstructionTree<Instruction> {
        RegularInstruction::list(RegularInstruction::calculate_children_count(
            &children,
        ))
        .with_children(children)
    }
}

pub macro instructions {
    ($($expr:expr),* $(,)?) => {vec![
        $($expr.into(),)*
    ]}
}
