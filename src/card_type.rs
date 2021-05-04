use crate::state::{ActivationStatus, GameState, CardInstance, CardStatus};

use std::fmt;

use serde::{Deserialize, Serialize};

/// A card type is like the class for cards. Multiple copies of Cards may share
/// a CardType that they are a copy of.
#[derive(Debug, Deserialize, Serialize)]
pub struct CardType {
    #[serde(skip_deserializing)]
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub effects: Vec<Box<dyn CardEffect>>,
    pub defense: u32,
    pub attack: u32,
}

#[typetag::serde(tag = "type")]
pub trait CardEffect: Send + Sync + fmt::Debug {
    fn can_activate(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> ActivationStatus;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OnSummon {
    pub mandatory: bool,
    pub trigger: Box<dyn EffectTrigger>,
}

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
            if self.mandatory { ActivationStatus::Mandatory } else { ActivationStatus::Can }
        } else {
            ActivationStatus::Cannot
        }
    }
}

#[typetag::serde(tag = "type")]
pub trait EffectTrigger: Send + Sync + fmt::Debug {
    fn activation(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance);
    fn resolution(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance);
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DestroySelfUnless {
    pub condition: Box<dyn Condition>,
}

#[typetag::serde]
impl EffectTrigger for DestroySelfUnless {
    fn activation(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance) {}
    fn resolution(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance) {
        let fire = self.condition.met(card_type, game_state, instance);
    }
}

#[typetag::serde(tag = "type")]
pub trait Condition: Send + Sync + fmt::Debug {
    fn met(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> bool;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NamedCardOnField {
    pub name: String,
}

#[typetag::serde]
impl Condition for NamedCardOnField {
    fn met(&self, card_type: &CardType, game_state: &GameState, instance: CardInstance) -> bool {
        game_state.field.values().any(|card| card.card_type.name == self.name)
    }
}
