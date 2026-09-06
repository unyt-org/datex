use crate::{
    instruction::Instruction,
    prelude::*,
    preludes::derive::{ToInstructions, ValueVisitor},
    shared_values::OwnedSharedContainer,
};

impl ToInstructions for OwnedSharedContainer {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        _ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        Box::new(gen move {
            todo!("Implement to_instructions for OwnedSharedContainer")
        })
    }
}
