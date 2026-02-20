use crate::keyword::Keyword;
use crate::mana::ManaCost;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OracleTextSegment {
    /// Plain prose text (trimmed sentence fragments between other segments).
    Text(Arc<str>),
    /// Inline mana symbol: {W}, {2/B}, etc.
    ManaCost(ManaCost),
    /// Keyword ability with optional cost/parameters.
    Keyword {
        keyword: Keyword,
        parameter: Option<Arc<str>>,
    },
    /// Reminder text in parentheses.
    Reminder(Arc<str>),
    /// Loyalty/energy/counter symbols.
    Symbol(Arc<str>),
    /// Section separator — double newline in Scryfall JSON.
    Paragraph,
}

/// Structured oracle text. Never a raw String in public API.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct OracleText(pub Vec<OracleTextSegment>);

impl OracleText {
    /// Lossy round-trip for display; do NOT use as canonical form.
    pub fn to_display_string(&self) -> String {
        todo!()
    }
    pub fn keywords(&self) -> impl Iterator<Item = &Keyword> {
        self.0.iter().filter_map(|s| {
            if let OracleTextSegment::Keyword { keyword, .. } = s {
                Some(keyword)
            } else {
                None
            }
        })
    }
}
