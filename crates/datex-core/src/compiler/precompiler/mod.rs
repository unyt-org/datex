use crate::{
    collections::HashSet, type_inference::infer_expression_type_detailed_errors,
};

use crate::prelude::*;
use alloc::format;
use core::{cell::RefCell, ops::Range, unreachable};
pub mod options;
pub mod precompiled_ast;
pub mod scope;
pub mod scope_stack;
use crate::{
    ast::{
        expressions::{
            BinaryOperation, CloneExpression, DatexExpression,
            DatexExpressionData, GetSharedRef, RemoteExecution,
            RequestSharedRef, Statements, TypeDeclaration, TypeDeclarationKind,
            Unbox, UnboxAssignment, ValueAccessType, VariableAccess,
            VariableAssignment, VariableDeclaration, VariableKind,
            VariantAccess,
        },
        resolved_variable::ResolvedVariable,
        spanned::Spanned,
        type_expressions::{TypeExpressionData, TypeVariantAccess},
    },
    compiler::error::{
        CompilerError, DetailedCompilerErrors,
        DetailedCompilerErrorsWithRichAst,
        SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst,
        SpannedCompilerError,
    },
    global::operators::{BinaryOperator, binary::ArithmeticOperator},
    libs::core::{core_lib_id::CoreLibId, type_id::CoreLibBaseTypeId},
    runtime::Runtime,
    shared_values::{
        PointerAddress, ReferenceMutability, SharedContainer,
        SharedContainerMutability,
    },
    types::{
        nominal_type_definition::NominalTypeDefinition,
        shared_container_containing_type::SharedContainerContainingType,
        r#type::Type, type_definition::TypeDefinition,
    },
    utils::maybe_action::{ErrorCollector, MaybeAction, collect_or_pass_error},
    values::core_value::CoreValue,
    visitor::{
        VisitAction,
        expression::{ExpressionVisitor, visitable::ExpressionVisitResult},
        type_expression::{
            TypeExpressionVisitor, visitable::TypeExpressionVisitResult,
        },
    },
};
use options::PrecompilerOptions;
use precompiled_ast::{AstMetadata, RichAst, VariableShape};
use scope::NewScopeType;
use scope_stack::PrecompilerScopeStack;

pub struct Precompiler<'a> {
    ast_metadata: Rc<RefCell<AstMetadata>>,
    scope_stack: &'a mut PrecompilerScopeStack,
    collected_errors: Option<DetailedCompilerErrors>,
    is_first_level_expression: bool,
    runtime: Runtime,
}

/// Precompile the AST by resolving variable references and collecting metadata.
/// Exits early on first error encountered, returning a SpannedCompilerError.
pub fn precompile_ast_simple_error(
    ast: DatexExpression,
    scope_stack: &mut PrecompilerScopeStack,
    ast_metadata: Rc<RefCell<AstMetadata>>,
    runtime: Runtime,
) -> Result<RichAst, SpannedCompilerError> {
    precompile_ast(
        ast,
        scope_stack,
        ast_metadata,
        PrecompilerOptions {
            detailed_errors: false,
        },
        runtime,
    )
    .map_err(|e| {
        match e {
            SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst::Simple(
                error,
            ) => error,
            _ => unreachable!(), // because detailed_errors: false
        }
    })
}

/// Precompile the AST by resolving variable references and collecting metadata.
/// Collects all errors encountered, returning a DetailedCompilerErrorsWithRichAst.
pub fn precompile_ast_detailed_errors(
    ast: DatexExpression,
    scope_stack: &mut PrecompilerScopeStack,
    ast_metadata: Rc<RefCell<AstMetadata>>,
    runtime: Runtime,
) -> Result<RichAst, DetailedCompilerErrorsWithRichAst> {
    precompile_ast(
        ast,
        scope_stack,
        ast_metadata,
        PrecompilerOptions {
            detailed_errors: true,
        },
        runtime,
    )
    .map_err(|e| {
        match e {
            SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst::Detailed(
                error,
            ) => error,
            _ => unreachable!(), // because detailed_errors: true
        }
    })
}

/// Precompile the AST by resolving variable references and collecting metadata.
pub fn precompile_ast(
    ast: DatexExpression,
    scope_stack: &mut PrecompilerScopeStack,
    ast_metadata: Rc<RefCell<AstMetadata>>,
    options: PrecompilerOptions,
    runtime: Runtime,
) -> Result<RichAst, SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst> {
    Precompiler::new(scope_stack, ast_metadata, runtime)
        .precompile(ast, options)
}

impl<'a> Precompiler<'a> {
    pub fn new(
        scope_stack: &'a mut PrecompilerScopeStack,
        ast_metadata: Rc<RefCell<AstMetadata>>,
        runtime: Runtime,
    ) -> Self {
        Self {
            ast_metadata,
            scope_stack,
            collected_errors: None,
            is_first_level_expression: true,
            runtime,
        }
    }

    /// Collects an error if detailed error collection is enabled,
    /// or returns the error as Err()
    fn collect_error(
        &mut self,
        error: SpannedCompilerError,
    ) -> Result<(), SpannedCompilerError> {
        match &mut self.collected_errors {
            Some(collected_errors) => {
                collected_errors.record_error(error);
                Ok(())
            }
            None => Err(error),
        }
    }

    /// Collects the Err variant of the Result if detailed error collection is enabled,
    /// or returns the Result mapped to a MaybeAction.
    fn collect_result<T>(
        &mut self,
        result: Result<T, SpannedCompilerError>,
    ) -> Result<MaybeAction<T>, SpannedCompilerError> {
        collect_or_pass_error(&mut self.collected_errors, result)
    }

    fn get_variable_and_update_metadata(
        &mut self,
        name: &str,
    ) -> Result<usize, CompilerError> {
        self.scope_stack.get_variable_and_update_metadata(
            name,
            &mut self.ast_metadata.borrow_mut(),
        )
    }

    /// Precompile the AST by resolving variable references and collecting metadata.
    fn precompile(
        mut self,
        mut ast: DatexExpression,
        options: PrecompilerOptions,
    ) -> Result<RichAst, SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst>
    {
        if options.detailed_errors {
            self.collected_errors = Some(DetailedCompilerErrors::default());
        }

        // Hoist top-level type declaration if any
        if let DatexExpressionData::TypeDeclaration(type_declaration) =
            &mut ast.data
        {
            self.hoist_variable(type_declaration);
        }

        // visit ast recursively
        // returns Error directly if early exit on first error is enabled

        self.visit_datex_expression(&mut ast)?;

        let mut rich_ast = RichAst {
            metadata: self.ast_metadata,
            ast,
        };

        // type inference - currently only if detailed errors are enabled
        // FIXME #675: always do type inference here, not only for detailed errors
        if options.detailed_errors {
            let type_res = infer_expression_type_detailed_errors(
                &mut rich_ast,
                &self.runtime.memory().borrow(),
            );

            // append type errors to collected_errors if any
            if let Some(collected_errors) = self.collected_errors.as_mut()
                && let Err(type_errors) = type_res
            {
                collected_errors.append(type_errors.into());
            }
        }

        // if collecting detailed errors and an error occurred, return
        if let Some(errors) = self.collected_errors
            && errors.has_errors()
        {
            Err(
                SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst::Detailed(
                    DetailedCompilerErrorsWithRichAst {
                        errors,
                        ast: rich_ast,
                    },
                ),
            )
        } else {
            Ok(rich_ast)
        }
    }

    /// Adds a new variable to the current scope and metadata
    /// Returns the new variable ID
    fn add_new_variable(&mut self, name: String, kind: VariableShape) -> usize {
        let new_id = self.ast_metadata.borrow().variables.len();
        let var_metadata =
            self.scope_stack
                .add_new_variable(name.clone(), new_id, kind);
        self.ast_metadata.borrow_mut().variables.push(var_metadata);
        new_id
    }

    /// Resolves a variable name to either a local variable ID if it was already declared (or hoisted),
    /// or to a core library pointer ID if it is a core variable.
    /// If the variable cannot be resolved, a CompilerError is returned.
    fn resolve_variable(
        &mut self,
        name: &str,
    ) -> Result<ResolvedVariable, CompilerError> {
        // If variable exist
        if let Ok(id) = self.get_variable_and_update_metadata(name) {
            Ok(ResolvedVariable::VariableId(id))
        }
        // try to resolve core variable
        else if let Some(core) = CoreLibId::try_from_str(name) {
            Ok(ResolvedVariable::PointerAddress(PointerAddress::External(
                core.into(),
            )))
        } else {
            Err(CompilerError::UndeclaredVariable(name.to_string()))
        }
    }

    fn scope_type_for_expression(
        &mut self,
        expr: &DatexExpression,
    ) -> NewScopeType {
        match &expr.data {
            DatexExpressionData::RemoteExecution(_) => NewScopeType::None,
            _ => NewScopeType::NewScope,
        }
    }

    /// Hoist a variable declaration by marking it as hoisted and
    /// registering it in the current scope and metadata.
    fn hoist_variable(&mut self, data: &mut TypeDeclaration) {
        // set hoisted to true
        data.hoisted = true;

        // register variable
        let type_id =
            self.add_new_variable(data.name.clone(), VariableShape::Type);

        let type_def = match data.kind {
            TypeDeclarationKind::Nominal => {
                let memory = self.runtime.memory().borrow();
                Type::nominal(
                    NominalTypeDefinition::new_base(
                        memory.get_core_type(CoreLibBaseTypeId::Unknown),
                        data.name.clone(),
                    ),
                    &mut self.runtime.pointer_address_provider().borrow_mut(),
                    &memory,
                )
            }
            TypeDeclarationKind::Alias => {
                let memory = self.runtime.memory().borrow();
                let unknown = memory.get_core_type(CoreLibBaseTypeId::Unknown);
                Type::Alias(TypeDefinition::Shared(unsafe {
                    SharedContainerContainingType::new_unchecked(SharedContainer::new_owned_with_inferred_allowed_type(
                        CoreValue::Type(unknown),
                        SharedContainerMutability::Mutable,
                        &mut self.runtime.pointer_address_provider().borrow_mut(),
                        &memory,
                    ))
                }).into())
            }
        };

        {
            self.ast_metadata
                .borrow_mut()
                .variable_metadata_mut(type_id)
                .expect("TypeDeclaration should have variable metadata")
                .var_type = Some(type_def.clone());
        }
    }
}

impl<'a> TypeExpressionVisitor<SpannedCompilerError> for Precompiler<'a> {
    fn visit_literal_type(
        &mut self,
        literal: &mut String,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<SpannedCompilerError> {
        let resolved_variable = self.resolve_variable(literal)?;
        Ok(VisitAction::Replace(match resolved_variable {
            ResolvedVariable::VariableId(id) => {
                TypeExpressionData::VariableAccess(VariableAccess {
                    id,
                    name: literal.to_string(),
                    access_type: ValueAccessType::MoveOrCopy,
                })
                .with_span(span.clone())
            }
            ResolvedVariable::PointerAddress(pointer_address) => {
                TypeExpressionData::GetReference(pointer_address)
                    .with_span(span.clone())
            }
        }))
    }
    fn visit_variant_access_type(
        &mut self,
        variant_access: &mut TypeVariantAccess,
        span: &Range<usize>,
    ) -> TypeExpressionVisitResult<SpannedCompilerError> {
        // ensure lhs exist
        let _ = self.resolve_variable(&variant_access.name)?;
        let literal =
            format!("{}/{}", variant_access.name, variant_access.variant);

        // resolve full variant access
        let resolved_variable = self.resolve_variable(&literal)?;
        Ok(VisitAction::Replace(match resolved_variable {
            ResolvedVariable::VariableId(id) => {
                TypeExpressionData::VariableAccess(VariableAccess {
                    id,
                    name: literal,
                    access_type: ValueAccessType::MoveOrCopy,
                })
                .with_span(span.clone())
            }
            ResolvedVariable::PointerAddress(pointer_address) => {
                TypeExpressionData::GetReference(pointer_address)
                    .with_span(span.clone())
            }
        }))
    }
}

impl Precompiler<'_> {
    /// Returns a new DATEX expression for getting an identifier either as variable access or as pointer.
    /// If the variable could not be resolved
    /// - Ok(None) is returned if error collection is enabled
    /// - Err() is returned if early abort for errors is enabled
    fn get_identifier_with_access_type(
        &mut self,
        identifier: &String,
        span: &Range<usize>,
        access_type: ValueAccessType,
    ) -> Result<Option<DatexExpression>, SpannedCompilerError> {
        let result = self.resolve_variable(identifier).map_err(|error| {
            SpannedCompilerError::new_with_span(error, span.clone())
        });
        let action = self.collect_result(result)?;
        Ok(if let MaybeAction::Do(resolved_variable) = action {
            Some(match resolved_variable {
                ResolvedVariable::VariableId(id) => {
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id,
                        name: identifier.clone(),
                        access_type,
                    })
                    .with_span(span.clone())
                }
                ResolvedVariable::PointerAddress(pointer_address) => {
                    DatexExpressionData::RequestSharedRef(RequestSharedRef {
                        address: pointer_address,
                        mutability: ReferenceMutability::Immutable,
                    })
                    .with_span(span.clone())
                }
            })
        } else {
            None
        })
    }

    fn visit_identifier_with_access_type(
        &mut self,
        identifier: &String,
        span: &Range<usize>,
        access_type: ValueAccessType,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        let expression = self.get_identifier_with_access_type(
            identifier,
            span,
            access_type,
        )?;

        Ok(match expression {
            Some(expression) => VisitAction::Replace(expression),
            None => VisitAction::SkipChildren,
        })
    }
}

impl<'a> ExpressionVisitor<SpannedCompilerError> for Precompiler<'a> {
    /// Handle expression errors by either recording them if collected_errors is Some,
    /// or aborting the traversal if collected_errors is None.
    fn handle_expression_error(
        &mut self,
        error: SpannedCompilerError,
        _expression: &DatexExpression,
    ) -> Result<VisitAction<DatexExpression>, SpannedCompilerError> {
        if let Some(collected_errors) = self.collected_errors.as_mut() {
            collected_errors.record_error(error);
            Ok(VisitAction::VisitChildren)
        } else {
            Err(error)
        }
    }

    fn before_visit_datex_expression(&mut self, expr: &mut DatexExpression) {
        match self.scope_type_for_expression(expr) {
            NewScopeType::NewScopeWithNewRealm => {
                self.scope_stack.push_scope();
                self.scope_stack.increment_realm_index();
            }
            NewScopeType::NewScope => {
                // if in top level scope, don't create a new scope if first ast level
                if !(self.scope_stack.scopes.len() == 1
                    && self.is_first_level_expression)
                {
                    self.scope_stack.push_scope();
                }
            }
            _ => {}
        };

        self.is_first_level_expression = false;
    }

    fn after_visit_datex_expression(&mut self, expr: &mut DatexExpression) {
        match self.scope_type_for_expression(expr) {
            NewScopeType::NewScope | NewScopeType::NewScopeWithNewRealm => {
                // always keep top level scope
                if self.scope_stack.scopes.len() > 1 {
                    self.scope_stack.pop_scope();
                }
            }
            _ => {}
        };
    }

    fn visit_remote_execution(
        &mut self,
        remote_execution: &mut RemoteExecution,
        _: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        self.visit_datex_expression(&mut remote_execution.left)?;

        self.scope_stack.push_scope();
        self.scope_stack.increment_realm_index();

        self.visit_datex_expression(&mut remote_execution.right)?;
        let scope = self.scope_stack.pop_scope();
        remote_execution.injected_variable_count =
            Some(scope.external_variables.len() as u32);
        Ok(VisitAction::SkipChildren)
    }

    fn visit_statements(
        &mut self,
        statements: &mut Statements,
        _: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        let mut registered_names = HashSet::new();
        let is_terminated = statements.is_terminated;
        let statements_length = statements.statements.len();
        for (i, statement_expressions) in
            statements.statements.iter_mut().enumerate()
        {
            match &mut statement_expressions.data {
                DatexExpressionData::TypeDeclaration(type_declaration) => {
                    let name = &type_declaration.name;
                    if registered_names.contains(name) {
                        self.collect_error(
                            CompilerError::InvalidRedeclaration(name.clone())
                                .into(),
                        )?
                    }
                    registered_names.insert(name.clone());
                    self.hoist_variable(type_declaration);
                }
                // also terminate execution block for remote execution if the result is not used
                DatexExpressionData::RemoteExecution(remote_execution) => {
                    // if not last statement, or last statement and terminated
                    if i != statements_length - 1 || is_terminated {
                        match &mut remote_execution.right.data {
                            DatexExpressionData::Statements(statements) => {
                                statements.is_terminated = true;
                            }
                            _ => {
                                *remote_execution.right =
                                    DatexExpressionData::Statements(
                                        Statements {
                                            is_terminated: true,
                                            unbounded: None,
                                            statements: vec![
                                                *remote_execution.right.clone(),
                                            ],
                                        },
                                    )
                                    .with_span(
                                        remote_execution.right.span.clone(),
                                    );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(VisitAction::VisitChildren)
    }

    fn visit_type_declaration(
        &mut self,
        type_declaration: &mut TypeDeclaration,
        _: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        let name = type_declaration.name.clone();
        if type_declaration.hoisted {
            let id = self
                .get_variable_and_update_metadata(
                    &type_declaration.name.clone(),
                )
                .ok();
            type_declaration.id = id;
        } else {
            type_declaration.id =
                Some(self.add_new_variable(name, VariableShape::Type));
        }
        Ok(VisitAction::VisitChildren)
    }

    fn visit_binary_operation(
        &mut self,
        binary_operation: &mut BinaryOperation,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        let operator = &binary_operation.operator;

        // handle special case: / operator
        if operator == &BinaryOperator::Arithmetic(ArithmeticOperator::Divide) {
            let left = &mut binary_operation.left;
            let right = &mut binary_operation.right;

            let lit_left =
                if let DatexExpressionData::Identifier(name) = &left.data {
                    name.clone()
                } else {
                    return Ok(VisitAction::VisitChildren);
                };
            let lit_right =
                if let DatexExpressionData::Identifier(name) = &right.data {
                    name.clone()
                } else {
                    return Ok(VisitAction::VisitChildren);
                };
            // both of the sides are identifiers
            let left_var = self.resolve_variable(lit_left.as_str());
            let is_right_defined =
                self.resolve_variable(lit_right.as_str()).is_ok();

            // left is defined (could be integer, or user defined variable)
            if let Ok(left_var) = left_var {
                if is_right_defined {
                    // both sides are defined, left side could be a type, or no,
                    // same for right side
                    // could be variant access if the left side is a type and right side does exist as subvariant,
                    // otherwise we try division
                    Ok(VisitAction::VisitChildren)
                } else {
                    // is right is not defined, fallback to variant access
                    // could be divison though, where user misspelled rhs (unhandled, will throw)
                    Ok(VisitAction::Replace(DatexExpression::new(
                        DatexExpressionData::VariantAccess(VariantAccess {
                            base: left_var,
                            name: lit_left,
                            variant: lit_right,
                        }),
                        span.clone(),
                    )))
                }
            } else {
                Ok(VisitAction::VisitChildren)
            }
        } else {
            Ok(VisitAction::VisitChildren)
        }
    }

    fn visit_variable_declaration(
        &mut self,
        variable_declaration: &mut VariableDeclaration,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        // check if variable already declared in active scope
        if let Some(existing_var_id) = self
            .scope_stack
            .get_active_scope()
            .variable_ids_by_name
            .get(&variable_declaration.name)
        {
            variable_declaration.id = Some(*existing_var_id);
            return Err(SpannedCompilerError::new_with_span(
                CompilerError::InvalidRedeclaration(
                    variable_declaration.name.clone(),
                ),
                span.clone(),
            ));
        }
        variable_declaration.id = Some(self.add_new_variable(
            variable_declaration.name.clone(),
            VariableShape::Value(variable_declaration.kind),
        ));
        Ok(VisitAction::VisitChildren)
    }

    fn visit_variable_assignment(
        &mut self,
        variable_assignment: &mut VariableAssignment,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        let res = self
            .get_variable_and_update_metadata(&variable_assignment.name)
            .map_err(|error| {
                SpannedCompilerError::new_with_span(error, span.clone())
            });
        let action = self.collect_result(res)?;
        if let MaybeAction::Do(new_id) = action {
            // continue
            // check if variable is const
            let var_shape = self
                .ast_metadata
                .borrow()
                .variable_metadata(new_id)
                .unwrap()
                .shape;
            variable_assignment.id = Some(new_id);
            if let VariableShape::Value(VariableKind::Const) = var_shape {
                self.collect_error(SpannedCompilerError::new_with_span(
                    CompilerError::AssignmentToConst(
                        variable_assignment.name.clone(),
                    ),
                    span.clone(),
                ))?;
            };
        }
        Ok(VisitAction::VisitChildren)
    }

    fn visit_get_shared_ref(
        &mut self,
        get_shared_ref: &mut GetSharedRef,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        // if expression is an identifier, set access type to shared (mut)
        if let DatexExpressionData::Identifier(name) =
            &get_shared_ref.expression.data
        {
            let access_type = ValueAccessType::from(&get_shared_ref.mutability);
            self.visit_identifier_with_access_type(name, span, access_type)
        }
        // if expression is placeholder, set access type to shared (mut)
        else if let DatexExpressionData::Placeholder(_access_type) =
            &get_shared_ref.expression.data
        {
            let access_type = ValueAccessType::from(&get_shared_ref.mutability);
            Ok(VisitAction::Replace(
                DatexExpressionData::Placeholder(access_type)
                    .with_span(span.clone()),
            ))
        } else {
            Ok(VisitAction::VisitChildren)
        }
    }

    fn visit_clone(
        &mut self,
        clone: &mut CloneExpression,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        // if expression is an identifier, set access type to clone
        if let DatexExpressionData::Identifier(name) = &clone.expression.data {
            self.visit_identifier_with_access_type(
                name,
                span,
                ValueAccessType::Clone,
            )
        }
        // if expression is placeholder, set access type to clone
        else if let DatexExpressionData::Placeholder(_access_type) =
            &clone.expression.data
        {
            Ok(VisitAction::Replace(
                DatexExpressionData::Placeholder(ValueAccessType::Clone)
                    .with_span(span.clone()),
            ))
        } else {
            Ok(VisitAction::VisitChildren)
        }
    }

    fn visit_unbox(
        &mut self,
        unbox: &mut Unbox,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        // if expression is an identifier, set access type to clone
        if let DatexExpressionData::Identifier(name) = &unbox.expression.data {
            let expression = self.get_identifier_with_access_type(
                name,
                &unbox.expression.span,
                ValueAccessType::Borrow,
            )?;
            match expression {
                Some(expression) => Ok(VisitAction::ReplaceRecurse(
                    DatexExpressionData::Unbox(Unbox {
                        expression: Box::new(expression),
                    })
                    .with_span(span.clone()),
                )),
                None => Ok(VisitAction::SkipChildren),
            }
        }
        // if expression is placeholder, set access type to clone
        else if let DatexExpressionData::Placeholder(_access_type) =
            &unbox.expression.data
        {
            Ok(VisitAction::Replace(
                DatexExpressionData::Placeholder(ValueAccessType::Borrow)
                    .with_span(span.clone()),
            ))
        } else {
            Ok(VisitAction::VisitChildren)
        }
    }

    fn visit_unbox_assignment(
        &mut self,
        unbox_assignment: &mut UnboxAssignment,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        // if expression is an identifier, set access type to clone
        if let DatexExpressionData::Identifier(name) =
            &unbox_assignment.unbox_expression.data
        {
            let expression = self.get_identifier_with_access_type(
                name,
                &unbox_assignment.unbox_expression.span,
                ValueAccessType::Borrow,
            )?;
            match expression {
                Some(expression) => Ok(VisitAction::ReplaceRecurse(
                    DatexExpressionData::UnboxAssignment(UnboxAssignment {
                        operator: unbox_assignment.operator,
                        unbox_expression: Box::new(expression),
                        assigned_expression: unbox_assignment
                            .assigned_expression
                            .clone(),
                    })
                    .with_span(span.clone()),
                )),
                None => Ok(VisitAction::SkipChildren),
            }
        } else {
            Ok(VisitAction::VisitChildren)
        }
    }

    fn visit_identifier(
        &mut self,
        identifier: &mut String,
        span: &Range<usize>,
    ) -> ExpressionVisitResult<SpannedCompilerError> {
        self.visit_identifier_with_access_type(
            identifier,
            span,
            ValueAccessType::MoveOrCopy,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ast::{
            expressions::{
                CreateShared, GetRef, GetSharedRef, RequestSharedRef, Unbox,
            },
            resolved_variable::ResolvedVariable,
            type_expressions::{StructuralMap, TypeExpressionData},
        },
        libs::core::type_id::{
            CoreLibBaseTypeId, CoreLibTypeId, CoreLibVariantTypeId,
        },
        parser::Parser,
        runtime::{RuntimeConfig, RuntimeRunner},
        shared_values::{PointerAddress, SharedContainerMutability},
        types::type_definition_with_metadata::LocalReferenceMutability,
        values::core_values::{
            endpoint::Endpoint,
            integer::{Integer, typed_integer::IntegerTypeVariant},
        },
    };
    use core::assert_matches;
    use std::str::FromStr;

    fn precompile(
        ast: DatexExpression,
        options: PrecompilerOptions,
    ) -> Result<RichAst, SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst>
    {
        let runtime = Runtime::stub();
        let mut scope_stack = PrecompilerScopeStack::default();
        let ast_metadata = Rc::new(RefCell::new(AstMetadata::default()));
        Precompiler::new(&mut scope_stack, ast_metadata, runtime)
            .precompile(ast, options)
    }

    #[test]
    fn precompiler_visit() {
        let options = PrecompilerOptions::default();
        let ast = Parser::parse_with_default_options(
            "var x: integer = 34; var y = 10; x + y",
        )
        .unwrap();
        precompile(ast, options).expect("Should precompile without errors");
    }

    #[test]
    fn property_access() {
        let options = PrecompilerOptions::default();
        let ast =
            Parser::parse_with_default_options("var x = {a: 1}; x.a").unwrap();
        precompile(ast, options).expect("Should precompile without errors");
    }

    #[test]
    fn property_access_assignment() {
        let options = PrecompilerOptions::default();
        let ast =
            Parser::parse_with_default_options("var x = {a: 1}; x.a = 2;")
                .unwrap();
        precompile(ast, options).expect("Should precompile without errors");
    }

    #[test]
    fn undeclared_variable_error() {
        let options = PrecompilerOptions {
            detailed_errors: true,
        };
        let ast = Parser::parse_with_default_options("x + 10").unwrap();
        let result = precompile(ast, options);
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_variable_error() {
        let options = PrecompilerOptions {
            detailed_errors: false,
        };
        let ast = Parser::parse_with_default_options("var x = 1; var x = 2;")
            .unwrap();
        let result = precompile(ast, options);
        assert_matches!(result.unwrap_err(), SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst::Simple(SpannedCompilerError{span, error: CompilerError::InvalidRedeclaration(name)})  if name == "x");
    }

    #[test]
    fn invalid_type_redeclaration() {
        let src = r#"
        type A = integer;
        type A = text; // redeclaration error
        "#;
        let ast = Parser::parse_with_default_options(src).unwrap();
        let result = precompile(ast, PrecompilerOptions::default());
        assert!(result.is_err());
        assert_matches!(
            result,
            Err(SimpleCompilerErrorOrDetailedCompilerErrorWithRichAst::Simple(SpannedCompilerError{span, error: CompilerError::InvalidRedeclaration(name)})) if name == "A"
        );
    }

    fn parse_unwrap(src: &str) -> DatexExpression {
        Parser::parse_with_default_options(src).unwrap()
    }

    fn parse_and_precompile_spanned_result(
        src: &str,
    ) -> Result<RichAst, SpannedCompilerError> {
        let runtime = Runtime::stub();
        let mut scope_stack = PrecompilerScopeStack::default();
        let ast_metadata = Rc::new(RefCell::new(AstMetadata::default()));
        let ast = Parser::parse_with_default_options(src)?;
        precompile_ast_simple_error(
            ast,
            &mut scope_stack,
            ast_metadata,
            runtime,
        )
    }

    fn parse_and_precompile(src: &str) -> Result<RichAst, CompilerError> {
        parse_and_precompile_spanned_result(src).map_err(|e| e.error)
    }

    #[test]
    fn undeclared_variable() {
        let result = parse_and_precompile_spanned_result("x + 42");
        assert!(result.is_err());
        assert_matches!(
            result,
            Err(SpannedCompilerError{ error: CompilerError::UndeclaredVariable(var_name), span })
            if var_name == "x" && span == Some(0..1 )
        );
    }

    #[test]
    fn nominal_type_declaration() {
        let result = parse_and_precompile("type User = {a: integer}; User");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::TypeDeclaration(TypeDeclaration {
                        id: Some(0),
                        name: "User".to_string(),
                        definition: TypeExpressionData::StructuralMap(
                            StructuralMap(vec![(
                                TypeExpressionData::Text("a".to_string())
                                    .with_default_span(),
                                TypeExpressionData::GetReference(
                                    CoreLibTypeId::Base(
                                        CoreLibBaseTypeId::Integer
                                    )
                                    .into()
                                )
                                .with_default_span(),
                            )])
                        )
                        .with_default_span(),
                        hoisted: true,
                        kind: TypeDeclarationKind::Nominal,
                    })
                    .with_default_span(),
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id: 0,
                        name: "User".to_string(),
                        access_type: ValueAccessType::default(),
                    })
                    .with_default_span()
                ]
            ))
            .with_default_span()
        );
        let metadata = rich_ast.metadata.borrow();
        let var_meta = metadata.variable_metadata(0).unwrap();
        assert_eq!(var_meta.shape, VariableShape::Type);
    }

    #[test]
    fn scoped_variable() {
        let result = parse_and_precompile("(var z = 42;z); z");
        assert!(result.is_err());
        assert_matches!(
            result,
            Err(CompilerError::UndeclaredVariable(var_name))
            if var_name == "z"
        );
    }

    #[test]
    fn core_types() {
        let result = parse_and_precompile("boolean");
        assert_matches!(
            result,
            Ok(
                RichAst {
                    ast: DatexExpression { data: DatexExpressionData::RequestSharedRef(RequestSharedRef{address, mutability}), ..},
                    ..
                }
            ) if address == CoreLibTypeId::Base(CoreLibBaseTypeId::Boolean).into() && mutability == ReferenceMutability::Immutable
        );
        let result = parse_and_precompile("integer");
        assert_matches!(
            result,
            Ok(
                RichAst {
                    ast: DatexExpression { data: DatexExpressionData::RequestSharedRef(RequestSharedRef{address, mutability}), ..},
                    ..
                }
            ) if address == CoreLibTypeId::Base(CoreLibBaseTypeId::Integer).into()  && mutability == ReferenceMutability::Immutable
        );

        let result = parse_and_precompile("integer/u8");
        assert_eq!(
            result.unwrap().ast,
            DatexExpressionData::VariantAccess(VariantAccess {
                base: ResolvedVariable::PointerAddress(
                    CoreLibBaseTypeId::Integer.into()
                ),
                name: "integer".to_string(),
                variant: "u8".to_string(),
            })
            .with_default_span()
        );
    }

    #[test]
    fn variant_access() {
        // core type should work
        let result =
            parse_and_precompile("integer/u8").expect("Precompilation failed");
        assert_eq!(
            result.ast,
            DatexExpressionData::VariantAccess(VariantAccess {
                base: ResolvedVariable::PointerAddress(
                    CoreLibTypeId::Base(CoreLibBaseTypeId::Integer).into()
                ),
                name: "integer".to_string(),
                variant: "u8".to_string(),
            })
            .with_default_span()
        );

        // invalid variant should work (will error later in type checking)
        let result = parse_and_precompile("integer/invalid").unwrap();
        assert_eq!(
            result.ast,
            DatexExpressionData::VariantAccess(VariantAccess {
                base: ResolvedVariable::PointerAddress(
                    CoreLibTypeId::Base(CoreLibBaseTypeId::Integer).into()
                ),
                name: "integer".to_string(),
                variant: "invalid".to_string(),
            })
            .with_default_span()
        );

        // unknown type should error
        let result = parse_and_precompile("invalid/u8");
        assert_matches!(result, Err(CompilerError::UndeclaredVariable(var_name)) if var_name == "invalid");

        // a variant access without declaring the super type should error
        let result = parse_and_precompile("type User/admin = {}; User/admin");
        assert!(result.is_err());
        assert_matches!(result, Err(CompilerError::UndeclaredVariable(var_name)) if var_name == "User");

        // declared subtype should work
        let result = parse_and_precompile(
            "type User = {}; type User/admin = {}; User/admin",
        );
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::TypeDeclaration(TypeDeclaration {
                        id: Some(0),
                        name: "User".to_string(),
                        definition: TypeExpressionData::StructuralMap(
                            StructuralMap(vec![])
                        )
                        .with_default_span(),
                        hoisted: true,
                        kind: TypeDeclarationKind::Nominal,
                    })
                    .with_default_span(),
                    DatexExpressionData::TypeDeclaration(TypeDeclaration {
                        id: Some(1),
                        name: "User/admin".to_string(),
                        definition: TypeExpressionData::StructuralMap(
                            StructuralMap(vec![])
                        )
                        .with_default_span(),
                        hoisted: true,
                        kind: TypeDeclarationKind::Nominal
                    })
                    .with_default_span(),
                    DatexExpressionData::VariantAccess(VariantAccess {
                        base: ResolvedVariable::VariableId(0),
                        name: "User".to_string(),
                        variant: "admin".to_string(),
                    })
                    .with_default_span()
                ]
            ))
            .with_default_span()
        );

        // value shall be interpreted as division
        let result = parse_and_precompile("var a = 42; var b = 69; a/b");
        assert!(result.is_ok());
        let statements = if let DatexExpressionData::Statements(stmts) =
            result.unwrap().ast.data
        {
            stmts
        } else {
            core::panic!("Expected statements");
        };
        assert_eq!(
            *statements.statements.get(2).unwrap(),
            DatexExpressionData::BinaryOperation(BinaryOperation {
                operator: BinaryOperator::Arithmetic(
                    ArithmeticOperator::Divide
                ),
                left: Box::new(
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id: 0,
                        name: "a".to_string(),
                        access_type: ValueAccessType::MoveOrCopy,
                    })
                    .with_default_span()
                ),
                right: Box::new(
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id: 1,
                        name: "b".to_string(),
                        access_type: ValueAccessType::MoveOrCopy,
                    })
                    .with_default_span()
                ),
                ty: None
            })
            .with_default_span()
        );

        // type with value should be interpreted as division
        let result = parse_and_precompile("var a = 10; type b = 42; a/b");
        assert!(result.is_ok());
        let statements = if let DatexExpressionData::Statements(stmts) =
            result.unwrap().ast.data
        {
            stmts
        } else {
            core::panic!("Expected statements");
        };
        assert_eq!(
            *statements.statements.get(2).unwrap(),
            DatexExpressionData::BinaryOperation(BinaryOperation {
                operator: BinaryOperator::Arithmetic(
                    ArithmeticOperator::Divide
                ),
                left: Box::new(
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id: 1,
                        name: "a".to_string(),
                        access_type: ValueAccessType::MoveOrCopy,
                    })
                    .with_default_span()
                ),
                right: Box::new(
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id: 0,
                        name: "b".to_string(),
                        access_type: ValueAccessType::MoveOrCopy,
                    })
                    .with_default_span()
                ),
                ty: None
            })
            .with_default_span()
        );
    }

    #[test]
    fn type_declaration_assigment() {
        let result = parse_and_precompile("type MyInt = 1; var x = MyInt;");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_terminated(vec![
                DatexExpressionData::TypeDeclaration(TypeDeclaration {
                    id: Some(0),
                    name: "MyInt".to_string(),
                    definition: TypeExpressionData::Integer(Integer::from(1))
                        .with_default_span(),
                    hoisted: true,
                    kind: TypeDeclarationKind::Nominal
                })
                .with_default_span(),
                DatexExpressionData::VariableDeclaration(VariableDeclaration {
                    id: Some(1),
                    kind: VariableKind::Var,
                    name: "x".to_string(),
                    // must refer to variable id 0
                    init_expression: Box::new(
                        DatexExpressionData::VariableAccess(VariableAccess {
                            id: 0,
                            name: "MyInt".to_string(),
                            access_type: ValueAccessType::MoveOrCopy,
                        })
                        .with_default_span()
                    ),
                    type_annotation: None,
                })
                .with_default_span(),
            ]))
            .with_default_span()
        )
    }

    #[test]
    fn type_declaration_hoisted_assigment() {
        let result = parse_and_precompile("var x = MyInt; type MyInt = 1;");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_terminated(vec![
                DatexExpressionData::VariableDeclaration(VariableDeclaration {
                    id: Some(1),
                    kind: VariableKind::Var,
                    name: "x".to_string(),
                    // must refer to variable id 0
                    init_expression: Box::new(
                        DatexExpressionData::VariableAccess(VariableAccess {
                            id: 0,
                            name: "MyInt".to_string(),
                            access_type: ValueAccessType::MoveOrCopy,
                        })
                        .with_default_span()
                    ),
                    type_annotation: None,
                })
                .with_default_span(),
                DatexExpressionData::TypeDeclaration(TypeDeclaration {
                    id: Some(0),
                    name: "MyInt".to_string(),
                    definition: TypeExpressionData::Integer(Integer::from(1))
                        .with_default_span(),
                    hoisted: true,
                    kind: TypeDeclarationKind::Nominal
                })
                .with_default_span(),
            ]))
            .with_default_span()
        )
    }

    #[test]
    fn type_declaration_hoisted_cross_assigment() {
        let result = parse_and_precompile("type x = MyInt; type MyInt = x;");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_terminated(vec![
                DatexExpressionData::TypeDeclaration(TypeDeclaration {
                    id: Some(0),
                    name: "x".to_string(),
                    definition: TypeExpressionData::VariableAccess(
                        VariableAccess {
                            id: 1,
                            name: "MyInt".to_string(),
                            access_type: ValueAccessType::MoveOrCopy,
                        }
                    )
                    .with_default_span(),
                    hoisted: true,
                    kind: TypeDeclarationKind::Nominal
                })
                .with_default_span(),
                DatexExpressionData::TypeDeclaration(TypeDeclaration {
                    id: Some(1),
                    name: "MyInt".to_string(),
                    definition: TypeExpressionData::VariableAccess(
                        VariableAccess {
                            id: 0,
                            name: "x".to_string(),
                            access_type: ValueAccessType::MoveOrCopy,
                        }
                    )
                    .with_default_span(),
                    hoisted: true,
                    kind: TypeDeclarationKind::Nominal
                })
                .with_default_span(),
            ]))
            .with_default_span()
        )
    }

    #[test]
    fn type_invalid_nested_type_declaration() {
        let result = parse_and_precompile(
            "type x = NestedVar; (1; type NestedVar = x;)",
        );
        assert_matches!(result, Err(CompilerError::UndeclaredVariable(var_name)) if var_name == "NestedVar");
    }

    #[test]
    fn type_valid_nested_type_declaration() {
        let result =
            parse_and_precompile("type x = 10; (1; type NestedVar = x;)");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::TypeDeclaration(TypeDeclaration {
                        id: Some(0),
                        name: "x".to_string(),
                        definition: TypeExpressionData::Integer(Integer::from(
                            10
                        ))
                        .with_default_span(),
                        hoisted: true,
                        kind: TypeDeclarationKind::Nominal
                    })
                    .with_default_span(),
                    DatexExpressionData::Statements(
                        Statements::new_terminated(vec![
                            DatexExpressionData::Integer(Integer::from(1))
                                .with_default_span(),
                            DatexExpressionData::TypeDeclaration(
                                TypeDeclaration {
                                    id: Some(1),
                                    name: "NestedVar".to_string(),
                                    definition:
                                        TypeExpressionData::VariableAccess(
                                            VariableAccess {
                                                id: 0,
                                                name: "x".to_string(),
                                                access_type:
                                                    ValueAccessType::MoveOrCopy,
                                            }
                                        )
                                        .with_default_span(),
                                    hoisted: true,
                                    kind: TypeDeclarationKind::Nominal
                                }
                            )
                            .with_default_span(),
                        ])
                    )
                    .with_default_span()
                ]
            ))
            .with_default_span()
        )
    }

    #[test]
    fn core_reference_type() {
        let result = parse_and_precompile("type x = integer");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::TypeDeclaration(TypeDeclaration {
                id: Some(0),
                name: "x".to_string(),
                definition: TypeExpressionData::GetReference(
                    PointerAddress::from(CoreLibTypeId::Base(
                        CoreLibBaseTypeId::Integer
                    ))
                )
                .with_default_span(),
                hoisted: true,
                kind: TypeDeclarationKind::Nominal
            })
            .with_default_span()
        );
    }

    #[test]
    fn unbox() {
        let result = parse_and_precompile("const x = &42; *x");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::VariableDeclaration(
                        VariableDeclaration {
                            id: Some(0),
                            kind: VariableKind::Const,
                            name: "x".to_string(),
                            init_expression: Box::new(
                                DatexExpressionData::GetRef(GetRef {
                                    mutability:
                                        LocalReferenceMutability::Immutable,
                                    expression: Box::new(
                                        DatexExpressionData::Integer(
                                            Integer::from(42)
                                        )
                                        .with_default_span()
                                    )
                                })
                                .with_default_span(),
                            ),
                            type_annotation: None,
                        }
                    )
                    .with_default_span(),
                    DatexExpressionData::Unbox(Unbox {
                        expression: Box::new(
                            DatexExpressionData::VariableAccess(
                                VariableAccess {
                                    id: 0,
                                    name: "x".to_string(),
                                    access_type: ValueAccessType::Borrow,
                                }
                            )
                            .with_default_span()
                        )
                    })
                    .with_default_span(),
                ]
            ))
            .with_default_span()
        );
    }

    #[test]
    fn unbox_shared() {
        let result = parse_and_precompile("const x = 'shared 42; *x");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::VariableDeclaration(
                        VariableDeclaration {
                            id: Some(0),
                            kind: VariableKind::Const,
                            name: "x".to_string(),
                            init_expression: Box::new(
                                DatexExpressionData::GetSharedRef(GetSharedRef {
                                    mutability: ReferenceMutability::Immutable,
                                    expression: Box::new(
                                        DatexExpressionData::CreateShared(CreateShared {
                                            expression: Box::new(DatexExpressionData::Integer(
                                                Integer::from(42)
                                            ).with_default_span()),
                                            mutability: SharedContainerMutability::Immutable,
                                        }).with_default_span(),
                                    )
                                })
                                    .with_default_span(),
                            ),
                            type_annotation: None,
                        }
                    )
                        .with_default_span(),
                    DatexExpressionData::Unbox(Unbox {
                        expression: Box::new(DatexExpressionData::VariableAccess(
                            VariableAccess {
                                id: 0,
                                name: "x".to_string(),
                                access_type: ValueAccessType::Borrow,
                            }
                        )
                            .with_default_span())
                    }).with_default_span(),
                ]
            ))
                .with_default_span()
        );
    }

    #[test]
    fn placeholder_access() {
        let result = parse_and_precompile("?");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Placeholder(ValueAccessType::MoveOrCopy)
                .with_default_span()
        );
    }

    #[test]
    fn placeholder_shared_ref_access() {
        let result = parse_and_precompile("'?");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Placeholder(ValueAccessType::SharedRef)
                .with_default_span()
        );

        let result = parse_and_precompile("'((?))");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Placeholder(ValueAccessType::SharedRef)
                .with_default_span()
        );
    }

    #[test]
    fn placeholder_mut_shared_ref_access() {
        let result = parse_and_precompile("'mut ?");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Placeholder(ValueAccessType::SharedRefMut)
                .with_default_span()
        );

        let result = parse_and_precompile("'mut ((?))");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Placeholder(ValueAccessType::SharedRefMut)
                .with_default_span()
        );
    }

    #[test]
    fn placeholder_clone_access() {
        let result = parse_and_precompile("clone ?");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Placeholder(ValueAccessType::Clone)
                .with_default_span()
        );

        let result = parse_and_precompile("clone ((?))");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Placeholder(ValueAccessType::Clone)
                .with_default_span()
        );
    }

    #[test]
    fn variable_shared_ref_access() {
        let result = parse_and_precompile("var x = 42; 'x");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::VariableDeclaration(
                        VariableDeclaration {
                            id: Some(0),
                            kind: VariableKind::Var,
                            name: "x".to_string(),
                            init_expression: Box::new(
                                DatexExpressionData::Integer(Integer::from(42))
                                    .with_default_span()
                            ),
                            type_annotation: None,
                        }
                    )
                    .with_default_span(),
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id: 0,
                        name: "x".to_string(),
                        access_type: ValueAccessType::SharedRef,
                    })
                    .with_default_span(),
                ]
            ))
            .with_default_span()
        );
    }

    #[test]
    fn variable_shared_ref_mut_access() {
        let result = parse_and_precompile("var x = 42; 'mut x");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::VariableDeclaration(
                        VariableDeclaration {
                            id: Some(0),
                            kind: VariableKind::Var,
                            name: "x".to_string(),
                            init_expression: Box::new(
                                DatexExpressionData::Integer(Integer::from(42))
                                    .with_default_span()
                            ),
                            type_annotation: None,
                        }
                    )
                    .with_default_span(),
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id: 0,
                        name: "x".to_string(),
                        access_type: ValueAccessType::SharedRefMut,
                    })
                    .with_default_span(),
                ]
            ))
            .with_default_span()
        );
    }

    #[test]
    fn variable_clone_access() {
        let result = parse_and_precompile("var x = 42; clone x");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::VariableDeclaration(
                        VariableDeclaration {
                            id: Some(0),
                            kind: VariableKind::Var,
                            name: "x".to_string(),
                            init_expression: Box::new(
                                DatexExpressionData::Integer(Integer::from(42))
                                    .with_default_span()
                            ),
                            type_annotation: None,
                        }
                    )
                    .with_default_span(),
                    DatexExpressionData::VariableAccess(VariableAccess {
                        id: 0,
                        name: "x".to_string(),
                        access_type: ValueAccessType::Clone,
                    })
                    .with_default_span(),
                ]
            ))
            .with_default_span()
        );
    }

    #[test]
    fn variable_unbox_access() {
        let result = parse_and_precompile("var x = 42; *x");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::VariableDeclaration(
                        VariableDeclaration {
                            id: Some(0),
                            kind: VariableKind::Var,
                            name: "x".to_string(),
                            init_expression: Box::new(
                                DatexExpressionData::Integer(Integer::from(42))
                                    .with_default_span()
                            ),
                            type_annotation: None,
                        }
                    )
                    .with_default_span(),
                    DatexExpressionData::Unbox(Unbox {
                        expression: Box::new(
                            DatexExpressionData::VariableAccess(
                                VariableAccess {
                                    id: 0,
                                    name: "x".to_string(),
                                    access_type: ValueAccessType::Borrow,
                                }
                            )
                            .with_default_span(),
                        )
                    })
                    .with_default_span(),
                ]
            ))
            .with_default_span()
        );
    }

    #[test]
    fn remote_execution_terminate_single_statement() {
        let result = parse_and_precompile("@example :: 1;");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast.data,
            DatexExpressionData::Statements(Statements::new_terminated(vec![
                DatexExpressionData::RemoteExecution(RemoteExecution {
                    left: Box::new(
                        DatexExpressionData::Endpoint(
                            Endpoint::from_str("@example").unwrap()
                        )
                        .with_default_span()
                    ),
                    right: Box::new(
                        DatexExpressionData::Statements(
                            Statements::new_terminated(vec![
                                DatexExpressionData::Integer(Integer::from(1))
                                    .with_default_span()
                            ])
                        )
                        .with_default_span()
                    ),
                    injected_variable_count: Some(0),
                })
                .with_default_span()
            ]))
        )
    }

    #[test]
    fn remote_execution_terminate_multiple_statements() {
        let result = parse_and_precompile("@example :: 1; 2");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast.data,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::RemoteExecution(RemoteExecution {
                        left: Box::new(
                            DatexExpressionData::Endpoint(
                                Endpoint::from_str("@example").unwrap()
                            )
                            .with_default_span()
                        ),
                        right: Box::new(
                            DatexExpressionData::Statements(
                                Statements::new_terminated(vec![
                                    DatexExpressionData::Integer(
                                        Integer::from(1)
                                    )
                                    .with_default_span()
                                ])
                            )
                            .with_default_span()
                        ),
                        injected_variable_count: Some(0),
                    })
                    .with_default_span(),
                    DatexExpressionData::Integer(Integer::from(2))
                        .with_default_span(),
                ]
            ))
        )
    }

    #[test]
    fn remote_execution_terminate_inner_unterminated() {
        let result = parse_and_precompile("@example :: (1;2);");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast.data,
            DatexExpressionData::Statements(Statements::new_terminated(vec![
                DatexExpressionData::RemoteExecution(RemoteExecution {
                    left: Box::new(
                        DatexExpressionData::Endpoint(
                            Endpoint::from_str("@example").unwrap()
                        )
                        .with_default_span()
                    ),
                    right: Box::new(
                        DatexExpressionData::Statements(
                            Statements::new_terminated(vec![
                                DatexExpressionData::Integer(Integer::from(1))
                                    .with_default_span(),
                                DatexExpressionData::Integer(Integer::from(2))
                                    .with_default_span(),
                            ],)
                        )
                        .with_default_span()
                    ),
                    injected_variable_count: Some(0),
                })
                .with_default_span()
            ]))
        )
    }

    #[test]
    fn remote_execution_injected_variables() {
        let result = parse_and_precompile("var x = 10; @example :: x");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast.data,
            DatexExpressionData::Statements(Statements::new_unterminated(
                vec![
                    DatexExpressionData::VariableDeclaration(
                        VariableDeclaration {
                            id: Some(0),
                            kind: VariableKind::Var,
                            name: "x".to_string(),
                            init_expression: Box::new(
                                DatexExpressionData::Integer(Integer::from(10))
                                    .with_default_span()
                            ),
                            type_annotation: None,
                        }
                    )
                    .with_default_span(),
                    DatexExpressionData::RemoteExecution(RemoteExecution {
                        left: Box::new(
                            DatexExpressionData::Endpoint(
                                Endpoint::from_str("@example").unwrap()
                            )
                            .with_default_span()
                        ),
                        right: Box::new(
                            DatexExpressionData::VariableAccess(
                                VariableAccess {
                                    id: 0,
                                    name: "x".to_string(),
                                    access_type: ValueAccessType::MoveOrCopy,
                                }
                            )
                            .with_default_span()
                        ),
                        injected_variable_count: Some(1),
                    })
                    .with_default_span(),
                ]
            ))
        )
    }

    #[test]
    fn remote_execution_injected_variables_statements() {
        let result = parse_and_precompile("var x = 10; @example :: (x;x+1);");
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast.data,
            DatexExpressionData::Statements(Statements::new_terminated(vec![
                DatexExpressionData::VariableDeclaration(VariableDeclaration {
                    id: Some(0),
                    kind: VariableKind::Var,
                    name: "x".to_string(),
                    init_expression: Box::new(
                        DatexExpressionData::Integer(Integer::from(10))
                            .with_default_span()
                    ),
                    type_annotation: None,
                })
                    .with_default_span(),
                DatexExpressionData::RemoteExecution(RemoteExecution {
                    left: Box::new(DatexExpressionData::Endpoint(Endpoint::from_str("@example").unwrap()).with_default_span()),
                    right: Box::new(DatexExpressionData::Statements(Statements::new_terminated(
                        vec![
                            DatexExpressionData::VariableAccess(VariableAccess {
                                id: 0,
                                name: "x".to_string(),
                                access_type: ValueAccessType::MoveOrCopy,
                            }).with_default_span(),
                            DatexExpressionData::BinaryOperation(BinaryOperation {
                                operator: BinaryOperator::Arithmetic(
                                    ArithmeticOperator::Add
                                ),
                                left: Box::new(DatexExpressionData::VariableAccess(VariableAccess {
                                    id: 0,
                                    name: "x".to_string(),
                                    access_type: ValueAccessType::MoveOrCopy,
                                }).with_default_span()),
                                right: Box::new(DatexExpressionData::Integer(Integer::from(1)).with_default_span()),
                                ty: None
                            }).with_default_span(),
                        ],
                    )).with_default_span()),
                    injected_variable_count: Some(1),
                }).with_default_span(),
            ]))
        )
    }

    #[test]
    fn nested_remote_execution_injected_variables() {
        let result = parse_and_precompile(
            "var x = 10; var y = 11; @example :: (x; @example2 :: y);",
        );
        assert!(result.is_ok());
        let rich_ast = result.unwrap();
        assert_eq!(
            rich_ast.ast.data,
            DatexExpressionData::Statements(Statements::new_terminated(vec![
                DatexExpressionData::VariableDeclaration(VariableDeclaration {
                    id: Some(0),
                    kind: VariableKind::Var,
                    name: "x".to_string(),
                    init_expression: Box::new(
                        DatexExpressionData::Integer(Integer::from(10))
                            .with_default_span()
                    ),
                    type_annotation: None,
                })
                    .with_default_span(),
                DatexExpressionData::VariableDeclaration(VariableDeclaration {
                    id: Some(1),
                    kind: VariableKind::Var,
                    name: "y".to_string(),
                    init_expression: Box::new(
                        DatexExpressionData::Integer(Integer::from(11))
                            .with_default_span()
                    ),
                    type_annotation: None,
                })
                    .with_default_span(),
                DatexExpressionData::RemoteExecution(RemoteExecution {
                    left: Box::new(DatexExpressionData::Endpoint(Endpoint::from_str("@example").unwrap()).with_default_span()),
                    right: Box::new(DatexExpressionData::Statements(Statements::new_terminated(vec![
                        DatexExpressionData::VariableAccess(VariableAccess {
                            id: 0,
                            name: "x".to_string(),
                            access_type: ValueAccessType::MoveOrCopy,
                        }).with_default_span(),
                        DatexExpressionData::RemoteExecution(RemoteExecution {
                            left: Box::new(DatexExpressionData::Endpoint(Endpoint::from_str("@example2").unwrap()).with_default_span()),
                            right: Box::new(DatexExpressionData::Statements(Statements::new_terminated(vec![
                                DatexExpressionData::VariableAccess(VariableAccess {
                                    id: 1,
                                    name: "y".to_string(),
                                    access_type: ValueAccessType::MoveOrCopy,
                                }).with_default_span(),
                            ])).with_default_span()),
                            injected_variable_count: Some(1),
                        }).with_default_span(),
                    ])).with_default_span()),
                    injected_variable_count: Some(2),
                }).with_default_span(),
            ]))
        )
    }
}
