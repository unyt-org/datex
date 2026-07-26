use std::marker::PhantomData;
use sqlx::{Database, Error, IntoArguments};
use crate::{
    libs::core::type_id::{CoreLibBaseTypeId, CoreLibTypeId},
    prelude::*,
    types::{
        literal_type_definition::LiteralTypeDefinition,
        type_definition::TypeDefinition,
    },
    values::core_values::text::Text,
};
use crate::values::value_container::ValueContainer;

pub struct CommonSqlProvider<T: Database>
where
        for<'c> &'c mut T::Connection: sqlx::Executor<'c, Database = T>,
        for<'c> <T as sqlx::Database>::Arguments<'c>: IntoArguments<'c, T>,
{
    pub(crate) conn: T::Connection,
}


pub struct Entry<'a> {
    pub values: Vec<(String, &'a ValueContainer)>,
}

impl<T: Database> CommonSqlProvider<T>
where
        for<'c> &'c mut T::Connection: sqlx::Executor<'c, Database = T>,
        for<'c> <T as sqlx::Database>::Arguments<'c>: IntoArguments<'c, T>,
{

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


    /// Creates a SQL statement to create a table with the given name and type definition.
    pub async fn create_table_sql(&mut self, table_name: &str, type_def: &TypeDefinition) -> Result<(), Error> {
        let q = self.create_create_query(table_name, type_def);
        sqlx::query(&q).execute(&mut self.conn).await?;
        Ok(())
    }

    /// Creates a SQL statement to insert a new entry into the table with the given name and entry.
    pub fn insert_table_sql(&self, table_name: &str, entry: &Entry) -> String {

        let mut values = Vec::new();
        for (column_name, value) in entry.values.iter() {
            values.push(format!("{} = ?", column_name));
        }
        format!("INSERT INTO {} ({}) VALUES ({})", table_name, values.join(", "), values.join(", "))
    }


    /// Creates a SQL statement to create a table with the given name and type definition.
    pub fn create_create_query(&self, table_name: &str, type_def: &TypeDefinition) -> String {
        let mut column_defs = Vec::new();
        for (column_name, column_type) in self.get_column_metadata(type_def) {
            column_defs.push(format!("{} {}", column_name, column_type));
        }
        format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, column_defs.join(", "))
    }
}
