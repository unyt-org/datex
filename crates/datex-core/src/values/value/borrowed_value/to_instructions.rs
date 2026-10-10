use itertools::Itertools;
use crate::core_compiler::to_instructions::ToInstructions;
use crate::core_compiler::value_visitor::ValueVisitor;
use crate::instruction::Instruction;
use crate::values::value::borrowed_value::BorrowedValue;
use crate::prelude::*;

impl ToInstructions for BorrowedValue<'_> {
    fn to_instructions<'ctx, 'a>(&'a self, ctx: &'a mut dyn ValueVisitor<'ctx>) -> Box<dyn Iterator<Item=Instruction> + 'a>
    where
        'ctx: 'a
    {
        match self {
            BorrowedValue::Native(native) => native.to_instructions(ctx),
            BorrowedValue::Core(core) => {
                let core = core.clone_to_core_value_with_classification();
                // FIXME: get this to work without collect
                Box::new(core.to_instructions(ctx).collect_vec().into_iter())
            }
        }
    }
}