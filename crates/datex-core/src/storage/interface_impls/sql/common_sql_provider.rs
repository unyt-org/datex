use crate::{
    libs::core::CoreLibPointerId,
    types::{
        definition::TypeDefinition,
        structural_type_definition::StructuralTypeDefinition,
    },
    values::core_values::text::Text,
};

pub struct CommonSqlProvider {}

impl CommonSqlProvider {
    pub fn get_column_metadata(
        &self,
        type_def: &TypeDefinition,
    ) -> Vec<(String, String)> {
        let mut columns = Vec::new();
        match type_def {
            TypeDefinition::Structural(StructuralTypeDefinition::Map(map)) => {
                for (key_type, value_type) in map {
                    let column_name = match &key_type.type_definition {
                        TypeDefinition::Structural(
                            StructuralTypeDefinition::Text(Text(key_name)),
                        ) => key_name,
                        _ => {
                            todo!()
                        }
                    };

                    let column_type = match &value_type.type_definition {
                        TypeDefinition::SharedReference(shared_ref) => {
                            let shared_type = shared_ref.borrow();
                            let pt = shared_type.pointer.address();
                            if pt == CoreLibPointerId::Text.into() {
                                "TEXT"
                            } else if pt
                                == CoreLibPointerId::Integer(None).into()
                            {
                                "INTEGER"
                            } else if pt
                                == CoreLibPointerId::Decimal(None).into()
                            {
                                "REAL"
                            } else if pt == CoreLibPointerId::Boolean.into() {
                                "SMALLINT"
                            } else if pt == CoreLibPointerId::Endpoint.into() {
                                "TEXT"
                            } else {
                                "TEXT"
                            }
                        }
                        _ => {
                            todo!()
                        }
                    };
                    columns.push((
                        column_name.to_string(),
                        column_type.to_string(),
                    ));
                }
            }
            _ => {
                todo!()
            }
        }
        columns
    }
}
