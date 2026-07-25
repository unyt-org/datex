use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    types::{
        literal_type_definition::LiteralTypeDefinition,
        type_definition::TypeDefinition,
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
            TypeDefinition::Map(map) => {
                for (key_type, value_type) in &map.0 {
                    let column_name =
                        key_type.with_collapsed_type_definition(|c| match c {
                            TypeDefinition::Literal(
                                LiteralTypeDefinition::Text(Text(key_name)),
                            ) => key_name.clone(),
                            _ => {
                                todo!()
                            }
                        });

                    let column_type = value_type
                        .with_collapsed_type_definition(|c| match c {
                            TypeDefinition::CoreType(core_lib_id) => {
                                match core_lib_id {
                                    CoreLibTypeId::Base(
                                        CoreLibBaseTypeId::Text,
                                    ) => "TEXT",
                                    CoreLibTypeId::Base(
                                        CoreLibBaseTypeId::Integer,
                                    ) => "INTEGER",
                                    CoreLibTypeId::Base(
                                        CoreLibBaseTypeId::Decimal,
                                    ) => "REAL",
                                    CoreLibTypeId::Base(
                                        CoreLibBaseTypeId::Boolean,
                                    ) => "SMALLINT",
                                    CoreLibTypeId::Base(
                                        CoreLibBaseTypeId::Endpoint,
                                    ) => "TEXT",
                                    _ => "TEXT",
                                }
                            }
                            _ => {
                                todo!()
                            }
                        });
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
