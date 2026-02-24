use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Condition {
    NearMint,
    LightlyPlayed,
    ModeratelyPlayed,
    HeavyPlayed,
    Damaged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionEntry {
    pub collection_id: Uuid,
    pub printing_id: Uuid,
    pub quantity: u32,
    pub condition: Condition,
    pub foil: bool,
    pub notes: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub name: Arc<str>,
    pub description: Option<Arc<str>>,
    pub entries: Vec<CollectionEntry>,
}
