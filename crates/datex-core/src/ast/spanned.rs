//! This module defines the [Spanned] trait for associating a span with an AST node.
use core::ops::Range;

pub trait Spanned: Sized {
    type Output;

    /// Associates a span with the AST node and returns the resulting node.
    fn with_span<T: Into<Range<usize>>>(self, span: T) -> Self::Output;

    /// Associates a default span with the AST node and returns the resulting node.
    fn with_default_span(self) -> Self::Output;
}
