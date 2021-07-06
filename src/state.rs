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
    Can,
    Mandatory,
}

/// A possible way that a card effect can be activated, if at all.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Activation {
    pub status: ActivationStatus,
    pub data: ActivationData,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct ActivationData {
    pub slot: Option<i32>,
}

impl Default for Activation {
    fn default() -> Self {
        Activation {
            status: ActivationStatus::Can,
            data: ActivationData::default(),
        }
    }
}

impl From<ActivationStatus> for Activation {
    fn from(activation_status: ActivationStatus) -> Self {
        Activation {
            status: activation_status,
            ..Default::default()
        }
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

    pub fn can_activate(&self, card_pool: &Cards, game_state: &GameState) -> Vec<Vec<Activation>> {
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
    /// Resolutions that will need to resolve once both players end their turn. Each index/element
    /// corresponds to the phase that the action will resolve in.
    pub queued_effects: Vec<PhaseResolutions>,
    /// The current phase of the player's turn. Starts at 0 and counts up each time the player
    /// takes a major action.
    pub phase: usize,
    /// True when the player has taken a major action such as summoning a card from their hand,
    /// (they are in the response window) and can only take actions which respond to the major
    /// action taken, or yield and start their next phase to take another major action.
    pub response_window: bool,
    /// The number of cards the player is still able to draw from their deck.
    pub remaining_draws: u32,
}

/// The set of effects that will resolve for a particular player in a particular phase of their
/// turn.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PhaseResolutions {
    pub effects: Vec<Resolution>,
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

/// An action is something the player can do immediately during their turn either as a game
/// mechanic like summoning a card from their hand or as a card effect activation cost.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// Summon a card from the hand onto the next available slot on the field.
    SummonFromHand(CardInstance),
    /// Summon a card from the hand onto a specific empty slot on the field.
    SummonFromHandToSlot(CardInstance, i32),
    /// Activate an effect of a card already on the field in a particular way.
    ActivateFromField(CardInstance, CardEffect, Activation),
    /// Activate an effect of a card in the hand in a particular way.
    ActivateFromHand(CardInstance, CardEffect, Activation),
    /// Destroy a card, moving it from the field to the graveyard, retaining its column from the field.
    DestroyOnField(CardInstance),
    /// Return a card on the field to the hand.
    ReturnFieldToHand(CardInstance),
    /// End the current response window so the player can take a new major action.
    YieldResponseWindow,
    DrawFromLeftDeck,
    DrawFromRightDeck,
}

/// A resolution is something the player can queue onto the current phase of their turn which
/// does not take immediate effect but resolves simulatenously with the other player's resolutions
/// when both players finish their turn. Unlike Actions, Resolutions can thus affect the other
/// player's gamestate, but they may also miss their intended targets.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    /// Destroy a card on the opponent's field, moving it from the field to the graveyard, retaining its column from the field.
    DestroyOnOpponentField(CardInstance),
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
            for (n, effect) in card.can_activate(card_pool, self).iter().enumerate() {
                for activation in effect.iter() {
                    match activation.status {
                        ActivationStatus::Mandatory => {
                            mandatory.push(Action::ActivateFromField(card.instance, n.into(), *activation))
                        }
                        ActivationStatus::Can => {
                            optional.push(Action::ActivateFromField(card.instance, n.into(), *activation))
                        }
                    }
                }
            }
        }
        for card in self.hand.iter() {
            for (n, effect) in card.can_activate(card_pool, self).iter().enumerate() {
                for activation in effect.iter() {
                    match activation.status {
                        ActivationStatus::Mandatory => {
                            mandatory.push(Action::ActivateFromHand(card.instance, n.into(), *activation))
                        }
                        ActivationStatus::Can => {
                            optional.push(Action::ActivateFromHand(card.instance, n.into(), *activation))
                        }
                    }
                }
            }
            if !self.response_window {
                optional.push(Action::SummonFromHand(card.instance));
            }
        }
        if mandatory.is_empty() {
            if self.response_window {
                optional.push(Action::YieldResponseWindow);
            } else if self.remaining_draws > 0 {
                if !self.left_deck.is_empty() {
                    optional.push(Action::DrawFromLeftDeck);
                }
                if !self.right_deck.is_empty() {
                    optional.push(Action::DrawFromRightDeck);
                }
            }
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

    pub fn reset(&mut self) {
        self.phase = 0;
        self.queued_effects.clear();
        self.remaining_draws = 1;
    }

    /// Adds a resolution to the current phase's queued effects.
    pub fn queue_resolution(&mut self, _card_pool: &Cards, resolution: Resolution) {
        if self.queued_effects.len() - 1 < self.phase {
            self.queued_effects.push(PhaseResolutions::default());
        }
        self.queued_effects[self.phase].effects.push(resolution);
    }

    pub fn take_action(&mut self, card_pool: &Cards, action: Action) -> Result<(), Box<dyn std::error::Error>> {
        return match action {
            Action::SummonFromHand(instance) => {
                let slot = self.empty_slot_on_field();
                self.take_action(card_pool, Action::SummonFromHandToSlot(instance, slot))
            }
            Action::SummonFromHandToSlot(instance, slot) => {
                if self.field.contains_key(&slot) {
                    return Err(Box::new(InvalidAction));
                }
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
                        self.field.insert(slot, card);
                        self.response_window = true;
                        Ok(())
                    }
                    None => Err(Box::new(InvalidAction))
                }
            }
            Action::ActivateFromField(instance, effect, activation) => {
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
                                effect_type.activate(card_pool, card_type, self, instance, activation);
                                self.response_window = true;
                                Ok(())
                            }
                            None => Err(Box::new(InvalidAction)),
                        }
                    }
                    None => Err(Box::new(InvalidAction)),
                }
            }
            Action::ActivateFromHand(instance, effect, activation) => {
                match self
                    .hand
                    .iter()
                    .enumerate()
                    .find(|(_, card)| card.instance == instance)
                    .map(|(i, _)| i)
                {
                    Some(index) => {
                        let card = self.hand.get(index).expect("card always present");
                        let card_type = card.lookup_self(card_pool);
                        match card_type.effects.get(effect.0 as usize) {
                            Some(effect_type) => {
                                effect_type.activate(card_pool, card_type, self, instance, activation);
                                self.response_window = true;
                                Ok(())
                            }
                            None => Err(Box::new(InvalidAction)),
                        }
                    }
                    None => Err(Box::new(InvalidAction)),
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
                        self.response_window = true;
                        Ok(())
                    }
                    None => Err(Box::new(InvalidAction)),
                }
            }
            Action::ReturnFieldToHand(instance) => {
                match self
                    .field
                    .iter()
                    .find(|(_, card)| card.instance == instance)
                    .map(|(i, _)| i)
                    .cloned()
                {
                    Some(index) => {
                        let mut card = self.field.remove(&index).expect("card always present");
                        card.state = CardStatus::ReturnedToHand;
                        self
                            .hand
                            .push(card);
                        self.response_window = true;
                        Ok(())
                    }
                    None => Err(Box::new(InvalidAction)),
                }
            }
            Action::YieldResponseWindow => {
                // Reset the status of all our cards since we're exiting this response window
                self.hand.iter_mut().for_each(|card| card.state = CardStatus::None);
                self.field.iter_mut().for_each(|(_, card)| card.state = CardStatus::None);
                self.graveyard.iter_mut().for_each(|(_, cards)| cards.iter_mut().for_each(|card| card.state = CardStatus::None));
                self.response_window = false;
                // Enter the next phase of our turn
                self.phase += 1;
                Ok(())
            }
            Action::DrawFromLeftDeck => {
                if self.remaining_draws == 0 {
                    return Err(Box::new(InvalidAction));
                }
                let card = self.left_deck.pop();
                self.remaining_draws -= 1;
                match card {
                    Some(mut card) => {
                        card.state = CardStatus::Drawn;
                        self.hand.push(card);
                        self.response_window = true;
                        Ok(())
                    }
                    None => Err(Box::new(InvalidAction)),
                }
            }
            Action::DrawFromRightDeck => {
                if self.remaining_draws == 0 {
                    return Err(Box::new(InvalidAction));
                }
                let card = self.right_deck.pop();
                self.remaining_draws -= 1;
                match card {
                    Some(mut card) => {
                        card.state = CardStatus::Drawn;
                        self.hand.push(card);
                        self.response_window = true;
                        Ok(())
                    }
                    None => Err(Box::new(InvalidAction)),
                }
            }
        }
    }
}


pub fn resolve(_card_pool: &Cards, player_one: &mut GameState, player_two: &mut GameState) {
    let total_phases = std::cmp::max(player_one.phase, player_two.phase);
    for phase in 0..total_phases {
        let player_one_effects = player_one.queued_effects.get(phase);
        let player_two_effects = player_two.queued_effects.get(phase);
        let total_effects = std::cmp::max(
            player_one_effects
                .map(|resolutions| resolutions.effects.len()).unwrap_or(0),
            player_two_effects
                .map(|resolutions| resolutions.effects.len()).unwrap_or(0),
        );
        for i in 0..total_effects {
            let _player_one_effect = player_one_effects.and_then(|resolutions| resolutions.effects.get(i));
            let _player_two_effect = player_two_effects.and_then(|resolutions| resolutions.effects.get(i));
            // TODO: Resolve the effects on the game states
        }
    }
    player_one.reset();
    player_two.reset();
}
