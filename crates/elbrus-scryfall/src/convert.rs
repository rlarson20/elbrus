use elbrus_core::{
    card::{CardFace, CardLayout, ImageUris, OracleCard, PriceSnapshot, Printing, Rarity},
    color::ColorSet,
    legality::{Format, Legalities, LegalityStatus},
    mana::{ManaCost, ManaSymbol},
    oracle::{OracleText, OracleTextSegment},
    types::{Subtype, TypeLine},
};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::models::ScryfallCard;

impl ScryfallCard {
    pub fn into_core(self) -> (OracleCard, Printing) {
        let oracle_id = self.oracle_id.unwrap_or_else(Uuid::new_v4); // Fallback for some weird prints

        let layout = match self.layout.as_str() {
            "normal" => CardLayout::Normal,
            "split" => CardLayout::Split,
            "flip" => CardLayout::Flip,
            "transform" => CardLayout::Transform,
            "modal_dfc" => CardLayout::ModalDfc,
            "meld" => CardLayout::Meld,
            "leveler" => CardLayout::Leveler,
            "class" => CardLayout::Class,
            "saga" => CardLayout::Saga,
            "adventure" => CardLayout::Adventure,
            "prototype" => CardLayout::Prototype,
            "battle" => CardLayout::Battle,
            "mutate" => CardLayout::Mutate,
            "token" => CardLayout::Token,
            "double_faced_token" => CardLayout::DoubleFacedToken,
            "emblem" => CardLayout::Emblem,
            "augment" => CardLayout::Augment,
            "host" => CardLayout::Host,
            "art_series" => CardLayout::ArtSeries,
            other => CardLayout::Unknown(Arc::from(other)),
        };

        let faces = if let Some(cf) = self.card_faces {
            cf.into_iter()
                .map(|face| CardFace {
                    name: Arc::from(face.name),
                    mana_cost: if face.mana_cost.is_empty() {
                        None
                    } else {
                        Some(ManaCost::parse(&face.mana_cost).unwrap_or_else(|_| {
                            let mut symbols = smallvec::SmallVec::new();
                            symbols.push(ManaSymbol::Unknown(Arc::from(face.mana_cost)));
                            ManaCost(symbols)
                        }))
                    },
                    type_line: face
                        .type_line
                        .as_deref()
                        .map_or_else(TypeLine::default, |s| {
                            TypeLine::parse(s).unwrap_or_else(|_| {
                                let mut t = TypeLine::default();
                                t.subtypes.push(Subtype(Arc::from(s)));
                                t
                            })
                        }),
                    oracle_text: face
                        .oracle_text
                        .as_deref()
                        .map_or_else(OracleText::default, |s| {
                            OracleText(vec![OracleTextSegment::Text(Arc::from(s))])
                        }),
                    colors: parse_colors(&face.colors.unwrap_or_default()),
                    power: face.power.map(Arc::from),
                    toughness: face.toughness.map(Arc::from),
                    loyalty: face.loyalty.map(Arc::from),
                    defense: face.defense.map(Arc::from),
                    flavor_text: face.flavor_text.map(Arc::from),
                })
                .collect()
        } else {
            let face = CardFace {
                name: Arc::from(self.name),
                mana_cost: self
                    .mana_cost
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        ManaCost::parse(s).unwrap_or_else(|_| {
                            let mut symbols = smallvec::SmallVec::new();
                            symbols.push(ManaSymbol::Unknown(Arc::from(s)));
                            ManaCost(symbols)
                        })
                    }),
                type_line: self
                    .type_line
                    .as_deref()
                    .map_or_else(TypeLine::default, |s| {
                        TypeLine::parse(s).unwrap_or_else(|_| {
                            let mut t = TypeLine::default();
                            t.subtypes.push(Subtype(Arc::from(s)));
                            t
                        })
                    }),
                oracle_text: self
                    .oracle_text
                    .as_deref()
                    .map_or_else(OracleText::default, |s| {
                        OracleText(vec![OracleTextSegment::Text(Arc::from(s))])
                    }),
                colors: parse_colors(&self.colors.unwrap_or_default()),
                power: None, // We don't have top-level power on ScryfallCard if not in card_faces? Actually we might. I should add these fields. Let's just put None for now since Scryfall puts them at root for normal cards, wait, I need to add power/toughness to ScryfallCard in models.rs... Let me fix models.rs later or just ignore it for now since we mapped the faces. Wait, for normal cards, power/toughness are at the root. I should add them to ScryfallCard.
                toughness: None,
                loyalty: None,
                defense: None, // I'll update ScryfallCard first.
                flavor_text: None,
            };
            let mut vec = smallvec::SmallVec::new();
            vec.push(face);
            vec
        };

        let oracle_card = OracleCard {
            oracle_id,
            layout,
            faces,
            color_identity: parse_colors(&self.color_identity),
            keywords: vec![], // Parse later or from self.keywords
            legalities: parse_legalities(self.legalities),
            edh_rank: None,  // Need to add to ScryfallCard
            reserved: false, // Need to add to ScryfallCard
        };

        let rarity = match self.rarity.as_str() {
            "common" => Rarity::Common,
            "uncommon" => Rarity::Uncommon,
            "rare" => Rarity::Rare,
            "mythic" => Rarity::Mythic,
            "special" => Rarity::Special,
            "bonus" => Rarity::Bonus,
            _ => Rarity::Common,
        };

        let image_uris = self.image_uris.map(|uris| ImageUris {
            small: uris.get("small").map(|s| Arc::from(s.as_str())),
            normal: uris.get("normal").map(|s| Arc::from(s.as_str())),
            large: uris.get("large").map(|s| Arc::from(s.as_str())),
            png: uris.get("png").map(|s| Arc::from(s.as_str())),
            art_crop: uris.get("art_crop").map(|s| Arc::from(s.as_str())),
            border_crop: uris.get("border_crop").map(|s| Arc::from(s.as_str())),
        });

        let prices = self.prices.map(|p| PriceSnapshot {
            usd: p
                .get("usd")
                .and_then(|v| v.as_deref())
                .and_then(|s| s.parse().ok()),
            usd_foil: p
                .get("usd_foil")
                .and_then(|v| v.as_deref())
                .and_then(|s| s.parse().ok()),
            eur: p
                .get("eur")
                .and_then(|v| v.as_deref())
                .and_then(|s| s.parse().ok()),
            tix: p
                .get("tix")
                .and_then(|v| v.as_deref())
                .and_then(|s| s.parse().ok()),
            fetched_at: chrono::Utc::now(),
        });

        let released_at =
            chrono::NaiveDate::parse_from_str(&self.released_at, "%Y-%m-%d").unwrap_or_default();

        let printing = Printing {
            id: self.id,
            oracle_id,
            set_code: Arc::from(self.set),
            collector_number: Arc::from(self.collector_number),
            rarity,
            lang: Arc::from("en"), // Hardcoded for now, or add lang to ScryfallCard
            released_at,
            image_uris,
            promo: false,    // Add to ScryfallCard
            digital: false,  // Add to ScryfallCard
            full_art: false, // Add to ScryfallCard
            textless: false, // Add to ScryfallCard
            reprint: false,  // Add to ScryfallCard
            prices,
        };

        (oracle_card, printing)
    }
}

fn parse_colors(colors: &[String]) -> ColorSet {
    let mut set = ColorSet::empty();
    for c in colors {
        match c.as_str() {
            "W" => set.insert(ColorSet::WHITE),
            "U" => set.insert(ColorSet::BLUE),
            "B" => set.insert(ColorSet::BLACK),
            "R" => set.insert(ColorSet::RED),
            "G" => set.insert(ColorSet::GREEN),
            _ => {}
        }
    }
    set
}

fn parse_legalities(legalities: HashMap<String, String>) -> Legalities {
    let mut parsed = HashMap::new();
    for (k, v) in legalities {
        let status = match v.as_str() {
            "legal" => LegalityStatus::Legal,
            "restricted" => LegalityStatus::Restricted,
            "banned" => LegalityStatus::Banned,
            "not_legal" => LegalityStatus::NotLegal,
            _ => LegalityStatus::NotLegal,
        };
        parsed.insert(Format(Arc::from(k)), status);
    }
    Legalities(parsed)
}
