use crate::backend::DbError;
use crate::sqlite::SqliteBackend;
use elbrus_core::{Collection, CollectionEntry, Condition};
use sqlx::Row;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn create_collection(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<Collection, DbError>;
    async fn delete_collection(&self, id: Uuid) -> Result<(), DbError>;
    async fn list_collections(&self) -> Result<Vec<Collection>, DbError>;
    async fn get_collection(&self, id: Uuid) -> Result<Option<Collection>, DbError>;
    async fn upsert_card(&self, entry: &CollectionEntry) -> Result<(), DbError>;
    async fn remove_card(
        &self,
        collection_id: Uuid,
        printing_id: Uuid,
        condition: Condition,
        foil: bool,
    ) -> Result<(), DbError>;
}

#[async_trait::async_trait]
impl CollectionRepository for SqliteBackend {
    async fn create_collection(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<Collection, DbError> {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO collections (id, name, description) VALUES (?, ?, ?)")
            .bind(id.to_string())
            .bind(name)
            .bind(description)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        Ok(Collection {
            id,
            name: name.into(),
            description: description.map(Into::into),
            entries: Vec::new(),
        })
    }

    async fn delete_collection(&self, id: Uuid) -> Result<(), DbError> {
        sqlx::query("DELETE FROM collections WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;
        Ok(())
    }

    async fn list_collections(&self) -> Result<Vec<Collection>, DbError> {
        let rows = sqlx::query("SELECT id, name, description FROM collections")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        let mut collections = Vec::new();
        for row in rows {
            let id_str: String = row.try_get("id").unwrap();
            let name_str: String = row.try_get("name").unwrap();
            let desc_str: Option<String> = row.try_get("description").unwrap();

            collections.push(Collection {
                id: Uuid::parse_str(&id_str).unwrap(),
                name: name_str.into(),
                description: desc_str.map(Into::into),
                entries: Vec::new(),
            });
        }
        Ok(collections)
    }

    async fn get_collection(&self, id: Uuid) -> Result<Option<Collection>, DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        let row_opt = sqlx::query("SELECT id, name, description FROM collections WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        if let Some(row) = row_opt {
            let name_str: String = row.try_get("name").unwrap();
            let desc_str: Option<String> = row.try_get("description").unwrap_or(None);

            let entry_rows = sqlx::query(
                "SELECT collection_id, printing_id, quantity, condition, foil, notes \
                 FROM collection_entries WHERE collection_id = ?",
            )
            .bind(id.to_string())
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

            let mut entries = Vec::new();
            for erow in entry_rows {
                let pid_str: String = erow.try_get("printing_id").unwrap();
                let quantity_i64: i64 = erow.try_get("quantity").unwrap();
                let quantity = u32::try_from(quantity_i64).unwrap_or(0);
                let cond_str: String = erow.try_get("condition").unwrap();
                let foil: bool = erow.try_get("foil").unwrap();
                let notes: Option<String> = erow.try_get("notes").unwrap_or(None);

                let condition: Condition =
                    serde_json::from_str(&format!("\"{cond_str}\"")).unwrap();

                entries.push(CollectionEntry {
                    collection_id: id,
                    printing_id: Uuid::parse_str(&pid_str).unwrap(),
                    quantity,
                    condition,
                    foil,
                    notes: notes.map(Into::into),
                });
            }

            tx.commit().await.map_err(|e| DbError::Sql(e.to_string()))?;

            Ok(Some(Collection {
                id,
                name: name_str.into(),
                description: desc_str.map(Into::into),
                entries,
            }))
        } else {
            Ok(None)
        }
    }

    async fn upsert_card(&self, entry: &CollectionEntry) -> Result<(), DbError> {
        let cond_str = serde_json::to_string(&entry.condition)
            .unwrap()
            .trim_matches('"')
            .to_string();
        sqlx::query(
            "INSERT INTO collection_entries (collection_id, printing_id, quantity, condition, foil, notes) \
             VALUES (?, ?, ?, ?, ?, ?) \
             ON CONFLICT (collection_id, printing_id, condition, foil) DO UPDATE SET \
             quantity = excluded.quantity, notes = excluded.notes"
        )
        .bind(entry.collection_id.to_string())
        .bind(entry.printing_id.to_string())
        .bind(i64::from(entry.quantity))
        .bind(cond_str)
        .bind(entry.foil)
        .bind(entry.notes.as_ref().map(ToString::to_string))
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        Ok(())
    }

    async fn remove_card(
        &self,
        collection_id: Uuid,
        printing_id: Uuid,
        condition: Condition,
        foil: bool,
    ) -> Result<(), DbError> {
        let cond_str = serde_json::to_string(&condition)
            .unwrap()
            .trim_matches('"')
            .to_string();
        sqlx::query(
            "DELETE FROM collection_entries \
             WHERE collection_id = ? AND printing_id = ? AND condition = ? AND foil = ?",
        )
        .bind(collection_id.to_string())
        .bind(printing_id.to_string())
        .bind(cond_str)
        .bind(foil)
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        Ok(())
    }
}
