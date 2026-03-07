use crate::{
    dif::update::DIFUpdate,
    prelude::*,
    shared_values::{
        observers::TransceiverId,
        pointer::Pointer,
        pointer_source::{
            AsyncPointerSource, PointerKey, ResolveCompleteness,
            ResolvedPointer, codec::PointerCodec, error::PointerSourceError,
            resolve_request::ResolveRequest,
        },
    },
    time::now_ms,
    types::definition::TypeDefinition,
    values::value_container::ValueContainer,
};
use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

pub struct SqlitePointerSource<C> {
    id: TransceiverId,
    pool: SqlitePool,
    codec: Arc<C>,
}

impl<C> SqlitePointerSource<C> {
    pub fn new(id: TransceiverId, pool: SqlitePool, codec: Arc<C>) -> Self {
        Self { id, pool, codec }
    }

    pub async fn migrate(&self) -> Result<(), PointerSourceError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pointers (
                pointer_key   TEXT PRIMARY KEY,
                value_blob    BLOB NOT NULL,
                allowed_type  BLOB NULL,
                updated_at_ms INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PointerSourceError::Backend(e.to_string()))?;

        Ok(())
    }
}
impl PointerKey for Pointer {
    fn storage_key(&self) -> String {
        self.address().to_string()
    }
}

#[async_trait(?Send)]
impl<C> AsyncPointerSource for SqlitePointerSource<C>
where
    C: PointerCodec,
    Pointer: PointerKey,
{
    fn id(&self) -> TransceiverId {
        self.id
    }

    fn name(&self) -> &'static str {
        "sqlite"
    }

    async fn has_pointer(
        &self,
        pointer: &Pointer,
    ) -> Result<bool, PointerSourceError> {
        let key = pointer.storage_key();

        let row =
            sqlx::query("SELECT 1 FROM pointers WHERE pointer_key = ? LIMIT 1")
                .bind(key)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| PointerSourceError::Backend(e.to_string()))?;

        Ok(row.is_some())
    }

    async fn resolve_pointer(
        &self,
        pointer: &Pointer,
        request: &ResolveRequest,
    ) -> Result<ResolvedPointer, PointerSourceError> {
        let key = pointer.storage_key();

        let row = sqlx::query(
            r#"
            SELECT value_blob, allowed_type
            FROM pointers
            WHERE pointer_key = ?
            LIMIT 1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PointerSourceError::Backend(e.to_string()))?
        .ok_or(PointerSourceError::NotFound)?;

        let value_blob: Vec<u8> = row
            .try_get("value_blob")
            .map_err(|e| PointerSourceError::Unsupported)?;

        let allowed_type_blob: Option<Vec<u8>> = row
            .try_get("allowed_type")
            .map_err(|e| PointerSourceError::Unsupported)?;

        let decoded = self.codec.decode_value(&value_blob)?;

        let completeness = if request.recursive {
            ResolveCompleteness::Partial
        } else {
            ResolveCompleteness::Full
        };

        let allowed_type = match allowed_type_blob {
            Some(bytes) => Some(self.codec.decode_type(&bytes)?),
            None => None,
        };

        Ok(ResolvedPointer {
            value_container: decoded,
            completeness,
            allowed_type,
            version: None,
        })
    }

    async fn put_pointer(
        &self,
        pointer: &Pointer,
        value: &ValueContainer,
        allowed_type: Option<&TypeDefinition>,
    ) -> Result<(), PointerSourceError> {
        let key = pointer.storage_key();
        let value_blob = self.codec.encode_value(value)?;
        let allowed_type_blob = match allowed_type {
            Some(ty) => Some(self.codec.encode_type(ty)?),
            None => None,
        };

        let now = now_ms() as i64;

        sqlx::query(
            r#"
            INSERT INTO pointers (pointer_key, value_blob, allowed_type, updated_at_ms)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(pointer_key) DO UPDATE SET
                value_blob = excluded.value_blob,
                allowed_type = excluded.allowed_type,
                updated_at_ms = excluded.updated_at_ms
            "#,
        )
        .bind(key)
        .bind(value_blob)
        .bind(allowed_type_blob)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| PointerSourceError::Backend(e.to_string()))?;

        Ok(())
    }

    async fn update_pointer(
        &self,
        pointer: &Pointer,
        update: &DIFUpdate,
    ) -> Result<(), PointerSourceError> {
        if update.source_id == self.id {
            return Ok(());
        }

        let key = pointer.storage_key();

        let row = sqlx::query(
            r#"
            SELECT value_blob
            FROM pointers
            WHERE pointer_key = ?
            LIMIT 1
            "#,
        )
        .bind(&key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PointerSourceError::Backend(e.to_string()))?
        .ok_or(PointerSourceError::NotFound)?;

        let value_blob: Vec<u8> = row
            .try_get("value_blob")
            .map_err(|e| PointerSourceError::Unavailable)?;

        let mut value = self.codec.decode_value(&value_blob)?;
        // self.codec.apply_update(&mut value, &update.update)?;
        let new_blob = self.codec.encode_value(&value)?;

        let now = now_ms() as i64;

        sqlx::query(
            r#"
            UPDATE pointers
            SET value_blob = ?, updated_at_ms = ?
            WHERE pointer_key = ?
            "#,
        )
        .bind(new_blob)
        .bind(now)
        .bind(&key)
        .execute(&self.pool)
        .await
        .map_err(|e| PointerSourceError::Backend(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::shared_values::{
        pointer_address::OwnedPointerAddress, pointer_source::codec,
    };

    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_sqlite_pointer_source() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create SQLite pool");

        let pointer_source =
            SqlitePointerSource::new(0, pool, Arc::new(codec::BincodeCodec));

        pointer_source.migrate().await.expect("Migration failed");
        let pointer =
            Pointer::new_owned(OwnedPointerAddress::new([1, 2, 3, 4, 5]));
        assert!(
            !pointer_source
                .has_pointer(&pointer)
                .await
                .expect("has_pointer failed")
        );
        let value_container = ValueContainer::from(42);
        pointer_source
            .put_pointer(&pointer, &value_container, None)
            .await
            .expect("put_pointer failed");
        assert!(
            pointer_source
                .has_pointer(&pointer)
                .await
                .expect("has_pointer failed")
        );
        let resolved = pointer_source
            .resolve_pointer(&pointer, &ResolveRequest::full())
            .await
            .expect("resolve_pointer failed");
        assert_eq!(resolved.value_container, (42).into());
    }
}
