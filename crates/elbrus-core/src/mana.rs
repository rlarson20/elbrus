use crate::color::{Color, ColorSet};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VarSym {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenericCost(u32);

impl GenericCost {
    pub const C1: Self = Self(1);
    pub const C2: Self = Self(2);
    pub const C3: Self = Self(3);
    pub const C4: Self = Self(4);
    pub const C5: Self = Self(5);
    pub const C6: Self = Self(6);
    pub const C7: Self = Self(7);
    pub const C8: Self = Self(8);
    pub const C9: Self = Self(9);
    pub const C10: Self = Self(10);
    pub const C11: Self = Self(11);
    pub const C12: Self = Self(12);
    pub const C13: Self = Self(13);
    pub const C14: Self = Self(14);
    pub const C15: Self = Self(15);
    pub const C16: Self = Self(16);
    pub const C17: Self = Self(17);
    pub const C18: Self = Self(18);
    pub const C19: Self = Self(19);
    pub const C20: Self = Self(20);
    pub const C100: Self = Self(100);
    pub const C1000000: Self = Self(1_000_000);
    pub const INFINITY: Self = Self(u32::MAX);

    pub const fn new(n: u32) -> Self {
        Self(n)
    }
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// A single symbol: {W}, {2/U}, {X}, {T}, {C}, {CHAOS}, etc.
/// TODO: not all symbols are mana symbols
/// put into a separate enum
/// and just combine into Symbol enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ManaSymbol {
    Colored(Color),
    Generic(GenericCost),
    Variable(VarSym), // X, Y, Z
    Colorless,        // C
    Snow,             // S
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
        Err(crate::error::CoreError::ParseError(
            "not implemented".into(),
        ))
    }
}
