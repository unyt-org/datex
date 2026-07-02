use crate::prelude::*;
use core::ops::Range;

use crate::{types::error::TypeError, utils::maybe_action::ErrorCollector};

#[derive(Debug)]
pub struct SpannedTypeError {
    pub error: TypeError,
    pub span: Option<Range<usize>>,
}

impl PartialEq for SpannedTypeError {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error
    }
}

impl SpannedTypeError {
    pub fn new_with_span(
        error: TypeError,
        span: Range<usize>,
    ) -> SpannedTypeError {
        SpannedTypeError {
            error,
            span: Some(span),
        }
    }
}

impl From<TypeError> for SpannedTypeError {
    fn from(value: TypeError) -> Self {
        SpannedTypeError {
            error: value,
            span: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct DetailedTypeErrors {
    pub errors: Vec<SpannedTypeError>,
}

impl ErrorCollector<SpannedTypeError> for DetailedTypeErrors {
    fn record_error(&mut self, error: SpannedTypeError) {
        self.errors.push(error);
    }
}

impl DetailedTypeErrors {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Debug, PartialEq)]
pub enum SimpleOrDetailedTypeError {
    Simple(SpannedTypeError),
    Detailed(DetailedTypeErrors),
}

impl From<SpannedTypeError> for SimpleOrDetailedTypeError {
    fn from(value: SpannedTypeError) -> Self {
        SimpleOrDetailedTypeError::Simple(value)
    }
}

impl From<DetailedTypeErrors> for SimpleOrDetailedTypeError {
    fn from(value: DetailedTypeErrors) -> Self {
        SimpleOrDetailedTypeError::Detailed(value)
    }
}
