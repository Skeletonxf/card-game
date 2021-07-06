use crate::cards::Cards;
use crate::state::{Action, Activation, ActivationData, ActivationStatus, GameState, CardInstance, CardStatus};

use std::fmt;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// A unique identifier for a card type. Not part of the files, autogenerated at loading time.
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CardTypeIdentifier(pub u32);

/// A card type is like the class for cards. Cards are instances of a single CardType
#[derive(Debug, Deserialize, Serialize)]
pub struct CardType {
    #[serde(skip_deserializing)]
    pub id: CardTypeIdentifier,
    pub name: String,
    #[serde(default)]
    pub effects: Vec<Box<dyn CardEffect>>,
    pub defense: u32,
    pub attack: u32,
}

#[typetag::serde(tag = "type")]
pub trait CardEffect: Send + Sync + fmt::Debug {
    /// How can this card type effect out of the card pool activate in this game state for this card instance in the game state?
    fn can_activate(&self, card_pool: &Cards, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> Vec<Activation>;

    /// Try to activate this card type effect out of the card pool in this game state for this card instance in the game state in a particular way.
    fn activate(&self, card_pool: &Cards, card_type: &CardType, game_state: &mut GameState, instance: CardInstance, activation: Activation);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OnSummon {
    pub mandatory: bool,
    pub trigger: Box<dyn EffectTrigger>,
}

#[typetag::serde]
impl CardEffect for OnSummon {
    fn can_activate(&self, card_pool: &Cards, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> Vec<Activation> {
        if game_state.field.values()
            .any(|card|
                card.instance == instance
                && card.instance_of(card_type)
                && card.state == CardStatus::Summoned
            )
        {
            self.trigger.variants(card_pool, card_type, game_state, instance).into_iter().map(|data| Activation {
                status: if self.mandatory { ActivationStatus::Mandatory } else { ActivationStatus::Can },
                data,
            }).collect()
        } else {
            vec![]
        }
    }

    fn activate(&self, card_pool: &Cards, card_type: &CardType, game_state: &mut GameState, instance: CardInstance, activation: Activation) {
        self.trigger.activation(card_pool, card_type, game_state, instance, activation);
        self.trigger.resolution(card_pool, card_type, game_state, instance, activation);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OnDraw {
    pub mandatory: bool,
    pub trigger: Box<dyn EffectTrigger>,
}

#[typetag::serde]
impl CardEffect for OnDraw {
    fn can_activate(&self, card_pool: &Cards, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> Vec<Activation> {
        if game_state.hand.iter()
            .any(|card|
                card.instance == instance
                && card.instance_of(card_type)
                && card.state == CardStatus::Drawn
            )
        {
            self.trigger.variants(card_pool, card_type, game_state, instance).into_iter().map(|data| Activation {
                status: if self.mandatory { ActivationStatus::Mandatory } else { ActivationStatus::Can },
                data,
            }).collect()
        } else {
            vec![]
        }
    }

    fn activate(&self, card_pool: &Cards, card_type: &CardType, game_state: &mut GameState, instance: CardInstance, activation: Activation) {
        self.trigger.activation(card_pool, card_type, game_state, instance, activation);
        self.trigger.resolution(card_pool, card_type, game_state, instance, activation);
    }
}

#[typetag::serde(tag = "type")]
#[allow(unused_variables)]
pub trait EffectTrigger: Send + Sync + fmt::Debug {
    /// In what different ways can this trigger activate?
    fn variants(&self, card_pool: &Cards, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> Vec<ActivationData> {
        vec![ActivationData::default()]
    }
    fn activation(&self, card_pool: &Cards, card_type: &CardType, game_state: &mut GameState, instance: CardInstance, activation: Activation) {}
    fn resolution(&self, card_pool: &Cards, card_type: &CardType, game_state: &mut GameState, instance: CardInstance, activation: Activation) {}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DestroySelfUnless {
    pub condition: Box<dyn Condition>,
}

#[typetag::serde]
impl EffectTrigger for DestroySelfUnless {
    fn activation(&self, card_pool: &Cards, card_type: &CardType, game_state: &mut GameState, instance: CardInstance, activation: Activation) {
        if !self.condition.met(card_pool, card_type, game_state, instance, activation) {
            // swallow error, we don't care if the instance is actually on the field, just that
            // it gets destroyed if it is
            let _ = game_state.take_action(card_pool, Action::DestroyOnField(instance));
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SwapHandWithField;

#[typetag::serde]
impl EffectTrigger for SwapHandWithField {
    // We can potentially activate on any column of our field
    fn variants(&self, _card_pool: &Cards, _card_type: &CardType, game_state: &GameState, _instance: CardInstance) -> Vec<ActivationData> {
        game_state.field.iter().map(|(i, _)| ActivationData {
            slot: Some(*i)
        }).collect()
    }

    fn activation(&self, card_pool: &Cards, _card_type: &CardType, game_state: &mut GameState, instance: CardInstance, activation: Activation) {
        let slot = match activation.data.slot {
            Some(slot) => slot,
            None => return
        };
        let target = match game_state.field.get(&slot).map(|card| card.instance) {
            Some(card) => card,
            None => return
        };
        let _ = game_state.take_action(card_pool, Action::ReturnFieldToHand(target))
            .and_then(|_| game_state.take_action(card_pool, Action::SummonFromHandToSlot(instance, slot)));
    }
}

#[typetag::serde(tag = "type")]
pub trait Condition: Send + Sync + fmt::Debug {
    /// Is this card type out of the card pool in this game state for for this card instance able to meet its condition?
    fn met(&self, card_pool: &Cards, card_type: &CardType, game_state: &GameState, instance: CardInstance, activation: Activation) -> bool;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NamedCardOnField {
    pub name: String,
}

#[typetag::serde]
impl Condition for NamedCardOnField {
    fn met(&self, card_pool: &Cards, _card_type: &CardType, game_state: &GameState, _instance: CardInstance, _activation: Activation) -> bool {
        game_state.field.values().any(|card| card.has_name(card_pool, &self.name))
    }
}
