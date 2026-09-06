use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::Instruction,
    prelude::*,
    types::r#type::Type,
};

impl ToInstructions for Type {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            match self {
                Type::Entity(_) => unreachable!(),
                Type::Definition(def) => {
                    for instruction in def.to_instructions(ctx) {
                        yield instruction;
                    }
                }
            }
        })
    }
}
