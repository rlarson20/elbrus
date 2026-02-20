use elbrus_core::Format;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DeckError {
    #[error("Parse error: {0}")]
    Parse(String),
}

pub struct Deck {
    pub name: Option<Arc<str>>,
    pub format: Option<Format>,
    pub mainboard: Vec<DeckEntry>,
    pub sideboard: Vec<DeckEntry>,
    pub commander: Vec<DeckEntry>, // EDH
    pub companion: Option<DeckEntry>,
}

pub struct DeckEntry {
    pub quantity: u32,
    pub card_name: Arc<str>,
    pub resolved: Option<Uuid>, // filled after db lookup
    pub set_hint: Option<Arc<str>>,
    pub foil: bool,
}

pub trait DeckParser: Send + Sync {
    fn can_parse(&self, input: &str) -> bool;
    fn parse(&self, input: &str) -> Result<Deck, DeckError>;
    fn serialize(&self, deck: &Deck) -> String;
}

pub struct ArenaParser;
pub struct MtgoParser;
pub struct MoxfieldParser; // URL-fetched or exported text

impl DeckParser for ArenaParser {
    fn can_parse(&self, _input: &str) -> bool {
        todo!()
    }
    fn parse(&self, _input: &str) -> Result<Deck, DeckError> {
        todo!()
    }
    fn serialize(&self, _deck: &Deck) -> String {
        todo!()
    }
}
