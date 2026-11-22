use crate::{ast::expressions::DatexExpression, prelude::*};

#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    pub entries: Vec<(DatexExpression, DatexExpression)>,
}

impl FromIterator<(DatexExpression, DatexExpression)> for Map {
    fn from_iter<T: IntoIterator<Item = (DatexExpression, DatexExpression)>>(
        iter: T,
    ) -> Self {
        let entries = iter.into_iter().collect();
        Map { entries }
    }
}

impl Map {
    pub fn new(entries: Vec<(DatexExpression, DatexExpression)>) -> Self {
        Map { entries }
    }
}
