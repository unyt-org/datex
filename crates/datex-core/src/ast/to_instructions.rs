use core::str::FromStr;
use crate::ast::expressions::{Apply, BinaryOperation, ComparisonOperation, CreateShared, DatexExpressionData, DeriveRef, DeriveSharedRef, GenericInstantiation, InterfaceMethodCall, List, Map, PropertyAccess, PropertyAssignment, RangeDeclaration, RequestSharedRef, RootPropertyAccess, TagExpression, UnaryOperation, Unbox, UnboxAssignment};
use crate::core_compiler::to_instructions::{InstructionContext, ToInstructions};
use crate::global::operators::ModificationOperator;
use crate::global::root_properties::RootProperty;
use crate::instruction::regular_instruction::RegularInstruction;
use crate::shared_values::{ReferenceMutability, SharedContainerMutability};

impl ToInstructions for RangeDeclaration {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::range();
            for instruction in self.start.to_instructions(ctx) {
                yield instruction;
            }
            for instruction in self.end.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}

impl ToInstructions for ComparisonOperation {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::comparison_operation(self.operator);
            for instruction in self.left.to_instructions(ctx) {
                yield instruction;
            }
            for instruction in self.right.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for UnboxAssignment {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            match self.operator {
                Some(operator) => match operator {
                    ModificationOperator::AddAssign => {
                        yield RegularInstruction::increment()
                    }
                    ModificationOperator::SubtractAssign => {
                        yield RegularInstruction::decrement()
                    }
                    _ => todo!("Generate x = x * z instructions;"),
                },
                None => yield RegularInstruction::set_shared_container_value(),
            };

            // compile assigned expression
            for instruction in self.assigned_expression.to_instructions(ctx) {
                yield instruction;
            }

            // compile unbox expression
            for instruction in self.unbox_expression.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for PropertyAssignment {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            let PropertyAssignment {
                base,
                property,
                assigned_expression,
                ..
            } = self;
            // depending on the key, handle different property assignments

            match &property.data() {
                DatexExpressionData::Text(key)
                if key.0.len() <= u8::MAX as usize =>
                    {
                        yield RegularInstruction::set_entry_text(key.0.clone());
                    }

                DatexExpressionData::Integer(index)
                if let Some(index) = index.as_u32() =>
                    {
                        yield RegularInstruction::set_entry_index(index);
                    }

                _ => {
                    for instruction in property.to_instructions(ctx) {
                        yield instruction;
                    }
                }
            }
            // compile assigned expression
            for instruction in assigned_expression.to_instructions(ctx) {
                yield instruction;
            }

            // compile base expression
            for instruction in base.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for UnaryOperation {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::unary_operation(self.operator);
            for instruction in self.expression.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for Apply {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::apply(self.arguments.len() as u8);
            // compile arguments
            for argument in &self.arguments {
                for instruction in argument.to_instructions(ctx) {
                    yield instruction;
                }
            }
            // compile function expression
            for instruction in self.base.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for InterfaceMethodCall {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            // TODO: replace with trait impls
            match self.method_name.as_str() {
                "append" => {
                    yield RegularInstruction::append_entry();

                    for argument in &self.arguments {
                        for instruction in argument.to_instructions(ctx) {
                            yield instruction;
                        }
                    }
                }

                "clear" => {
                    yield RegularInstruction::clear();
                }

                "splice" => {
                    yield RegularInstruction::splice_dynamic();

                    for argument in &self.arguments {
                        for instruction in argument.to_instructions(ctx) {
                            yield instruction;
                        }
                    }
                }

                _ => {
                    yield RegularInstruction::call_method(
                        self.method_name.clone(),
                        self.arguments.len() as u8,
                    );

                    for argument in &self.arguments {
                        for instruction in argument.to_instructions(ctx) {
                            yield instruction;
                        }
                    }
                }
            }

            // compile target expression
            for instruction in self.target.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for PropertyAccess {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            // depending on the key, handle different property accesses
            match self.property.data() {
                // simple text key if length fits in u8
                DatexExpressionData::Text(key) if key.0.len() <= 255 => {
                    yield RegularInstruction::get_entry_text(key.0.clone());
                }
                // index access if integer fits in u32
                DatexExpressionData::Integer(index)
                if let Some(index) = index.as_u32() =>
                    {
                        yield RegularInstruction::get_entry_index(index);
                    }
                _ => {
                    yield RegularInstruction::get_entry_dynamic();
                    for instruction in self.property.to_instructions(ctx) {
                        yield instruction;
                    }
                }
            }

            // compile base expression
            for instruction in self.base.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for GenericInstantiation {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            // NOTE: might already be handled in type compilation
            todo!()
        })
    }
}

impl ToInstructions for RequestSharedRef {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(core::iter::once(RegularInstruction::get_shared_ref(
            self.address.clone(),
            &self.mutability,
        )))
    }
}

impl ToInstructions for List {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::list(self.items.len() as u32);
            for item in &self.items {
                for instruction in item.to_instructions(ctx) {
                    yield instruction;
                }
            }
        })
    }
}


impl ToInstructions for Map {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::map(self.entries.len() as u32);
            for (key, value) in &self.entries {
                match &*key.data {
                    // text -> insert key string
                    DatexExpressionData::Text(text) => {
                        if text.len() < 256 {
                            yield RegularInstruction::key_value_short_text(
                                text.0.clone(),
                            );
                        } else {
                            yield RegularInstruction::key_value_dynamic();
                            yield RegularInstruction::text(text.0.clone());
                        }
                    }
                    // other -> insert key as dynamic
                    _ => {
                        yield RegularInstruction::key_value_dynamic();
                        for instruction in key.to_instructions(ctx) {
                            yield instruction;
                        }
                    }
                };

                // value
                for instruction in value.to_instructions(ctx) {
                    yield instruction;
                }
            }
        })
    }
}

impl ToInstructions for TagExpression {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::tagged_value(
                self.tag.clone(),
                self.expression.is_none(),
            );
            if let Some(expression) = &self.expression {
                for instruction in expression.to_instructions(ctx) {
                    yield instruction;
                }
            }
        })
    }
}

impl ToInstructions for RootPropertyAccess {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            let root_property = RootProperty::from_str(&self.property_name)
                .expect("invalid root property name");
            yield RegularInstruction::get_root_property(root_property);
        })
    }
}


impl ToInstructions for Unbox {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::unbox();
            for instruction in self.expression.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for DeriveRef {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            for instruction in self.expression.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for CreateShared {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            match self.mutability {
                SharedContainerMutability::Immutable => {
                    yield RegularInstruction::create_shared();
                }
                SharedContainerMutability::Mutable => {
                    yield RegularInstruction::create_shared_mut();
                }
            }

            for instruction in self.expression.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}


impl ToInstructions for DeriveSharedRef {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            match self.mutability {
                ReferenceMutability::Immutable => {
                    yield RegularInstruction::derive_shared_reference();
                }
                ReferenceMutability::Mutable => {
                    yield RegularInstruction::derive_shared_reference_mut();
                }
            }

            for instruction in self.expression.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}

impl ToInstructions for BinaryOperation {
    type InstructionType = RegularInstruction;

    fn to_instructions<'tracking, 'ctx, 'iter>(
        &'iter self,
        ctx: &'iter InstructionContext<'tracking, 'ctx>,
    ) -> Box<impl Iterator<Item = Self::InstructionType> + 'iter> {
        Box::new(gen move {
            yield RegularInstruction::binary_operation(self.operator);
            for instruction in self.left.to_instructions(ctx) {
                yield instruction;
            }
            for instruction in self.right.to_instructions(ctx) {
                yield instruction;
            }
        })
    }
}