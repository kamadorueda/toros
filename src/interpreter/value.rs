// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::rc::Rc;

use nixel::ast::BinaryOperator;
use nixel::ast::AST;

use super::location::Location;
use super::location::LocationInFileFragment;
use crate::interpreter::bindings::Bindings;
use crate::interpreter::scope::Scope;
use crate::interpreter::scope::ScopeKind;

#[derive(Debug)]
pub(crate) enum Value {
    Boolean(bool),
    DeferredValue {
        ast:   AST,
        path:  Rc<String>,
        scope: Scope,
    },
    BuiltInFunction {
        expected_arguments: usize,
        identifier:         String,
    },
    Function {
        bind_to:        Option<String>,
        // destructure_to: Vec<FunctionArgument>,
        // ellipsis:       bool,
        implementation: AST,
        path:           Rc<String>,
        scope:          Scope,
    },
    FunctionApplication {
        argument_index: usize,
        arguments:      Vec<Rc<Value>>,
        function:       Rc<Value>,
        location:       Location,
    },
    Int(i64),
    Variable {
        identifier: String,
        location:   Location,
        scope:      Scope,
    },
}

impl Value {
    pub(crate) fn from_ast(path: Rc<String>, ast: AST, scope: &Scope) -> Value {
        match ast {
            AST::BinaryOperation { operands, operator, position } => {
                let identifier = match operator {
                    BinaryOperator::Addition => "built-in +",
                    BinaryOperator::Concatenation => "built-in ++",
                    BinaryOperator::Division => "built-in /",
                    BinaryOperator::EqualTo => "built-in ==",
                    BinaryOperator::GreaterThan => "built-in >",
                    BinaryOperator::GreaterThanOrEqualTo => "built-in >=",
                    BinaryOperator::Implication => "built-in ->",
                    BinaryOperator::LessThan => "built-in <",
                    BinaryOperator::LessThanOrEqualTo => "built-in <=",
                    BinaryOperator::LogicalAnd => "built-in &&",
                    BinaryOperator::LogicalOr => "built-in ||",
                    BinaryOperator::Multiplication => "built-in *",
                    BinaryOperator::NotEqualTo => "built-in !=",
                    BinaryOperator::Subtraction => "built-in -",
                    BinaryOperator::Update => "built-in //",
                };

                Value::FunctionApplication {
                    argument_index: 0,
                    arguments:      operands
                        .into_iter()
                        .map(|ast| {
                            Rc::new(Value::DeferredValue {
                                ast,
                                path: path.clone(),
                                scope: scope.clone(),
                            })
                        })
                        .collect(),

                    function: Rc::new(Value::BuiltInFunction {
                        identifier:         identifier.to_string(),
                        expected_arguments: 2,
                    }),
                    location: Location::InFileFragment(
                        LocationInFileFragment {
                            column: position.column,
                            line: position.line,
                            path,
                        },
                    ),
                }
            }

            AST::Function { argument, definition, .. } => {
                Value::Function {
                    bind_to: argument,
                    implementation: *definition,
                    // ellipsis: arguments.ellipsis,
                    path,
                    scope: scope.clone(),
                }
            }

            AST::FunctionApplication { arguments, function } => {
                let position = function.position();

                Value::FunctionApplication {
                    argument_index: 0,
                    arguments:      arguments
                        .into_iter()
                        .map(|ast| {
                            Rc::new(Value::DeferredValue {
                                ast,
                                path: path.clone(),
                                scope: scope.clone(),
                            })
                        })
                        .collect(),
                    function:       Rc::new(Value::from_ast(
                        path.clone(),
                        *function,
                        scope,
                    )),
                    location:       Location::InFileFragment(
                        LocationInFileFragment {
                            column: position.column,
                            line: position.line,
                            path,
                        },
                    ),
                }
            }

            AST::Int { value, .. } => Value::Int(value),

            AST::LetIn { bindings, target, position: _ } => {
                let bindings = Bindings::new(bindings);

                let scope_with_bindings = scope.derive(ScopeKind::Plain);

                for (binding_attribute, binding) in bindings.bindings {
                    scope_with_bindings.bind(
                        binding_attribute,
                        Rc::new(Value::DeferredValue {
                            ast:   binding.ast,
                            path:  path.clone(),
                            scope: if binding.inherited {
                                scope.clone()
                            } else {
                                scope_with_bindings.clone()
                            },
                        }),
                    )
                }

                Value::from_ast(path, *target, &scope_with_bindings)
            }

            AST::Variable { identifier, position } => Value::Variable {
                identifier,
                location: Location::InFileFragment(LocationInFileFragment {
                    column: position.column,
                    line: position.line,
                    path,
                }),
                scope: scope.clone(),
            },

            ast => todo!("Value::from_ast: {:#?}", ast),
        }
    }

    pub(crate) fn kind(&self) -> &str {
        match &self {
            Value::Boolean { .. } => "Boolean",
            Value::BuiltInFunction { .. } => "BuiltInFunction",
            Value::DeferredValue { .. } => "DeferredValue",
            Value::Function { .. } => "Function",
            Value::FunctionApplication { .. } => "FunctionApplication",
            Value::Int { .. } => "Int",
            Value::Variable { .. } => "Variable",
        }
    }
}
