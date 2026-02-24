use crate::{
    color::ColorSet, legality::Legalities, mana::ManaCost, oracle::OracleText, types::TypeLine,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Canonical card face. A split/MDFC card has 2 of these.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardFace {
    pub name: Arc<str>,
    pub mana_cost: Option<ManaCost>,
    pub type_line: TypeLine,
    pub oracle_text: OracleText,
    pub colors: ColorSet,
    pub power: Option<Arc<str>>, // Keep as str: */2+*, etc.
    pub toughness: Option<Arc<str>>,
    pub loyalty: Option<Arc<str>>,
    pub defense: Option<Arc<str>>, // Battle defense
    pub flavor_text: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CardLayout {
    Normal,
    Split,
    Flip,
    Transform,
    ModalDfc,
    Meld,
    Leveler,
    Class,
    Saga,
    Adventure,
    Prototype,
    Battle,
    Mutate,
    Token,
    DoubleFacedToken,
    Emblem,
    Augment,
    Host,
    ArtSeries,
    Unknown(Arc<str>),
}

/// The Oracle card — one record per unique `oracle_id`.
/// Printing-specific data (set, collector number, art, prices) lives in `Printing`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OracleCard {
    /// Scryfall `oracle_id` — stable across printings.
    pub oracle_id: Uuid,
    pub layout: CardLayout,
    pub faces: smallvec::SmallVec<[CardFace; 2]>,
    pub color_identity: ColorSet,
    pub keywords: Vec<crate::keyword::Keyword>,
    pub legalities: Legalities,
    pub edh_rank: Option<u32>,
    pub reserved: bool,
}

impl OracleCard {
    #[must_use]
    pub fn primary_face(&self) -> &CardFace {
        &self.faces[0]
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.primary_face().name
    }
    #[must_use]
    pub fn cmc(&self) -> f32 {
        self.primary_face()
            .mana_cost
            .as_ref()
            .map_or(0.0, |m| m.cmc())
    }
}

/// A specific printing — one record per Scryfall card id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Printing {
    /// Scryfall card id — the canonical PK throughout elbrus.
    pub id: Uuid,
    pub oracle_id: Uuid,
    pub set_code: Arc<str>,
    pub collector_number: Arc<str>,
    pub rarity: Rarity,
    pub lang: Arc<str>,
    pub released_at: chrono::NaiveDate,
    pub image_uris: Option<ImageUris>,
    pub promo: bool,
    pub digital: bool,
    pub full_art: bool,
    pub textless: bool,
    pub reprint: bool,
    pub prices: Option<PriceSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
    Special,
    Bonus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageUris {
    pub small: Option<Arc<str>>,
    pub normal: Option<Arc<str>>,
    pub large: Option<Arc<str>>,
    pub png: Option<Arc<str>>,
    pub art_crop: Option<Arc<str>>,
    pub border_crop: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub usd: Option<rust_decimal::Decimal>,
    pub usd_foil: Option<rust_decimal::Decimal>,
    pub eur: Option<rust_decimal::Decimal>,
    pub tix: Option<rust_decimal::Decimal>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}
