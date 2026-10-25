use crate::{
    core_compiler::{
        to_instructions::ToInstructions, value_visitor::ValueVisitor,
    },
    instruction::{Instruction, regular_instruction::RegularInstruction},
    prelude::*,
    values::core_values::integer::typed_integer::TypedInteger,
};
use crate::runtime::cache::shared_references_cache::SharedReferencesCache;

impl ToInstructions for TypedInteger {
    fn to_bytecode_items<'a>(
        &'a self,
        visitor: &'a mut dyn ValueVisitor,
        cache: &'a mut SharedReferencesCache,
    ) -> Box<dyn Iterator<Item = crate::instruction::BytecodeItem> + 'a> {
        Box::new(gen move {
            match self {
                TypedInteger::I8(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::I16(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::I32(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::I64(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::I128(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::U8(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::U16(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::U32(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::U64(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::U128(val) => {
                    for i in val.to_bytecode_items(visitor, cache) {
                        yield i;
                    }
                }
                TypedInteger::IBig(val) => {
                    yield RegularInstruction::big_integer(val.clone()).into()
                }
            };
        })
    }
}
