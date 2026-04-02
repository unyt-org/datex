use std::panic;

use sqlx::{Connection, Row, SqliteConnection, sqlite::SqliteConnectOptions};

use crate::{
    collections::HashMap,
    shared_values::pointer_address::PointerAddress,
    storage::{
        StorageEntryId, StorageInterface,
        interface_impls::sql::common_sql_provider::CommonSqlProvider,
    },
    values::value_container::ValueContainer,
};

/// SQLite-backed implementation of the [`StorageInterface`] trait.
pub struct SqliteStorageInterface {
    /// Active database connection.
    conn: SqliteConnection,
    /// Maps collection IDs to their corresponding table name.
    table_name_map: HashMap<u32, String>,
    provider: CommonSqlProvider,
}

impl SqliteStorageInterface {
    /// Creates a new interface by establishing a connection with the given options.
    pub async fn new(
        options: &SqliteConnectOptions,
    ) -> Result<Self, sqlx::Error> {
        let conn = SqliteConnection::connect_with(options).await?;

        Ok(Self {
            conn,
            table_name_map: HashMap::new(),
            provider: CommonSqlProvider {},
        })
    }

    /// Returns the table name for the given collection ID, if one exists.
    fn get_table_name(&self, collection_id: u32) -> Option<&str> {
        self.table_name_map.get(&collection_id).map(|s| s.as_str())
    }
}

impl StorageInterface for SqliteStorageInterface {
    async fn create(&mut self, value: &ValueContainer) -> StorageEntryId {
        let table_name = "table_X";
        let table_id = sqlx::query(&format!(
            "
             CREATE TABLE IF NOT EXISTS \"{table_name}\" (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 name TEXT
             )
         ",
        ))
        .execute(&mut self.conn)
        .await
        .unwrap()
        .last_insert_rowid();

        self.table_name_map
            .insert(table_id as u32, table_name.to_string());

        let name = "bene";
        let result = sqlx::query(&format!(
            "
             INSERT INTO \"{table_name}\" (name) VALUES (?)
         ",
        ))
        .bind(name)
        .execute(&mut self.conn)
        .await
        .unwrap();
        StorageEntryId::new(
            result.last_insert_rowid() as u32,
            Some(table_id as u32),
        )
    }

    async fn delete(&mut self, id: StorageEntryId) -> bool {
        let table_name =
            self.get_table_name(id.collection_id().unwrap()).unwrap();
        let result = sqlx::query(&format!(
            "
             DELETE FROM \"{table_name}\" WHERE id = ?
         ",
        ))
        .bind(id.id())
        .execute(&mut self.conn)
        .await
        .unwrap();
        result.rows_affected() == 1
    }

    async fn has(&mut self, id: StorageEntryId) -> bool {
        let table_name =
            self.get_table_name(id.collection_id().unwrap()).unwrap();
        let result = sqlx::query(&format!(
            "
             SELECT count(*) as count FROM \"{table_name}\" WHERE id = ?
         ",
        ))
        .bind(id.id())
        .fetch_one(&mut self.conn)
        .await
        .unwrap();
        result.try_get::<u64, _>("count").unwrap() == 1
    }

    async fn get(&mut self, id: StorageEntryId) -> Option<ValueContainer> {
        todo!()
    }

    async fn resolve_pointer_address(
        &mut self,
        address: PointerAddress,
    ) -> StorageEntryId {
        todo!()
    }

    async fn update(&mut self, id: StorageEntryId, value: &ValueContainer) {
        let table_name =
            self.get_table_name(id.collection_id().unwrap()).unwrap();
        let columns = self.provider.get_column_metadata(&value.allowed_type());
        panic!("columns: {:#?}", columns)
    }
}

#[cfg(test)]
mod tests {
    use crate::values::{
        core_value::CoreValue,
        core_values::{
            text::Text,
            r#type::{LocalMutability, Type, TypeMetadata},
        },
        value::Value,
    };

    use super::*;

    async fn get_interface() -> SqliteStorageInterface {
        SqliteStorageInterface::new(
            &SqliteConnectOptions::new().in_memory(true),
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_create() {
        let mut interface = get_interface().await;
        let value = ValueContainer::from("bene");
        let id = interface.create(&value).await;
        assert!(interface.has(id).await);
    }

    #[tokio::test]
    async fn test_delete() {
        let mut interface = get_interface().await;
        let value = ValueContainer::from("bene");
        let id = interface.create(&value).await;
        assert!(interface.delete(id).await);
        assert!(!interface.has(id).await);
    }

    #[tokio::test]
    async fn test_get() {
        let mut interface = get_interface().await;
        let value = ValueContainer::from("bene");
        let id = interface.create(&value).await;
        let retrieved = interface.get(id).await;
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_update() {
        let mut interface = get_interface().await;
        let value = ValueContainer::from("bene");
        let id = interface.create(&value).await;
        let updated = ValueContainer::Local(Value {
            inner: CoreValue::Null,
            actual_type: Box::new(TypeDefinition::Structural(
                StructuralTypeDefinition::Map(vec![
                    (
                        Type::structural(
                            StructuralTypeDefinition::Text(Text(
                                "name".to_string(),
                            )),
                            TypeMetadata::Local {
                                mutability: LocalMutability::Mutable,
                                reference_mutability: None,
                            },
                        ),
                        Type::text(),
                    ),
                    (
                        Type::structural(
                            StructuralTypeDefinition::Text(Text(
                                "age".to_string(),
                            )),
                            TypeMetadata::Local {
                                mutability: LocalMutability::Mutable,
                                reference_mutability: None,
                            },
                        ),
                        Type::integer(),
                    ),
                ]),
            )),
        });
        interface.update(id, &updated).await;
        let retrieved = interface.get(id).await;
        assert_eq!(retrieved, Some(updated));
    }

    #[tokio::test]
    async fn test_not_has() {
        let mut interface = get_interface().await;
        let value = ValueContainer::from("bene");
        let _ = interface.create(&value).await;
        let id = StorageEntryId::new(19, Some(1));
        assert!(!interface.has(id).await);
    }
}
