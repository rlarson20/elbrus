use crate::backend::DbError;
use crate::sqlite::SqliteBackend;
use elbrus_core::{CardFace, CardLayout, Format, OracleCard, Printing, color::ColorSet};
use sqlx::Row;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait CardRepository: Send + Sync {
    async fn upsert_oracle(&self, card: &OracleCard) -> Result<(), DbError>;
    async fn upsert_printing(&self, p: &Printing) -> Result<(), DbError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Printing>, DbError>;
    async fn get_oracle(&self, oracle_id: Uuid) -> Result<Option<OracleCard>, DbError>;
    async fn search_name(&self, q: &str, limit: u32) -> Result<Vec<OracleCard>, DbError>;
    async fn search_fts(&self, q: &str, limit: u32) -> Result<Vec<OracleCard>, DbError>;
    async fn cards_in_set(&self, set_code: &str) -> Result<Vec<Printing>, DbError>;
    async fn legal_in_format(&self, format: &Format) -> Result<Vec<OracleCard>, DbError>;
}

fn parse_oracle_card(row: &sqlx::sqlite::SqliteRow) -> Result<OracleCard, DbError> {
    let oracle_id_str: String = row
        .try_get("oracle_id")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let oracle_id = Uuid::parse_str(&oracle_id_str).map_err(|e| DbError::Unknown(e.to_string()))?;

    let layout_str: String = row
        .try_get("layout")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let layout = serde_json::from_str(&format!("\"{layout_str}\""))
        .unwrap_or_else(|_| CardLayout::Unknown(layout_str.into()));

    let color_identity_val: i32 = row
        .try_get("color_identity")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let color_identity_u8 = u8::try_from(color_identity_val).unwrap_or(0);
    let color_identity = ColorSet::from_bits_truncate(color_identity_u8);

    let keywords_str: String = row
        .try_get("keywords")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let keywords =
        serde_json::from_str(&keywords_str).map_err(|e| DbError::Unknown(e.to_string()))?;

    let legalities_str: String = row
        .try_get("legalities")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let legalities =
        serde_json::from_str(&legalities_str).map_err(|e| DbError::Unknown(e.to_string()))?;

    let edh_rank: Option<i64> = row
        .try_get("edh_rank")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let edh_rank = edh_rank.and_then(|n| u32::try_from(n).ok());

    let reserved: bool = row
        .try_get("reserved")
        .map_err(|e| DbError::Sql(e.to_string()))?;

    Ok(OracleCard {
        oracle_id,
        layout,
        faces: Default::default(),
        color_identity,
        keywords,
        legalities,
        edh_rank,
        reserved,
    })
}

fn parse_card_face(row: &sqlx::sqlite::SqliteRow) -> Result<CardFace, DbError> {
    let name: String = row
        .try_get("name")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let mana_cost: Option<String> = row
        .try_get("mana_cost")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let type_line_str: String = row
        .try_get("type_line")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let oracle_text_str: String = row
        .try_get("oracle_text")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let colors_val: i32 = row
        .try_get("colors")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let power: Option<String> = row
        .try_get("power")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let toughness: Option<String> = row
        .try_get("toughness")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let loyalty: Option<String> = row
        .try_get("loyalty")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let defense: Option<String> = row
        .try_get("defense")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let flavor_text: Option<String> = row
        .try_get("flavor_text")
        .map_err(|e| DbError::Sql(e.to_string()))?;

    let parsed_mana_cost = match mana_cost {
        Some(s) => Some(serde_json::from_str(&s).map_err(|e| DbError::Unknown(e.to_string()))?),
        None => None,
    };
    let type_line =
        serde_json::from_str(&type_line_str).map_err(|e| DbError::Unknown(e.to_string()))?;
    let oracle_text =
        serde_json::from_str(&oracle_text_str).map_err(|e| DbError::Unknown(e.to_string()))?;

    let colors_u8 = u8::try_from(colors_val).unwrap_or(0);

    Ok(CardFace {
        name: name.into(),
        mana_cost: parsed_mana_cost,
        type_line,
        oracle_text,
        colors: ColorSet::from_bits_truncate(colors_u8),
        power: power.map(|s| s.into()),
        toughness: toughness.map(|s| s.into()),
        loyalty: loyalty.map(|s| s.into()),
        defense: defense.map(|s| s.into()),
        flavor_text: flavor_text.map(|s| s.into()),
    })
}

fn parse_printing(row: &sqlx::sqlite::SqliteRow) -> Result<Printing, DbError> {
    let id_str: String = row.try_get("id").map_err(|e| DbError::Sql(e.to_string()))?;
    let oracle_id_str: String = row
        .try_get("oracle_id")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let set_code: String = row
        .try_get("set_code")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let collector_number: String = row
        .try_get("collector_number")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let rarity_str: String = row
        .try_get("rarity")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let lang: String = row
        .try_get("lang")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let released_at_str: String = row
        .try_get("released_at")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let image_uris_str: Option<String> = row
        .try_get("image_uris")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let promo: bool = row
        .try_get("promo")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let digital: bool = row
        .try_get("digital")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let full_art: bool = row
        .try_get("full_art")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let textless: bool = row
        .try_get("textless")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let reprint: bool = row
        .try_get("reprint")
        .map_err(|e| DbError::Sql(e.to_string()))?;
    let prices_str: Option<String> = row
        .try_get("prices")
        .map_err(|e| DbError::Sql(e.to_string()))?;

    let id = Uuid::parse_str(&id_str).map_err(|e| DbError::Unknown(e.to_string()))?;
    let oracle_id = Uuid::parse_str(&oracle_id_str).map_err(|e| DbError::Unknown(e.to_string()))?;
    let rarity = serde_json::from_str(&format!("\"{rarity_str}\""))
        .map_err(|e| DbError::Unknown(e.to_string()))?;

    // Fallback naive date for robustness against parse failures on edge cases.
    let released_at = sqlx::types::chrono::NaiveDate::parse_from_str(&released_at_str, "%Y-%m-%d")
        .unwrap_or_else(|_| sqlx::types::chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());

    let image_uris = match image_uris_str {
        Some(s) => Some(serde_json::from_str(&s).map_err(|e| DbError::Unknown(e.to_string()))?),
        None => None,
    };
    let prices = match prices_str {
        Some(s) => Some(serde_json::from_str(&s).map_err(|e| DbError::Unknown(e.to_string()))?),
        None => None,
    };

    Ok(Printing {
        id,
        oracle_id,
        set_code: set_code.into(),
        collector_number: collector_number.into(),
        rarity,
        lang: lang.into(),
        released_at,
        image_uris,
        promo,
        digital,
        full_art,
        textless,
        reprint,
        prices,
    })
}

#[async_trait::async_trait]
impl CardRepository for SqliteBackend {
    async fn upsert_oracle(&self, card: &OracleCard) -> Result<(), DbError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        sqlx::query(
            "INSERT INTO oracle_cards (oracle_id, layout, color_identity, keywords, legalities, edh_rank, reserved) \
             VALUES (?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT (oracle_id) DO UPDATE SET \
             layout = excluded.layout, color_identity = excluded.color_identity, \
             keywords = excluded.keywords, legalities = excluded.legalities, \
             edh_rank = excluded.edh_rank, reserved = excluded.reserved"
        )
        .bind(card.oracle_id.to_string())
        .bind(serde_json::to_string(&card.layout).unwrap().trim_matches('"').to_string())
        .bind(i64::from(card.color_identity.bits()))
        .bind(serde_json::to_string(&card.keywords).unwrap())
        .bind(serde_json::to_string(&card.legalities).unwrap())
        .bind(card.edh_rank.map(i64::from))
        .bind(card.reserved)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        for (i, face) in card.faces.iter().enumerate() {
            let i_i64 = i64::try_from(i).unwrap_or(0);
            sqlx::query(
                "INSERT INTO card_faces (oracle_id, face_index, name, mana_cost, type_line, oracle_text, colors, power, toughness, loyalty, defense, flavor_text) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT (oracle_id, face_index) DO UPDATE SET \
                 name = excluded.name, mana_cost = excluded.mana_cost, \
                 type_line = excluded.type_line, oracle_text = excluded.oracle_text, \
                 colors = excluded.colors, power = excluded.power, \
                 toughness = excluded.toughness, loyalty = excluded.loyalty, \
                 defense = excluded.defense, flavor_text = excluded.flavor_text"
            )
            .bind(card.oracle_id.to_string())
            .bind(i_i64)
            .bind(face.name.to_string())
            .bind(face.mana_cost.as_ref().map(|m| serde_json::to_string(m).unwrap()))
            .bind(serde_json::to_string(&face.type_line).unwrap())
            .bind(serde_json::to_string(&face.oracle_text).unwrap())
            .bind(i64::from(face.colors.bits()))
            .bind(face.power.as_ref().map(|s| s.to_string()))
            .bind(face.toughness.as_ref().map(|s| s.to_string()))
            .bind(face.loyalty.as_ref().map(|s| s.to_string()))
            .bind(face.defense.as_ref().map(|s| s.to_string()))
            .bind(face.flavor_text.as_ref().map(|s| s.to_string()))
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;
        }

        tx.commit().await.map_err(|e| DbError::Sql(e.to_string()))?;
        Ok(())
    }

    async fn upsert_printing(&self, p: &Printing) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO printings (id, oracle_id, set_code, collector_number, rarity, lang, released_at, image_uris, promo, digital, full_art, textless, reprint, prices) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT (id) DO UPDATE SET \
             oracle_id = excluded.oracle_id, set_code = excluded.set_code, \
             collector_number = excluded.collector_number, rarity = excluded.rarity, \
             lang = excluded.lang, released_at = excluded.released_at, \
             image_uris = excluded.image_uris, promo = excluded.promo, \
             digital = excluded.digital, full_art = excluded.full_art, \
             textless = excluded.textless, reprint = excluded.reprint, prices = excluded.prices"
        )
        .bind(p.id.to_string())
        .bind(p.oracle_id.to_string())
        .bind(p.set_code.to_string())
        .bind(p.collector_number.to_string())
        .bind(serde_json::to_string(&p.rarity).unwrap().trim_matches('"').to_string())
        .bind(p.lang.to_string())
        .bind(p.released_at.format("%Y-%m-%d").to_string())
        .bind(p.image_uris.as_ref().map(|i| serde_json::to_string(i).unwrap()))
        .bind(p.promo)
        .bind(p.digital)
        .bind(p.full_art)
        .bind(p.textless)
        .bind(p.reprint)
        .bind(p.prices.as_ref().map(|pr| serde_json::to_string(pr).unwrap()))
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Printing>, DbError> {
        let row = sqlx::query("SELECT * FROM printings WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(parse_printing(&r)?)),
            None => Ok(None),
        }
    }

    async fn get_oracle(&self, oracle_id: Uuid) -> Result<Option<OracleCard>, DbError> {
        let rows = sqlx::query(
            "SELECT o.*, f.name, f.mana_cost, f.type_line, f.oracle_text, f.colors, f.power, f.toughness, f.loyalty, f.defense, f.flavor_text \
             FROM oracle_cards o \
             LEFT JOIN card_faces f ON o.oracle_id = f.oracle_id \
             WHERE o.oracle_id = ? \
             ORDER BY f.face_index ASC"
        )
        .bind(oracle_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        if rows.is_empty() {
            return Ok(None);
        }

        let mut card = parse_oracle_card(&rows[0])?;
        for row in &rows {
            // Because it's a LEFT JOIN, check if f.name is present to ensure face exists
            if row.try_get::<String, _>("name").is_ok() {
                let face = parse_card_face(row)?;
                card.faces.push(face);
            }
        }
        Ok(Some(card))
    }

    async fn search_name(&self, q: &str, limit: u32) -> Result<Vec<OracleCard>, DbError> {
        let pattern = format!("%{q}%");
        let rows = sqlx::query(
            "SELECT o.*, f.name, f.mana_cost, f.type_line, f.oracle_text, f.colors, f.power, f.toughness, f.loyalty, f.defense, f.flavor_text \
             FROM oracle_cards o \
             JOIN card_faces f ON o.oracle_id = f.oracle_id \
             WHERE o.oracle_id IN ( \
                 SELECT DISTINCT oracle_id \
                 FROM card_faces \
                 WHERE name LIKE ? \
                 LIMIT ? \
             ) \
             ORDER BY o.oracle_id, f.face_index ASC"
        )
        .bind(&pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        let mut cards: Vec<OracleCard> = Vec::new();
        for row in rows {
            let row_oracle_id_str: String = row.try_get("oracle_id").unwrap();
            let row_oracle_id = Uuid::parse_str(&row_oracle_id_str).unwrap();

            let is_new = cards.last().is_none_or(|c| c.oracle_id != row_oracle_id);
            if is_new {
                cards.push(parse_oracle_card(&row)?);
            }

            let face = parse_card_face(&row)?;
            cards.last_mut().unwrap().faces.push(face);
        }

        Ok(cards)
    }

    async fn search_fts(&self, q: &str, limit: u32) -> Result<Vec<OracleCard>, DbError> {
        // FTS match syntax requires q
        let rows = sqlx::query(
            "SELECT o.*, f.name, f.mana_cost, f.type_line, f.oracle_text, f.colors, f.power, f.toughness, f.loyalty, f.defense, f.flavor_text \
             FROM oracle_cards o \
             JOIN card_faces f ON o.oracle_id = f.oracle_id \
             WHERE o.oracle_id IN ( \
                 SELECT DISTINCT c.oracle_id \
                 FROM card_faces_fts fts \
                 JOIN card_faces c ON fts.rowid = c.rowid \
                 WHERE card_faces_fts MATCH ? \
                 LIMIT ? \
             ) \
             ORDER BY o.oracle_id, f.face_index ASC"
        )
        .bind(q)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        let mut cards: Vec<OracleCard> = Vec::new();
        for row in rows {
            let row_oracle_id_str: String = row.try_get("oracle_id").unwrap();
            let row_oracle_id = Uuid::parse_str(&row_oracle_id_str).unwrap();

            let is_new = cards.last().is_none_or(|c| c.oracle_id != row_oracle_id);
            if is_new {
                cards.push(parse_oracle_card(&row)?);
            }

            let face = parse_card_face(&row)?;
            cards.last_mut().unwrap().faces.push(face);
        }

        Ok(cards)
    }

    async fn cards_in_set(&self, set_code: &str) -> Result<Vec<Printing>, DbError> {
        let rows = sqlx::query("SELECT * FROM printings WHERE set_code = ?")
            .bind(set_code)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::Sql(e.to_string()))?;

        let mut printings = Vec::with_capacity(rows.len());
        for row in rows {
            printings.push(parse_printing(&row)?);
        }
        Ok(printings)
    }

    async fn legal_in_format(&self, format: &Format) -> Result<Vec<OracleCard>, DbError> {
        let path = format!("$.{}", format.0);
        let rows = sqlx::query(
            "SELECT o.*, f.name, f.mana_cost, f.type_line, f.oracle_text, f.colors, f.power, f.toughness, f.loyalty, f.defense, f.flavor_text \
             FROM oracle_cards o \
             JOIN card_faces f ON o.oracle_id = f.oracle_id \
             WHERE json_extract(o.legalities, ?) = 'legal' \
             ORDER BY o.oracle_id, f.face_index ASC"
        )
        .bind(&path)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::Sql(e.to_string()))?;

        let mut cards: Vec<OracleCard> = Vec::new();
        for row in rows {
            let row_oracle_id_str: String = row.try_get("oracle_id").unwrap();
            let row_oracle_id = Uuid::parse_str(&row_oracle_id_str).unwrap();

            let is_new = cards.last().is_none_or(|c| c.oracle_id != row_oracle_id);
            if is_new {
                cards.push(parse_oracle_card(&row)?);
            }

            let face = parse_card_face(&row)?;
            cards.last_mut().unwrap().faces.push(face);
        }

        Ok(cards)
    }
}
