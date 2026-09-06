use datex_core::{
    core_compiler::{
        core_compilation_context::{CompileInput, CoreCompilationContext},
        to_instructions::ToInstructions,
        value_visitor::ValueVisitor,
    },
    disassembler::assertions::{
        assert_instruction_lists_eq, assert_instructions_equal, instructions,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    runtime::pointer_availability_lookup::PointerAvailabilityLookup,
};
use datex_macros_internal::Datex;
#[derive(Datex, Debug)]
#[datex(structural)]
struct ExampleStruct {
    a: u8,
    b: String,
}

fn to_instructions<T: ToInstructions>(structure: &T) -> Vec<Instruction> {
    let pointer_lookup = PointerAvailabilityLookup::default();
    let mut context = CoreCompilationContext::new(
        vec![],
        CompileInput::new(&pointer_lookup, &[]),
    );
    structure.to_instructions(&mut context).collect::<Vec<_>>()
}

#[test]
fn example_struct_to_instructions() {
    let structure = ExampleStruct {
        a: 42u8,
        b: "Test".to_string(),
    };
    assert_instruction_lists_eq!(
        to_instructions(&structure),
        (RegularInstruction::map(2).with_children(instructions!(
            // a
            RegularInstruction::text("a".to_string()),
            RegularInstruction::uint8(42),
            // b
            RegularInstruction::text("b".to_string()),
            RegularInstruction::text("Test".to_string()),
        )))
    )
}
