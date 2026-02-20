use serde::{Deserialize, Serialize};

/// Unique identifier for a game object (card, token, etc.) during simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u32);

/// Identifies a game zone (Library, Hand, Battlefield, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneId(pub u8);
