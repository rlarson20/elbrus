use crate::backend::DbError;
use crate::sqlite::SqliteBackend;
use chrono::{DateTime, Utc};
use elbrus_core::card::PriceSnapshot;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait PriceRepository: Send + Sync {
    async fn insert_snapshot(
        &self,
        printing_id: Uuid,
        snapshot: &PriceSnapshot,
    ) -> Result<(), DbError>;
    async fn get_latest_price(&self, printing_id: Uuid) -> Result<Option<PriceSnapshot>, DbError>;
    async fn get_price_history(&self, printing_id: Uuid) -> Result<Vec<PriceSnapshot>, DbError>;
    async fn get_collection_value(
        &self,
        collection_id: Uuid,
    ) -> Result<rust_decimal::Decimal, DbError>;
}

fn parse_price_snapshot(row: &sqlx::sqlite::SqliteRow) -> Result<PriceSnapshot, DbError> {
    let fetched_at_str: String = row
        .try_get("fetched_at")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let usd_str: Option<String> = row
        .try_get("usd")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let usd_foil_str: Option<String> = row
        .try_get("usd_foil")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let eur_str: Option<String> = row
        .try_get("eur")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let tix_str: Option<String> = row
        .try_get("tix")
        .map_err(|e| DbError::Sql(e.to_string()))?;

    let fetched_at = DateTime::parse_from_rfc3339(&fetched_at_str)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| DbError::Unknown(e.to_string()))?;

    let parse_dec = |opt: Option<String>| {
        opt.as_ref()
            .and_then(|s| rust_decimal::Decimal::from_str(s).ok())
    };

    Ok(PriceSnapshot {
        usd: parse_dec(usd_str),
        usd_foil: parse_dec(usd_foil_str),
        eur: parse_dec(eur_str),
        tix: parse_dec(tix_str),
        fetched_at,
    })
}

#[async_trait::async_trait]
impl PriceRepository for SqliteBackend {
    async fn insert_snapshot(
        &self,
        printing_id: Uuid,
        snapshot: &PriceSnapshot,
    ) -> Result<(), DbError> {
        let fetched_at_str = snapshot.fetched_at.to_rfc3339();

        sqlx::query(
            "INSERT INTO price_snapshots (printing_id, fetched_at, usd, usd_foil, eur, tix) \
             VALUES (?, ?, ?, ?, ?, ?) \
             ON CONFLICT (printing_id, fetched_at) DO UPDATE SET \
             usd = excluded.usd, usd_foil = excluded.usd_foil, eur = excluded.eur, tix = excluded.tix"
        )
        .bind(printing_id.to_string())
        .bind(fetched_at_str)
        .bind(snapshot.usd.map(|d| d.to_string()))
        .bind(snapshot.usd_foil.map(|d| d.to_string()))
        .bind(snapshot.eur.map(|d| d.to_string()))
        .bind(snapshot.tix.map(|d| d.to_string()))
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        Ok(())
    }

    async fn get_latest_price(&self, printing_id: Uuid) -> Result<Option<PriceSnapshot>, DbError> {
        let row_opt = sqlx::query(
            "SELECT fetched_at, usd, usd_foil, eur, tix FROM price_snapshots \
             WHERE printing_id = ? ORDER BY fetched_at DESC LIMIT 1",
        )
        .bind(printing_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        match row_opt {
            Some(row) => Ok(Some(parse_price_snapshot(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_price_history(&self, printing_id: Uuid) -> Result<Vec<PriceSnapshot>, DbError> {
        let rows = sqlx::query(
            "SELECT fetched_at, usd, usd_foil, eur, tix FROM price_snapshots \
             WHERE printing_id = ? ORDER BY fetched_at DESC",
        )
        .bind(printing_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        let mut snapshots = Vec::new();
        for row in rows {
            snapshots.push(parse_price_snapshot(&row)?);
        }
        Ok(snapshots)
    }

    async fn get_collection_value(
        &self,
        collection_id: Uuid,
    ) -> Result<rust_decimal::Decimal, DbError> {
        let rows = sqlx::query(
            "SELECT ce.quantity, ce.foil, p.usd, p.usd_foil \
             FROM collection_entries ce \
             LEFT JOIN ( \
                 SELECT p1.printing_id, p1.usd, p1.usd_foil \
                 FROM price_snapshots p1 \
                 INNER JOIN ( \
                     SELECT printing_id, MAX(fetched_at) as max_fetched \
                     FROM price_snapshots \
                     GROUP BY printing_id \
                 ) p2 ON p1.printing_id = p2.printing_id AND p1.fetched_at = p2.max_fetched \
             ) p ON ce.printing_id = p.printing_id \
             WHERE ce.collection_id = ?",
        )
        .bind(collection_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        let mut total = rust_decimal::Decimal::ZERO;
        for row in rows {
            let quantity = row.try_get::<i64, _>("quantity").unwrap() as u32;
            let foil: bool = row.try_get("foil").unwrap();

            // unwrap_or on trying to get the columns because they are nullable from the outer join
            let usd_str: Option<String> = row.try_get("usd").unwrap_or(None);
            let usd_foil_str: Option<String> = row.try_get("usd_foil").unwrap_or(None);

            let price_str_target = if foil {
                usd_foil_str.or(usd_str.clone())
            } else {
                usd_str.clone()
            };

            if let Some(val) =
                price_str_target.and_then(|p| rust_decimal::Decimal::from_str(&p).ok())
            {
                total += val * rust_decimal::Decimal::from(quantity);
            }
        }
        Ok(total)
    }
}
