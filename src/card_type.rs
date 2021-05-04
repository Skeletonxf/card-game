use crate::state::{ActivationStatus, CardInstance, CardStatus, GameState};

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
    fn can_activate(
        &self,
        card_type: &CardType,
        game_state: &GameState,
        instance: CardInstance,
    ) -> ActivationStatus;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OnSummon {}

#[typetag::serde]
impl CardEffect for OnSummon {
    fn can_activate(
        &self,
        card_type: &CardType,
        game_state: &GameState,
        instance: CardInstance,
    ) -> ActivationStatus {
        if game_state.field.values().any(|card| {
            card.instance == instance
                && card.id() == card_type.id
                && card.state == CardStatus::Summoned
        }) {
            ActivationStatus::Can
        } else {
            ActivationStatus::Cannot
        }
    }
}
