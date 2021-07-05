use crate::card_type::{CardType, CardTypeIdentifier};

use once_cell::sync::Lazy;
use std::fs;

static CARDS: Lazy<Cards> = Lazy::new(|| Cards::load().unwrap());

pub struct Cards {
    cards: Vec<CardType>,
}

impl Cards {
    pub fn get() -> &'static Self {
        &CARDS
    }

    // TODO: Generic directory walking should be extracted
    // TODO: Walk subfolders
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let mut cards = Vec::new();
        let mut id = 0;
        for entry in fs::read_dir("data/cards")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let str = String::from_utf8(fs::read(path)?)?;
                let mut parsed: CardType = toml::from_str(&str)?;
                // Here we maintain the invariant that the position of a CardType in our cards Vec
                // is also the CardTypeIdentifier that we assign to the CardType, which ensures
                // we have 0(1) lookup when fetching cards by ID
                parsed.id = CardTypeIdentifier(id);
                id += 1;
                cards.push(parsed);
            }
        }
        Ok(Cards { cards })
    }

    pub fn card<R: Into<Reference>>(&self, reference: R) -> Option<&CardType> {
        let reference: Reference = reference.into();
        match reference {
            Reference::Identifier(id) => self.cards.iter().find(|s| s.name == id),
            Reference::StaticIdentifier(id) => self.cards.iter().find(|s| s.name == id),
            Reference::ID(id) => self.cards.get(id.0 as usize),
        }
    }

    pub fn from_test(toml_cards: Vec<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cards = Vec::new();
        let mut id = 0;
        for str in toml_cards.iter() {
            let mut parsed: CardType = toml::from_str(&str)?;
            parsed.id = CardTypeIdentifier(id);
            id += 1;
            cards.push(parsed);
        }
        Ok(Cards { cards })
    }
}

pub enum Reference {
    Identifier(String),
    StaticIdentifier(&'static str),
    ID(CardTypeIdentifier),
}

impl From<CardTypeIdentifier> for Reference {
    fn from(id: CardTypeIdentifier) -> Self {
        Reference::ID(id)
    }
}

impl From<&'static str> for Reference {
    fn from(identifier: &'static str) -> Self {
        Reference::StaticIdentifier(identifier)
    }
}

impl From<String> for Reference {
    fn from(identifier: String) -> Self {
        Reference::Identifier(identifier)
    }
}
