use crate::collections::HashMap;
use core::hash::Hash;
use crate::ast::expressions::DatexExpressionData;
use crate::ast::spanned::Spanned;
use crate::traits::to_datex_expression_data::ToDatexExpressionData;
use crate::prelude::*;

impl<K, V> ToDatexExpressionData for HashMap<K, V>
where
    K: ToDatexExpressionData + Eq + Hash,
    V: ToDatexExpressionData,
{
    fn to_datex_expression_data(
        &self,
    ) -> DatexExpressionData {
        DatexExpressionData::Map(
            self.iter()
                .map(|(k, v)| {
                    (
                        k.to_datex_expression_data().with_default_span(),
                        v.to_datex_expression_data().with_default_span(),
                    )
                })
                .collect(),
        )
    }
}