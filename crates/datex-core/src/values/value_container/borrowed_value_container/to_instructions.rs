use crate::core_compiler::to_instructions::ToInstructions;
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;
use crate::values::value_container::borrowed_value_container::BorrowedValueContainer;
use crate::prelude::*;

impl ToInstructions for BorrowedValueContainer<'_> {
    fn to_instructions<'ctx, 'a>(
        &'a self,
        ctx: &'a mut dyn ValueVisitor<'ctx>,
    ) -> Box<dyn Iterator<Item = Instruction> + 'a>
    where
        'ctx: 'a,
    {
        match self {
            BorrowedValueContainer::Local(value) => value.to_instructions(ctx),
            BorrowedValueContainer::Shared(shared_container) => shared_container.to_instructions(ctx),
        }
    }
}