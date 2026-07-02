//! This module contains the parser for DATEX, which converts a string representation of DATEX source code into an abstract syntax tree (AST) that can be processed by the compiler.
pub(crate) mod body;
pub(crate) mod instruction_collector;
pub(crate) mod next_instructions_stack;
