use crate::{
    core_compiler::to_instructions::ToInstructions,
    global::protocol_structures::type_instructions::TypeInstruction,
    types::r#type::Type,
};

impl ToInstructions for Type {
    type InstructionType = TypeInstruction;

    fn to_instructions(
        &self,
    ) -> Box<dyn Iterator<Item = Self::InstructionType> + '_> {
        Box::new(gen {
            match self {
                Type::Nominal(_) => unreachable!(),
                Type::Alias(def) => {
                    yield TypeInstruction::TypeInstructionWithMetadata(
                        def.metadata.clone(),
                    );
                    for instruction in def.to_instructions() {
                        yield instruction;
                    }
                }
            }
        })
    }
}
