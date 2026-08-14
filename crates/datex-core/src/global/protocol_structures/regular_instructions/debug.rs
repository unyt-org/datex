use crate::{
    disassembler::{InnerInstructions, InstructionTree},
    dxb_parser::body::{DXBParserError, InstructionWithSpan},
    global::{
        instruction_codes::InstructionCode,
        protocol_structures::{
            instruction_data::{
                CallableData, CallableDataBody, CallableDataBodyDebugFlat,
                CallableDataBodyDebugTree, CallableDataDebugFlat,
                CallableDataDebugTree, CallableDeclarationData,
                CallableDeclarationDataDebugFlat,
                CallableDeclarationDataDebugTree, InstructionBlockData,
                InstructionBlockDataDebugFlat, InstructionBlockDataDebugTree,
            },
            instructions::NestedInstructionResolutionStrategy,
            regular_instructions::RegularInstruction,
        },
    },
    prelude::*,
};

impl RegularInstruction {
    pub fn remote_execution_debug_tree(
        tree: InstructionBlockDataDebugTree,
    ) -> Self {
        RegularInstruction::_RemoteExecutionDebugTree(tree)
    }

    pub fn remote_execution_debug_flat(
        tree: InstructionBlockDataDebugFlat,
    ) -> Self {
        RegularInstruction::_RemoteExecutionDebugFlat(tree)
    }

    /// Maps special debug instruction variants to their corresponding instruction codes.
    /// Instruction codes for normal instruction variants are set by the `#[magic]` attribute and are not included here.
    pub(crate) fn debug_instruction_code(&self) -> Option<InstructionCode> {
        match self {
            RegularInstruction::_RemoteExecutionDebugTree(_)
            | RegularInstruction::_RemoteExecutionDebugFlat(_) => {
                Some(InstructionCode::REMOTE_EXECUTION)
            }
            RegularInstruction::_CallableDeclarationDebugTree(_)
            | RegularInstruction::_CallableDeclarationDebugFlat(_) => {
                Some(InstructionCode::CALLABLE_DECLARATION)
            }
            RegularInstruction::_CallableDebugTree(_)
            | RegularInstruction::_CallableDebugFlat(_) => {
                Some(InstructionCode::CALLABLE)
            }
            _ => None,
        }
    }

    pub fn inner_instructions_from_debug_instruction(
        &self,
    ) -> InnerInstructions<'_> {
        match self {
            RegularInstruction::_RemoteExecutionDebugTree(data) => {
                InnerInstructions::Tree(&data.body)
            }
            RegularInstruction::_RemoteExecutionDebugFlat(data) => {
                InnerInstructions::Flat(&data.body)
            }
            RegularInstruction::_CallableDeclarationDebugTree(data) => {
                InnerInstructions::Tree(&data.body.body)
            }
            RegularInstruction::_CallableDeclarationDebugFlat(data) => {
                InnerInstructions::Flat(&data.body.body)
            }
            RegularInstruction::_CallableDebugTree(data) => {
                InnerInstructions::Tree(&data.body.body)
            }
            RegularInstruction::_CallableDebugFlat(data) => {
                InnerInstructions::Flat(&data.body.body)
            }
            _ => InnerInstructions::None,
        }
    }

    pub fn inner_instructions(&self) -> Option<&[u8]> {
        match self {
            RegularInstruction::RemoteExecution(data) => Some(&data.body),
            RegularInstruction::CallableDeclaration(data) => {
                Some(&data.body.body)
            }
            RegularInstruction::Callable(data) => Some(&data.body.body),
            _ => None,
        }
    }

    pub fn get_debug_tree_instruction(
        self,
        instructions: InstructionTree<InstructionWithSpan>,
    ) -> Option<Self> {
        match self {
            RegularInstruction::RemoteExecution(tree) => {
                Some(RegularInstruction::_RemoteExecutionDebugTree(
                    InstructionBlockDataDebugTree {
                        length: tree.length,
                        injected_variable_count: tree.injected_value_count,
                        injected_values: tree.injected_values,
                        body: instructions,
                    },
                ))
            }
            RegularInstruction::CallableDeclaration(tree) => {
                Some(RegularInstruction::_CallableDeclarationDebugTree(
                    CallableDeclarationDataDebugTree {
                        signature: tree.signature,
                        body: InstructionBlockDataDebugTree {
                            length: tree.body.length,
                            injected_variable_count: tree
                                .body
                                .injected_value_count,
                            injected_values: tree.body.injected_values,
                            body: instructions,
                        },
                    },
                ))
            }
            RegularInstruction::Callable(tree) => Some(
                RegularInstruction::_CallableDebugTree(CallableDataDebugTree {
                    signature: tree.signature,
                    body: CallableDataBodyDebugTree {
                        length: tree.body.length,
                        injected_value_count: tree.body.injected_value_count,
                        body: instructions,
                    },
                }),
            ),
            _ => None,
        }
    }

    pub fn get_debug_flat_instruction(
        self,
        instructions: Vec<InstructionWithSpan>,
    ) -> Option<Self> {
        match self {
            RegularInstruction::RemoteExecution(tree) => {
                Some(RegularInstruction::_RemoteExecutionDebugFlat(
                    InstructionBlockDataDebugFlat {
                        length: tree.length,
                        injected_variable_count: tree.injected_value_count,
                        injected_values: tree.injected_values,
                        body: instructions,
                    },
                ))
            }
            RegularInstruction::CallableDeclaration(tree) => {
                Some(RegularInstruction::_CallableDeclarationDebugFlat(
                    CallableDeclarationDataDebugFlat {
                        signature: tree.signature,
                        body: InstructionBlockDataDebugFlat {
                            length: tree.body.length,
                            injected_variable_count: tree
                                .body
                                .injected_value_count,
                            injected_values: tree.body.injected_values,
                            body: instructions,
                        },
                    },
                ))
            }
            RegularInstruction::Callable(tree) => Some(
                RegularInstruction::_CallableDebugFlat(CallableDataDebugFlat {
                    signature: tree.signature,
                    body: CallableDataBodyDebugFlat {
                        length: tree.body.length,
                        injected_value_count: tree.body.injected_value_count,
                        body: instructions,
                    },
                }),
            ),
            _ => None,
        }
    }

    pub fn get_normal_instruction_from_debug_instruction(
        &self,
    ) -> Option<Self> {
        match self {
            RegularInstruction::_RemoteExecutionDebugTree(tree) => Some(
                RegularInstruction::RemoteExecution(InstructionBlockData {
                    length: tree.length,
                    injected_value_count: tree.injected_variable_count,
                    injected_values: tree.injected_values.clone(),
                    body: self.inner_instructions().unwrap().to_vec(),
                }),
            ),
            RegularInstruction::_RemoteExecutionDebugFlat(tree) => Some(
                RegularInstruction::RemoteExecution(InstructionBlockData {
                    length: tree.length,
                    injected_value_count: tree.injected_variable_count,
                    injected_values: tree.injected_values.clone(),
                    body: self.inner_instructions().unwrap().to_vec(),
                }),
            ),
            RegularInstruction::_CallableDeclarationDebugTree(tree) => {
                Some(RegularInstruction::CallableDeclaration(
                    CallableDeclarationData {
                        signature: tree.signature.clone(),
                        body: InstructionBlockData {
                            length: tree.body.length,
                            injected_value_count: tree
                                .body
                                .injected_variable_count,
                            injected_values: tree.body.injected_values.clone(),
                            body: self.inner_instructions().unwrap().to_vec(),
                        },
                    },
                ))
            }
            RegularInstruction::_CallableDeclarationDebugFlat(tree) => {
                Some(RegularInstruction::CallableDeclaration(
                    CallableDeclarationData {
                        signature: tree.signature.clone(),
                        body: InstructionBlockData {
                            length: tree.body.length,
                            injected_value_count: tree
                                .body
                                .injected_variable_count,
                            injected_values: tree.body.injected_values.clone(),
                            body: self.inner_instructions().unwrap().to_vec(),
                        },
                    },
                ))
            }
            RegularInstruction::_CallableDebugTree(tree) => {
                Some(RegularInstruction::Callable(CallableData {
                    signature: tree.signature.clone(),
                    body: CallableDataBody {
                        length: tree.body.length,
                        injected_value_count: tree.body.injected_value_count,
                        body: self.inner_instructions().unwrap().to_vec(),
                    },
                }))
            }
            RegularInstruction::_CallableDebugFlat(tree) => {
                Some(RegularInstruction::Callable(CallableData {
                    signature: tree.signature.clone(),
                    body: CallableDataBody {
                        length: tree.body.length,
                        injected_value_count: tree.body.injected_value_count,
                        body: self.inner_instructions().unwrap().to_vec(),
                    },
                }))
            }
            _ => None,
        }
    }

    pub fn flatten_instruction(self) -> Option<Self> {
        match self {
            RegularInstruction::_RemoteExecutionDebugTree(tree) => {
                Some(RegularInstruction::_RemoteExecutionDebugFlat(
                    InstructionBlockDataDebugFlat {
                        length: tree.length,
                        injected_variable_count: tree.injected_variable_count,
                        injected_values: tree.injected_values,
                        body: tree.body.flatten_instructions(),
                    },
                ))
            }
            RegularInstruction::_CallableDeclarationDebugTree(tree) => {
                Some(RegularInstruction::_CallableDeclarationDebugFlat(
                    CallableDeclarationDataDebugFlat {
                        signature: tree.signature,
                        body: InstructionBlockDataDebugFlat {
                            length: tree.body.length,
                            injected_variable_count: tree
                                .body
                                .injected_variable_count,
                            injected_values: tree.body.injected_values,
                            body: tree.body.body.flatten_instructions(),
                        },
                    },
                ))
            }
            RegularInstruction::_CallableDebugTree(tree) => Some(
                RegularInstruction::_CallableDebugFlat(CallableDataDebugFlat {
                    signature: tree.signature,
                    body: CallableDataBodyDebugFlat {
                        length: tree.body.length,
                        injected_value_count: tree.body.injected_value_count,
                        body: tree.body.body.flatten_instructions(),
                    },
                }),
            ),
            _ => None,
        }
    }

    pub fn convert_to_nested(
        self,
        strategy: NestedInstructionResolutionStrategy,
    ) -> Result<Self, DXBParserError> {
        match strategy {
            NestedInstructionResolutionStrategy::ResolveNestedScopesFlat
            | NestedInstructionResolutionStrategy::ResolveNestedScopesTree => {
                let body = self.inner_instructions();
                if body.is_none() {
                    return Ok(self);
                }
                let body = body.unwrap();

                let (inner_instructions, err) =
                    crate::disassembler::disassemble_body(body, strategy);

                if let Some(err) = err {
                    return Err(err);
                }

                if strategy
                    == NestedInstructionResolutionStrategy::
                ResolveNestedScopesFlat
                {
                    Ok(self.clone().get_debug_flat_instruction(inner_instructions.flatten()).unwrap_or(self))
                } else {
                    Ok(self.clone().get_debug_tree_instruction(inner_instructions).unwrap_or(self))
                }
            }

            _ => Ok(self),
        }
    }
}
