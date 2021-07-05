use crate::cards::Cards;
use crate::card_type::{CardTypeIdentifier, CardType};

use std::collections::BTreeMap;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CardStatus {
    Drawn,
    Discarded,
    Summoned,
    Destroyed,
    ReturnedToHand,
    None,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ActivationStatus {
    Cannot,
    Can,
    Mandatory,
}

impl ActivationStatus {
    pub fn possible(&self) -> bool {
        self != &ActivationStatus::Cannot
    }
}

#[derive(Debug)]
pub struct Card {
    pub card_type: CardTypeIdentifier,
    pub state: CardStatus,
    pub instance: CardInstance,
}

impl Card {
    pub fn instance_of(&self, card_type: &CardType) -> bool {
        self.card_type == card_type.id
    }

    pub fn has_name(&self, card_pool: &Cards, name: &str) -> bool {
        self.lookup_self(card_pool).name == name
    }

    pub fn lookup_self<'a>(&self, card_pool: &'a Cards) -> &'a CardType {
        card_pool
            .card(self.card_type)
            .expect("CardType lookup should always succeed since we create card instances from the card pool")
    }

    pub fn can_activate(&self, card_pool: &Cards, game_state: &GameState) -> Vec<ActivationStatus> {
        let card_type = self.lookup_self(card_pool);
        card_type.effects
            .iter()
            .map(|card| card.can_activate(card_pool, card_type, game_state, self.instance))
            .collect()
    }
}

/// A particular game state, defines where all the card instances are. Conceptually this should
/// live for no longer than the card pool does, since every card instance only works by looking
/// up their card type from the card pool first. However, the card pool is essentially 'static
/// apart from unit testing, and everything is much easier if the card pool exists seperately
/// to the gamestate since then there are no issues with a card type (stored in the card pool)
/// freely mutating the game state.
#[derive(Default, Debug)]
pub struct GameState {
    pub field: BTreeMap<i32, Card>,
    pub graveyard: BTreeMap<i32, Vec<Card>>,
    pub hand: Vec<Card>,
    pub left_deck: Vec<Card>,
    pub right_deck: Vec<Card>,
}

/// A unique id assigned to a Card to uniquely identify the copy
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CardInstance(pub u32);

/// The ith card effect a CardType may have
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CardEffect(pub u32);

impl From<usize> for CardEffect {
    fn from(index: usize) -> Self {
        CardEffect(index as u32)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// Summon a card from the hand onto the next available slot on the field.
    SummonFromHand(CardInstance),
    /// Activate an effect of a card already on the field.
    ActivateFromField(CardInstance, CardEffect),
    /// Destroy a card, moving it from the field to the graveyard, retaining its column from the field.
    DestroyOnField(CardInstance),
}
#[derive(Debug, Clone)]
struct InvalidAction;

impl fmt::Display for InvalidAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Action")
    }
}

impl std::error::Error for InvalidAction {}

impl GameState {
    pub fn actions(&self, card_pool: &Cards) -> Vec<Action> {
        let mut mandatory = Vec::new();
        let mut optional = Vec::new();
        for card in self.field.values() {
            for (n, effect_status) in card.can_activate(card_pool, self).iter().enumerate() {
                match effect_status {
                    ActivationStatus::Mandatory => {
                        mandatory.push(Action::ActivateFromField(card.instance, n.into()))
                    }
                    ActivationStatus::Can => {
                        optional.push(Action::ActivateFromField(card.instance, n.into()))
                    }
                    _ => (),
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

    pub fn take_action(&mut self, card_pool: &Cards, action: Action) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            Action::SummonFromHand(instance) => {
                match self
                    .hand
                    .iter()
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
            Action::ActivateFromField(instance, effect) => {
                match self
                    .field
                    .iter()
                    .find(|(_, card)| card.instance == instance)
                    .map(|(i, _)| i)
                    .cloned()
                {
                    Some(index) => {
                        let card = self.field.get(&index).expect("card always present");
                        let card_type = card.lookup_self(card_pool);
                        match card_type.effects.get(effect.0 as usize) {
                            Some(effect_type) => {
                                if effect_type.can_activate(card_pool, card_type, self, instance).possible() {
                                    effect_type.activate(card_pool, card_type, self, instance);
                                } else {
                                    return Err(Box::new(InvalidAction));
                                }
                            }
                            None => return Err(Box::new(InvalidAction)),
                        }
                    }
                    None => return Err(Box::new(InvalidAction)),
                }
            }
            Action::DestroyOnField(instance) => {
                match self
                    .field
                    .iter()
                    .find(|(_, card)| card.instance == instance)
                    .map(|(i, _)| i)
                    .cloned()
                {
                    Some(index) => {
                        let mut card = self.field.remove(&index).expect("card always present");
                        card.state = CardStatus::Destroyed;
                        self
                            .graveyard
                            .entry(index)
                            .or_insert_with(|| Vec::with_capacity(1))
                            .push(card);
                    }
                    None => return Err(Box::new(InvalidAction)),
                }
            }
        }
        Ok(())
    }
}
