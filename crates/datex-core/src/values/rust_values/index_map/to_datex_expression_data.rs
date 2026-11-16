use crate::{
    ast::{expressions::DatexExpressionData, spanned::Spanned},
    random::RandomState,
    traits::to_datex_expression_data::ToDatexExpressionData,
};
use core::hash::Hash;
use indexmap::IndexMap;

impl<K, V> ToDatexExpressionData for IndexMap<K, V, RandomState>
where
    K: ToDatexExpressionData + Eq + Hash,
    V: ToDatexExpressionData,
{
    fn to_datex_expression_data(&self) -> DatexExpressionData {
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
