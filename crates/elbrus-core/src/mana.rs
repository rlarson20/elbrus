use crate::color::{Color, ColorSet};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::sync::Arc;

/// A single symbol: {W}, {2/U}, {X}, {T}, {C}, {CHAOS}, etc.
/// TODO: not all symbols are mana symbols
/// put into a separate enum
/// and just combine into Symbol enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ManaSymbol {
    Colored(Color),
    Generic(u32),
    Variable,  // X
    Colorless, // C
    Snow,      // S
    Hybrid(Color, Color),
    TwoBrid(Color),   // {2/W}
    Phyrexian(Color), // {W/P}
    HybridPhyrexian(Color, Color),
    Tap, // {T} (for reminder text)
    Unknown(Arc<str>),
}

/// Mana cost as ordered sequence of symbols.
/// Use SmallVec<[_; 8]>: typical spell costs fit on stack.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ManaCost(pub SmallVec<[ManaSymbol; 8]>);

impl ManaCost {
    pub fn cmc(&self) -> f32 {
        todo!()
    }
    pub fn color_identity(&self) -> ColorSet {
        todo!()
    }
    pub fn parse(_s: &str) -> Result<Self, crate::error::CoreError> {
        todo!()
    }
}
