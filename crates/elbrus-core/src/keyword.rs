use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Comprehensive Rules keyword abilities and keyword actions.
/// #[non_exhaustive] because Wizards adds keywords every set.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Keyword {
    // Keyword abilities (CR 702)
    Deathtouch,
    Defender,
    DoubleStrike,
    Enchant,
    Equip,
    FirstStrike,
    Flash,
    Flying,
    Haste,
    Hexproof,
    Indestructible,
    Intimidate,
    Landwalk,
    Lifelink,
    Menace,
    Protection,
    Reach,
    Shroud,
    Trample,
    Vigilance,
    Infect,
    Wither,
    Persist,
    Undying,
    Riot,
    Cascade,
    Convoke,
    Delve,
    Emerge,
    Escape,
    Foretell,
    Jumpstart,
    Kicker,
    Mutate,
    Overload,
    Replicate,
    Spectacle,
    Surge,
    Transmute,
    Unearth,
    Improvise,
    Affinity,
    Aftermath,
    Bestow,
    Cycling,
    Dash,
    Evoke,
    Flashback,
    Madness,
    Miracle,
    Morph,
    Ninjutsu,
    Prowl,
    Suspend,
    Transfigure,
    Ward,
    Ravenous,
    Squad,
    // Keyword actions (CR 701) — subset needed for segment parsing
    Scry,
    Surveil,
    Mill,
    Investigate,
    Explore,
    // Fallback for future/unknown keywords
    Unknown(Arc<str>),
}

impl Keyword {
    pub fn takes_cost(&self) -> bool {
        todo!()
    }
    pub fn is_evasion(&self) -> bool {
        todo!()
    }
}
