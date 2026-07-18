pub use crate::global::operators::modification::ModificationOperator;

pub mod binary;
pub use binary::BinaryOperator;

pub mod comparison;
pub use comparison::ComparisonOperator;

pub mod modification;
pub mod unary;

pub use unary::{
    ArithmeticUnaryOperator, BitwiseUnaryOperator, LogicalUnaryOperator,
    SharedValueUnaryOperator, UnaryOperator,
};
