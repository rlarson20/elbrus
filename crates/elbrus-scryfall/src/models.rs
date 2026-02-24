use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScryfallError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Network error: {0}")]
    Network(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScryfallCard {
    pub id: uuid::Uuid,
    pub oracle_id: Option<uuid::Uuid>,
    pub name: String,
    pub layout: String,

    #[serde(default)]
    pub mana_cost: Option<String>,

    #[serde(default)]
    pub type_line: Option<String>,

    #[serde(default)]
    pub oracle_text: Option<String>,

    #[serde(default)]
    pub colors: Option<Vec<String>>,

    #[serde(default)]
    pub color_identity: Vec<String>,

    #[serde(default)]
    pub keywords: Vec<String>,

    #[serde(default)]
    pub legalities: HashMap<String, String>,

    pub set: String,

    #[serde(default)]
    pub collector_number: String,

    pub rarity: String,

    #[serde(default)]
    pub released_at: String,

    #[serde(default)]
    pub power: Option<String>,

    #[serde(default)]
    pub toughness: Option<String>,

    #[serde(default)]
    pub flavor_text: Option<String>,

    #[serde(default)]
    pub lang: String,

    #[serde(default)]
    pub edhrec_rank: Option<u32>,

    #[serde(default)]
    pub reserved: bool,

    #[serde(default)]
    pub promo: bool,

    #[serde(default)]
    pub digital: bool,

    #[serde(default)]
    pub full_art: bool,

    #[serde(default)]
    pub textless: bool,

    #[serde(default)]
    pub reprint: bool,

    #[serde(default)]
    pub image_uris: Option<HashMap<String, String>>,

    #[serde(default)]
    pub prices: Option<HashMap<String, Option<String>>>,

    #[serde(default)]
    pub card_faces: Option<Vec<ScryfallCardFace>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScryfallCardFace {
    pub name: String,

    #[serde(default)]
    pub mana_cost: String,

    #[serde(default)]
    pub type_line: Option<String>,

    #[serde(default)]
    pub oracle_text: Option<String>,

    #[serde(default)]
    pub colors: Option<Vec<String>>,

    #[serde(default)]
    pub power: Option<String>,

    #[serde(default)]
    pub toughness: Option<String>,

    #[serde(default)]
    pub loyalty: Option<String>,

    #[serde(default)]
    pub defense: Option<String>,

    #[serde(default)]
    pub flavor_text: Option<String>,

    #[serde(default)]
    pub image_uris: Option<HashMap<String, String>>,
}
