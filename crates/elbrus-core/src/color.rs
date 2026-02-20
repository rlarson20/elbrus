use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct ColorSet: u8 {
        const WHITE = 0b00001;
        const BLUE  = 0b00010;
        const BLACK = 0b00100;
        const RED   = 0b01000;
        const GREEN = 0b10000;
    }
}

impl ColorSet {
    pub fn is_colorless(self) -> bool {
        self.is_empty()
    }
    pub fn is_multicolor(self) -> bool {
        self.bits().count_ones() > 1
    }
    pub fn devotion(self, _color: ColorSet) -> u8 {
        todo!()
    }
}

/// Pips in a mana cost, in WUBRG order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    W,
    U,
    B,
    R,
    G,
}
