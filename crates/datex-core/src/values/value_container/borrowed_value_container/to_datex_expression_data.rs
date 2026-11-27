use crate::{
    ast::{
        expressions::{NormalExpressionData, DeriveSharedRef},
        spanned::Spanned,
    },
    shared_values::SharedContainer,
    traits::to_datex_expression_data::ToDatexExpressionData,
    values::value_container::borrowed_value_container::{
        BorrowedValueContainer, BorrowedValueContainerMut,
    },
};

impl<'a> ToDatexExpressionData for BorrowedValueContainer<'a> {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        match self {
            BorrowedValueContainer::Local(value) => {
                value.to_datex_expression_data()
            }
            BorrowedValueContainer::Shared(shared) => match shared {
                SharedContainer::Referenced(referenced_container) => {
                    NormalExpressionData::DeriveSharedRef(DeriveSharedRef {
                        mutability: referenced_container.reference_mutability(),
                        expression: referenced_container
                            .to_datex_expression_data()
                            .with_default_span(),
                    })
                }
                SharedContainer::Owned(owned_container) => {
                    owned_container.to_datex_expression_data()
                }
            },
        }
    }
}

impl<'a> ToDatexExpressionData for BorrowedValueContainerMut<'a> {
    fn to_datex_expression_data(&self) -> NormalExpressionData {
        match self {
            BorrowedValueContainerMut::Local(value) => {
                value.to_datex_expression_data()
            }
            BorrowedValueContainerMut::Shared(shared) => match shared {
                SharedContainer::Referenced(referenced_container) => {
                    NormalExpressionData::DeriveSharedRef(DeriveSharedRef {
                        mutability: referenced_container.reference_mutability(),
                        expression: referenced_container
                            .to_datex_expression_data()
                            .with_default_span(),
                    })
                }
                SharedContainer::Owned(owned_container) => {
                    owned_container.to_datex_expression_data()
                }
            },
        }
    }
}
