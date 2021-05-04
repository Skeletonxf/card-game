use crate::card_type::CardType;

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

#[derive(Debug)]
pub struct Card<'card_pool> {
    pub card_type: &'card_pool CardType,
    pub state: CardStatus,
    pub instance: CardInstance,
}

impl<'a> Card<'a> {
    pub fn id(&self) -> u32 {
        self.card_type.id
    }

    pub fn can_activate(&self, game_state: &GameState) -> Vec<ActivationStatus> {
        self.card_type
            .effects
            .iter()
            .map(|card| card.can_activate(self.card_type, game_state, self.instance))
            .collect()
    }
}

#[derive(Default, Debug)]
pub struct GameState<'card_pool> {
    pub field: BTreeMap<i32, Card<'card_pool>>,
    pub graveyard: BTreeMap<i32, Vec<Card<'card_pool>>>,
    pub hand: Vec<Card<'card_pool>>,
    pub left_deck: Vec<Card<'card_pool>>,
    pub right_deck: Vec<Card<'card_pool>>,
}

/// A unique id assigned to a Card to uniquely identify the copy
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CardInstance(pub u32);

/// The ith card effect a CardType may have
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CardEffectID(pub u32);

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

impl<'a> GameState<'a> {
    pub fn actions(&self) -> Vec<Action> {
        let mut mandatory = Vec::new();
        let mut optional = Vec::new();
        for card in self.field.values() {
            for (n, effect_status) in card.can_activate(self).iter().enumerate() {
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

    pub fn take_action(&mut self, action: Action) -> Result<(), Box<dyn std::error::Error>> {
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
            _ => (),
        }
        Ok(())
    }
}
