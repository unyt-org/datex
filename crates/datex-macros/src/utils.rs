use core::str::FromStr;

use alloc::format;
use datex_core::values::{
    core_values::integer::typed_integer::{IntegerTypeVariant, TypedInteger},
    value_container::ValueContainer,
};
use syn::{Error, Expr, ExprLit, ExprUnary, Lit, UnOp};

/// Converts a syn expression to a ValueContainer, supporting string literals,
/// integer literals (with optional type suffixes), float literals (with optional type suffixes),
/// and boolean literals. For negative numbers, it supports unary negation of integer and float literals.
pub fn expr_to_value_container(exp: &Expr) -> Result<ValueContainer, Error> {
    match exp {
        Expr::Unary(ExprUnary {
            op: UnOp::Neg(_),
            expr: inner,
            ..
        }) => match inner.as_ref() {
            Expr::Lit(ExprLit {
                lit: Lit::Int(i), ..
            }) => {
                let variant = if i.suffix().is_empty() {
                    IntegerTypeVariant::I32
                } else {
                    IntegerTypeVariant::from_str(i.suffix()).unwrap()
                };
                Ok(ValueContainer::from(
                    TypedInteger::from_string_with_variant(
                        &format!("-{}", i.base10_digits()),
                        variant,
                    )
                    .unwrap(),
                ))
            }
            Expr::Lit(ExprLit {
                lit: Lit::Float(f), ..
            }) => {
                let suffix = if f.suffix().is_empty() {
                    "f64"
                } else {
                    f.suffix()
                };
                match suffix {
                    "f32" => Ok(ValueContainer::from(
                        format!("-{}", f.base10_digits())
                            .parse::<f32>()
                            .unwrap(),
                    )),
                    "f64" => Ok(ValueContainer::from(
                        format!("-{}", f.base10_digits())
                            .parse::<f64>()
                            .unwrap(),
                    )),
                    _ => panic!("Unsupported float literal suffix: {}", suffix),
                }
            }
            _ => Err(Error::new_spanned(
                exp,
                "Only unary negation of integer and float literals is supported",
            )),
        },
        Expr::Lit(lit) => match &lit.lit {
            Lit::Str(s) => Ok(ValueContainer::from(s.value())),
            Lit::Int(i) => {
                let variant = if i.suffix().is_empty() {
                    IntegerTypeVariant::I32
                } else {
                    IntegerTypeVariant::from_str(i.suffix()).unwrap()
                };
                Ok(ValueContainer::from(
                    TypedInteger::from_string_with_variant(
                        i.base10_digits(),
                        variant,
                    )
                    .unwrap(),
                ))
            }
            Lit::Float(f) => {
                let suffix = if f.suffix().is_empty() {
                    "f64"
                } else {
                    f.suffix()
                };
                match suffix {
                    "f32" => Ok(ValueContainer::from(
                        f.base10_parse::<f32>().unwrap(),
                    )),
                    "f64" => Ok(ValueContainer::from(
                        f.base10_parse::<f64>().unwrap(),
                    )),
                    _ => panic!("Unsupported float literal suffix: {}", suffix),
                }
            }
            Lit::Bool(b) => Ok(ValueContainer::from(b.value)),
            _ => Err(Error::new_spanned(
                exp,
                "Unsupported literal type. Only string, integer, float, and boolean literals are supported.",
            )),
        },
        _ => Err(Error::new_spanned(
            exp,
            "Unsupported expression type. Only literals and unary negation of literals are supported.",
        )),
    }
}
