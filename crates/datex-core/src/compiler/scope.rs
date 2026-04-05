use crate::{
    ast::expressions::VariableKind,
    collections::HashMap,
    compiler::{
        Variable, VariableRepresentation,
        context::InjectedParentVariable,
        precompiler::{
            precompiled_ast::RichAst, scope_stack::PrecompilerScopeStack,
        },
    },
    runtime::execution::context::ExecutionMode,
};
use core::cell::RefCell;
use crate::global::protocol_structures::injected_variable_type::InjectedVariableType;
use crate::global::protocol_structures::instruction_data::StackIndex;

#[derive(Debug, Default, Clone)]
pub struct PrecompilerData {
    // precompiler ast metadata
    pub rich_ast: RichAst,
    // precompiler scope stack
    pub precompiler_scope_stack: RefCell<PrecompilerScopeStack>,
}

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct CompilationScope {
    /// List of variables, mapped by name to their slot address and type.
    variables: HashMap<String, Variable>,
    /// parent scope, accessible from a child scope
    parent_scope: Option<Box<CompilationScope>>,
    /// scope of a parent context, e.g. when inside a block scope for remote execution calls or function bodies
    external_parent_scope: Option<Box<CompilationScope>>,
    /// next available index that can be allocated in the stack
    next_stack_index: StackIndex,

    // ------- Data only relevant for the root scope (FIXME: refactor?) -------
    /// optional precompiler data, only on the root scope
    pub precompiler_data: Option<PrecompilerData>,
    /// The execution mode of the scope.
    /// When the mode is set to Unbounded, the outer statements block will be an unbounded statement block.
    pub execution_mode: ExecutionMode,
    /// If was_used is true, the scope has been used for compilation and should not be reused if once is true.
    pub was_used: bool,
}

impl Default for CompilationScope {
    fn default() -> Self {
        CompilationScope {
            variables: HashMap::new(),
            parent_scope: None,
            external_parent_scope: None,
            next_stack_index: StackIndex(0),
            precompiler_data: Some(PrecompilerData::default()),
            execution_mode: ExecutionMode::Static,
            was_used: false,
        }
    }
}

impl CompilationScope {
    pub fn new(execution_mode: ExecutionMode) -> CompilationScope {
        CompilationScope {
            execution_mode,
            ..CompilationScope::default()
        }
    }

    pub fn new_with_external_parent_scope(
        parent_context: CompilationScope,
        initial_stack_index: StackIndex,
    ) -> CompilationScope {
        CompilationScope {
            external_parent_scope: Some(Box::new(parent_context)),
            next_stack_index: initial_stack_index,
            ..CompilationScope::default()
        }
    }

    pub fn mark_as_last_execution(&mut self) {
        match self.execution_mode {
            ExecutionMode::Static => {
                panic!(
                    "mark_as_last_execution can only be called for Unbounded execution modes"
                );
            }
            ExecutionMode::Unbounded { .. } => {
                self.execution_mode =
                    ExecutionMode::Unbounded { has_next: false };
            }
            _ => {}
        }
    }

    pub fn has_external_parent_scope(&self) -> bool {
        self.external_parent_scope.is_some()
    }

    pub fn register_variable_slot(&mut self, variable: Variable) {
        self.variables.insert(variable.name.clone(), variable);
    }

    pub fn get_next_stack_index(&mut self) -> StackIndex {
        let index = self.next_stack_index;
        self.next_stack_index += 1;
        index
    }

    /// Returns the stack index for a variable in this scope or potentially in the parent scope.
    /// The returned tuple contains the slot address, variable type, and a boolean indicating if it
    /// is a local variable (false) or from a parent scope (true).
    /// Returns an error if the variables comes from an external parent scope, but no slot type is provided to downgrade the virtual slot.
    pub fn resolve_variable_name_to_stack_index(
        &self,
        name: &str,
        slot_type: Option<InjectedVariableType>,
    ) -> Result<Option<(StackIndex, VariableKind)>, ()> {
        if let Some(variable) = self.variables.get(name) {
            Ok(Some((variable.index, variable.kind)))
        } else if let Some(external_parent) = &self.external_parent_scope {
            if let Some(slot_type) = slot_type {
                Ok(
                    // TODO: record external usage:
                    external_parent
                        .resolve_variable_name_to_stack_index(name, Some(slot_type))?
                        .map(|(virt_slot, var_kind)| (virt_slot, var_kind))
                )
            }
            else {
                Err(())
            }
        } else if let Some(parent) = &self.parent_scope {
            parent.resolve_variable_name_to_stack_index(name, slot_type)
        } else {
            Ok(None)
        }
    }

    /// Returns the virtual slot address for a variable in this scope or potentially in the parent scope.
    /// The returned tuple contains the slot address, variable type, and a boolean indicating if it
    /// is a local variable (false) or from a parent scope (true).
    pub fn resolve_variable_name_to_stack_index_with_slot_type(
        &self,
        name: &str,
        slot_type: InjectedVariableType,
    ) -> Option<(StackIndex, VariableKind)> {
        self.resolve_variable_name_to_stack_index(name, Some(slot_type)).unwrap()
    }

    /// Creates a new `CompileScope` that is a child of the current scope.
    pub fn push(self) -> CompilationScope {
        CompilationScope {
            next_stack_index: self.next_stack_index,
            parent_scope: Some(Box::new(self)),
            external_parent_scope: None,
            variables: HashMap::new(),
            precompiler_data: None,
            execution_mode: ExecutionMode::Static,
            was_used: false,
        }
    }

    /// Drops the current scope and returns to the parent scope
    pub fn pop(mut self) -> Option<CompilationScope> {
        Some(*(self.parent_scope.take()?))
    }

    pub fn pop_external(self) -> Option<CompilationScope> {
        self.external_parent_scope
            .map(|external_parent| *external_parent)
    }
}
