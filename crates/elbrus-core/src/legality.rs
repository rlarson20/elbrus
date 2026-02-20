use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LegalityStatus {
    Legal,
    Restricted,
    Banned,
    NotLegal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Format(pub Arc<str>); // open set

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Legalities(pub HashMap<Format, LegalityStatus>);

impl Legalities {
    pub fn is_legal_in(&self, format: &Format) -> bool {
        self.0.get(format).copied() == Some(LegalityStatus::Legal)
    }
}
