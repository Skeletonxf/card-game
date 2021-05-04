use std::collections::BTreeMap;
use std::fs;
use std::fmt;
use once_cell::sync::Lazy;

use serde::{Deserialize, Serialize};

/// A card type is like the class for cards. Multiple copies of Cards may share
/// a CardType that they are a copy of.
#[derive(Debug, Deserialize, Serialize)]
pub struct CardType {
    #[serde(skip_deserializing)]
    id: u32,
    name: String,
    #[serde(default)]
    effects: Vec<Box<dyn CardEffect>>,
    defense: u32,
    attack: u32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ActivationStatus {
    Cannot,
    Can,
    Mandatory
}

#[typetag::serde(tag = "type")]
pub trait CardEffect: Send + Sync + fmt::Debug {
    fn can_activate(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> ActivationStatus;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OnSummon {}

#[typetag::serde]
impl CardEffect for OnSummon {
    fn can_activate(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> ActivationStatus {
        if game_state.field.values()
            .any(|card|
                card.instance == instance
                && card.id() == card_type.id
                && card.state == CardStatus::Summoned
            )
        {
            ActivationStatus::Can
        } else {
            ActivationStatus::Cannot
        }
    }
}

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
                parsed.id = id;
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
            Reference::ID(id) => self.cards.get(id as usize),
        }
    }
}

pub enum Reference {
    Identifier(String),
    StaticIdentifier(&'static str),
    ID(u32),
}

impl From<u32> for Reference {
    fn from(id: u32) -> Self {
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CardStatus {
    Drawn,
    Discarded,
    Summoned,
    Destroyed,
    ReturnedToHand,
    None,
}

#[derive(Debug)]
pub struct Card<'card_pool> {
    card_type: &'card_pool CardType,
    state: CardStatus,
    instance: CardInstance,
}

impl <'a> Card<'a> {
    fn id(&self) -> u32 {
        self.card_type.id
    }

    fn can_activate(&self, game_state: &GameState) -> Vec<ActivationStatus> {
        self.card_type.effects.iter()
            .map(|card| card.can_activate(self.card_type, game_state, self.instance))
            .collect()
    }
}

#[derive(Default, Debug)]
pub struct GameState<'card_pool> {
    field: BTreeMap<i32, Card<'card_pool>>,
    graveyard: BTreeMap<i32, Vec<Card<'card_pool>>>,
    hand: Vec<Card<'card_pool>>,
    left_deck: Vec<Card<'card_pool>>,
    right_deck: Vec<Card<'card_pool>>,
}

/// A unique id assigned to a Card to uniquely identify the copy
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CardInstance(u32);

/// The ith card effect a CardType may have
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CardEffectID(u32);

impl From<usize> for CardEffectID {
    fn from(index: usize) -> Self {
        CardEffectID(index as u32)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Action {
    SummonFromHand(CardInstance),
    ActivateFromField(CardInstance, CardEffectID),
}
#[derive(Debug, Clone)]
struct InvalidAction;

impl fmt::Display for InvalidAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Action")
    }
}

impl std::error::Error for InvalidAction {}

impl <'a> GameState<'a> {
    pub fn actions(&self) -> Vec<Action> {
        let mut mandatory = Vec::new();
        let mut optional = Vec::new();
        for card in self.field.values() {
            for (n, effect_status) in card.can_activate(self).iter().enumerate() {
                match effect_status {
                    ActivationStatus::Mandatory => mandatory.push(Action::ActivateFromField(card.instance, n.into())),
                    ActivationStatus::Can => optional.push(Action::ActivateFromField(card.instance, n.into())),
                    _ => ()
                }
            }
        }
        for card in self.hand.iter() {
            optional.push(Action::SummonFromHand(card.instance));
        }
        if mandatory.is_empty() {
            optional
        } else {
            mandatory
        }
    }

    fn empty_slot_on_field(&self) -> i32 {
        let mut index = 0;
        loop {
            if !self.field.contains_key(&index) {
                return index;
            }
            if index >= 0 {
                index = -(index + 1);
            } else {
                index = -index
            }
        }
    }

    pub fn take_action(&mut self, action: Action) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            Action::SummonFromHand(instance) => {
                match self.hand.iter()
                    .enumerate()
                    .find(|(_, card)| card.instance == instance)
                    .map(|(i, _)| i)
                {
                    Some(index) => {
                        let mut card = self.hand.remove(index);
                        card.state = CardStatus::Summoned;
                        let slot = self.empty_slot_on_field();
                        self.field.insert(slot, card);
                    }
                    None => {
                        return Err(Box::new(InvalidAction));
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let cards = Cards::get();
        let card = cards.card("Staple Dragon").unwrap();
        assert_eq!(card.name, "Staple Dragon");
    }

    #[test]
    fn summon_from_hand() {
        let card = CardType {
            id: 0,
            name: "Staple Dragon".to_owned(),
            effects: vec![
                Box::new(OnSummon {})
            ],
            defense: 5,
            attack: 6,
        };
        let mut player = GameState {
            hand: vec![
                Card {
                    card_type: &card,
                    state: CardStatus::None,
                    instance: CardInstance(0),
                }
            ],
            ..Default::default()
        };
        let actions = player.actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::SummonFromHand(CardInstance(0)));
        let result = player.take_action(actions[0]);
        assert!(result.is_ok());
        assert!(player.field.values().any(|card| card.instance == CardInstance(0)));
        assert!(!player.hand.iter().any(|card| card.instance == CardInstance(0)));
        let actions = player.actions();
        println!("Actions: {:?}", actions);
    }
}
