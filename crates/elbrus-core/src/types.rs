use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Supertype {
    Basic,
    Legendary,
    Snow,
    World,
    Token,
    Unknown(Arc<str>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CardType {
    Artifact,
    Battle,
    Conspiracy,
    Creature,
    Dungeon,
    Enchantment,
    Instant,
    Land,
    Phenomenon,
    Plane,
    Planeswalker,
    Scheme,
    Sorcery,
    Tribal,
    Vanguard,
    Unknown(Arc<str>),
}

/// Subtype is an open set — Wizards adds creature types, land types, spell types freely.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subtype(pub Arc<str>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TypeLine {
    pub supertypes: SmallVec<[Supertype; 2]>,
    pub card_types: SmallVec<[CardType; 2]>,
    pub subtypes: SmallVec<[Subtype; 4]>,
}

impl TypeLine {
    pub fn is_creature(&self) -> bool {
        self.card_types.contains(&CardType::Creature)
    }
    pub fn is_permanent(&self) -> bool {
        self.card_types.iter().any(|t| {
            matches!(
                t,
                CardType::Artifact
                    | CardType::Battle
                    | CardType::Creature
                    | CardType::Enchantment
                    | CardType::Land
                    | CardType::Planeswalker
            )
        })
    }
    pub fn parse(_s: &str) -> Result<Self, crate::error::CoreError> {
        Err(crate::error::CoreError::ParseError(
            "not implemented".into(),
        ))
    }
}
