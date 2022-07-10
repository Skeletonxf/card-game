use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::cards::Cards;
use crate::card_type::{CardTypeIdentifier, CardType};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ActivatableType {
    Can,
    Mandatory,
}

static CARD_INSTANCES: AtomicU32 = AtomicU32::new(0);

// purposely not copy or clone so we never dupe cards by accident
#[derive(Debug, Eq, PartialEq)]
pub struct Card {
    pub card_type: CardTypeIdentifier,
    pub instance: CardInstance,
}

impl Card {
    /// creates a new card (intended for initialisation of a game only)
    pub fn instantiate(card_type: &CardType) -> Card {
        Card {
            card_type: card_type.id,
            instance: CardInstance(CARD_INSTANCES.fetch_add(1, Ordering::SeqCst)),
        }
    }

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
}

/// A unique id assigned to a Card to uniquely identify the copy
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CardInstance(pub u32);

impl fmt::Debug for CardInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:?}", self.0)
    }
}

/// The ith card effect a CardType may have
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CardEffect(pub u32);

impl From<usize> for CardEffect {
    fn from(index: usize) -> Self {
        CardEffect(index as u32)
    }
}
#[derive(Debug, Clone)]
pub struct InvalidAction;

impl fmt::Display for InvalidAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid Action")
    }
}

impl std::error::Error for InvalidAction {}

enum CardState {
    Deck,
    Hand,
    Field,
    Destroyed,
}

// Players choose the allocation and order of their left + center + right decks prior to turn 1
// then draw 5 cards (drawing is always the players' choice of left/right)
// Card effects that 'return to deck' are always the choice of the player the owns the card
// Players cannot search their decks. Once game is started, the decks are strictkly like a stack
// where cards can only be drawn off the top or returned to the bottom or top of the deck by card
// effect
// 'spells' are just cards like 'monsters'/'creatures' with 0 atk and 0 def
// the player will typically want to activate/summon these to their back row for protection
// but they are not distinct card types and can go to either location
// 'activating' a card from hand is just a shorthand for summoning a card from the hand with an
// 'on summon activate' effect.
#[derive(Debug, Eq, PartialEq)]
pub struct Field {
    // Cards on the front or back column in the field are always face up. Cards have both atk and
    // hp. After being attacked, damage counters are placed on the card equal to the atk. Cards
    // are destroyed when they have as many or more damage counters than hp.
    front: [Option<Card> ; 7],
    // Counters only exist on cards while they are on the field. If a card returns to the hand,
    // deck or is destroyed it loses all its counters. In this way, bouncing cards around the
    // possible states could be used to reset a card's damage.
    back: [Option<Card> ; 7],
    left_deck: Vec<Card>,
    // the center deck may only contain up to 20 cards these cards are 'in the deck'
    // before summoning
    // Cards with no cost to summon may not be placed in the center deck
    // The center deck is face up, has no order, and is public knowledge.
    center_deck: Vec<Card>,
    right_deck: Vec<Card>,
    // destroyed cards retain their column upon death, and go to a row behind the three decks
    // revival effects would typically involve the column they were destroyed in
    // The destroyed column is also face up and public knowledge, with no order to the stacked
    // cards.
    destroyed: [Vec<Card> ; 7],
    // The hand is orderless private knowledge for each player.
    hand: Vec<Card>,
}

impl Field {
    fn has_cards_to_draw(&self) -> bool {
        self.has_cards_to_draw_left() || self.has_cards_to_draw_right()
    }

    fn has_cards_to_draw_left(&self) -> bool {
        !self.left_deck.is_empty()
    }

    fn has_cards_to_draw_right(&self) -> bool {
        !self.right_deck.is_empty()
    }

    fn field_slots(&self) -> impl Iterator<Item = &Option<Card>> {
        self.front.iter().chain(self.back.iter())
    }

    fn space_on_field(&self) -> bool {
        self.field_slots().any(|slot| slot.is_none())
    }

    fn cards_to_summon(&self) -> Vec<CardInstance> {
        if self.space_on_field() {
            self.hand.iter().map(|card| card.instance).collect()
        } else {
            vec![]
        }
    }

    fn cards_to_attack(&self) -> Vec<CardInstance> {
        self.field_slots().filter_map(|slot| slot.as_ref().map(|card| card.instance)).collect()
    }

    fn slot_is_empty(&self, slot: FieldSlot) -> bool {
        self[slot].is_none()
    }

    fn empty_slots(&self) -> Vec<FieldSlot> {
        self.field_slots().enumerate().filter(|(_, s)| s.is_none()).map(|(i, _)| match i {
            0 => FieldSlot::F0,
            1 => FieldSlot::F1,
            2 => FieldSlot::F2,
            3 => FieldSlot::F3,
            4 => FieldSlot::F4,
            5 => FieldSlot::F5,
            6 => FieldSlot::F6,
            7 => FieldSlot::B0,
            8 => FieldSlot::B1,
            9 => FieldSlot::B2,
            10 => FieldSlot::B3,
            11 => FieldSlot::B4,
            12 => FieldSlot::B5,
            13 => FieldSlot::B6,
            _ => panic!(),
        }).collect()
    }
}

impl std::ops::Index<FieldSlot> for Field {
    type Output = Option<Card>;

    fn index(&self, index: FieldSlot) -> &Self::Output {
        match index {
            FieldSlot::F0 => &self.front[0],
            FieldSlot::F1 => &self.front[1],
            FieldSlot::F2 => &self.front[2],
            FieldSlot::F3 => &self.front[3],
            FieldSlot::F4 => &self.front[4],
            FieldSlot::F5 => &self.front[5],
            FieldSlot::F6 => &self.front[6],
            FieldSlot::B0 => &self.back[0],
            FieldSlot::B1 => &self.back[1],
            FieldSlot::B2 => &self.back[2],
            FieldSlot::B3 => &self.back[3],
            FieldSlot::B4 => &self.back[4],
            FieldSlot::B5 => &self.back[5],
            FieldSlot::B6 => &self.back[6],
        }
    }
}

impl std::ops::IndexMut<FieldSlot> for Field {
    fn index_mut(&mut self, index: FieldSlot) -> &mut Self::Output {
        match index {
            FieldSlot::F0 => &mut self.front[0],
            FieldSlot::F1 => &mut self.front[1],
            FieldSlot::F2 => &mut self.front[2],
            FieldSlot::F3 => &mut self.front[3],
            FieldSlot::F4 => &mut self.front[4],
            FieldSlot::F5 => &mut self.front[5],
            FieldSlot::F6 => &mut self.front[6],
            FieldSlot::B0 => &mut self.back[0],
            FieldSlot::B1 => &mut self.back[1],
            FieldSlot::B2 => &mut self.back[2],
            FieldSlot::B3 => &mut self.back[3],
            FieldSlot::B4 => &mut self.back[4],
            FieldSlot::B5 => &mut self.back[5],
            FieldSlot::B6 => &mut self.back[6],
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Player {
    One,
    Two,
}

impl Player {
    fn next(&self) -> Player {
        match self {
            Player::One => Player::Two,
            Player::Two => Player::One,
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct GameState {
    player_one: Field,
    player_two: Field,
    active: Player,
    open: GameStateType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionType {
    Effect,
    Summon,
    Attack,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GameStateType {
    /// The active player may draw and/or then take an action.
    Open {
        phase: Phase,
    },
    /// An effect has been activated and now both players may respond in turn.
    Closed,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    MayDraw, MayTakeAction,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceDownDeck {
    Left, Right,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayerOption {
    Draw(FaceDownDeck),
    SkipDraw,
    Action(Action),
    SkipAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Action {
    pub action_type: ActionType,
    pub instance: CardInstance,
    pub slot: Option<FieldSlot>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[rustfmt::skip]
pub enum FieldSlot {
    F0, F1, F2, F3, F4, F5, F6,
    B0, B1, B2, B3, B4, B5, B6,
}

use Phase::{MayDraw, MayTakeAction};
use GameStateType::{Open, Closed};

// Each player gets one action before passing priority to the other player.
// When in an open game state, a player may optionally draw a card from the top of either their
// left or right decks (this **cannot** be responded to). Regardless, the player must then take an
// action or pass to the other player. If a player is unable to draw a card or to take an action
// then they immediately lose the game. (If a player may draw but not take actions or take actions
// but not draw they are still in the game for as long as they can limp on).
// An action consists of activating an effect of a card or summoning a card to the field.
// When we pass priority to the other player after an action is taken, the game state is closed.
// The other player must choose to activate an appropriate effect of a card in response or pass
// back to the opponent.
// When both players pass, we resolve in reverse order all the activations activated in response
// to the initial action (effects during resolution **cannot** be responded to).
// After resolution, the other player gets priority and the game state is back to open.

// If a player has a card(s) with a mandatory effect that may be activated in response, they must
// choose one of the mandatory effects to activate in response. Only after all mandatory effects
// have responded may a player elect to respond with optional effects.

// Unlike YuGiOh, interactivity is built into the priority passing, not just for chain links.
// OTKs and FTKs are not a thing because you can only summon one card before your opponent gets
// priority to summon their own.
// Also unlike YuGiOh, there is no randomness in deck construction to worry about, and no searching
// or shuffling of the deck, so in person play should be a lot more fluid.

// There is no such thing as spell/trap/monster, cards are just Cards, and may be summoned to
// any of the 14 positions on the field. There is also no 'one' grave, cards retain their column
// on death. The 'extra deck' in the center is also a lot more loose, containing only cards
// with summoning costs means there's no 'extra deck' type of card either, a card in the center
// deck could also be in the left/right deck and drawn, or a player may elect that a card
// 'returned to the deck' by card effect goes back to their center deck so they can summon it
// again on the following turn.

impl GameState {
    /// Initialise a game state with both players having drawn hands and supplied decks
    pub fn start(
        player_one: (Vec<Card>, Vec<Card>, Vec<Card>, Vec<Card>),
        player_two: (Vec<Card>, Vec<Card>, Vec<Card>, Vec<Card>),
    ) -> Self {
        GameState {
            player_one: Field {
                front: [None, None, None, None, None, None, None],
                back: [None, None, None, None, None, None, None],
                left_deck: player_one.0,
                center_deck: player_one.1,
                right_deck: player_one.2,
                destroyed: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
                hand: player_one.3,
            },
            player_two: Field {
                front: [None, None, None, None, None, None, None],
                back: [None, None, None, None, None, None, None],
                left_deck: player_two.0,
                center_deck: player_two.1,
                right_deck: player_two.2,
                destroyed: [vec![], vec![], vec![], vec![], vec![], vec![], vec![]],
                hand: player_two.3,
            },
            active: Player::One,
            open: Open {
                phase: Phase::MayDraw,
            },
        }
    }

    /// Returns which player has priority
    pub fn priority(&self) -> Player {
        self.active
    }

    pub fn open(&self) -> GameStateType {
        self.open
    }

    pub fn priority_player(&self) -> &Field {
        match self.priority() {
            Player::One => &self.player_one,
            Player::Two => &self.player_two,
        }
    }

    fn priority_player_mut(&mut self) -> &mut Field {
        match self.priority() {
            Player::One => &mut self.player_one,
            Player::Two => &mut self.player_two,
        }
    }

    pub fn priority_player_options(&self) -> Vec<PlayerOption> {
        let field = self.priority_player();
        match self.open {
            Open { phase: Phase::MayDraw } => {
                let mut options = vec![ PlayerOption::SkipDraw ];
                if field.has_cards_to_draw() {
                    if field.has_cards_to_draw_left() {
                        options.push(PlayerOption::Draw(FaceDownDeck::Left));
                    }
                    if field.has_cards_to_draw_right() {
                        options.push(PlayerOption::Draw(FaceDownDeck::Right));
                    }
                }
                options
            }
            Open { phase: Phase::MayTakeAction } => {
                let mut options = vec![ PlayerOption::SkipAction ];
                for card in field.cards_to_summon() {
                    for slot in field.empty_slots() {
                        options.push(PlayerOption::Action(Action {
                            action_type: ActionType::Summon,
                            instance: card,
                            slot: Some(slot),
                        }));
                    }
                }
                for card in field.cards_to_attack() {
                    options.push(PlayerOption::Action(Action {
                        action_type: ActionType::Attack,
                        instance: card,
                        slot: None, // TODO
                    }))
                }
                // TODO: Activating effects of cards summoned on the field
                options
            },
            Closed => {
                // TODO: Responding to effects
                vec![]
            }
        }
    }

    pub fn priorty_player_take_option(&mut self, option: PlayerOption) -> Result<(), InvalidAction> {
        if !self.priority_player_options().contains(&option) {
            return Err(InvalidAction);
        }
        match option {
            PlayerOption::SkipDraw => {
                self.open = GameStateType::Open { phase: MayTakeAction };
            },
            PlayerOption::Draw(deck) => {
                let player = self.priority_player_mut();
                // move the card from deck to hand
                match deck {
                    FaceDownDeck::Left => {
                        player.hand.push(player.left_deck.pop().ok_or(InvalidAction)?);
                    }
                    FaceDownDeck::Right => {
                        player.hand.push(player.right_deck.pop().ok_or(InvalidAction)?);
                    }
                }
                self.open = GameStateType::Open { phase: MayTakeAction };
            },
            PlayerOption::SkipAction => {
                // immediately passes priority
                self.active = self.active.next();
                self.open = GameStateType::Open { phase: MayDraw };
            },
            PlayerOption::Action(action) => {
                let player = self.priority_player_mut();
                // passes priority but game state is now closed, other player may only respond
                // to the action
                match action.action_type {
                    ActionType::Summon => {
                        let slot = action.slot.ok_or(InvalidAction)?; //should this be defind on the summon subtype?
                        let card_index = player
                            .hand
                            .iter()
                            .position(|card| card.instance == action.instance)
                            .ok_or(InvalidAction)?;
                        if player.slot_is_empty(slot) {
                            let card = player.hand.remove(card_index);
                            player[slot] = Some(card);
                        } else {
                            return Err(InvalidAction);
                        }
                    }
                    ActionType::Effect => (),
                    ActionType::Attack => (),
                }
                self.open = GameStateType::Closed;
            },
        }
        Ok(())
    }
}

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "GameState {{")?;
        writeln!(f, "{:?}", self.player_one.hand)?;
        writeln!(f, "{:?}", self.player_one.destroyed)?;
        writeln!(f, "{:?} {:?} {:?}", self.player_one.left_deck, self.player_one.center_deck, self.player_one.right_deck)?;
        writeln!(f, "{:?}", self.player_one.back)?;
        writeln!(f, "{:?}", self.player_one.front)?;
        writeln!(f, "=======")?;
        writeln!(f, "{:?}", self.player_two.front)?;
        writeln!(f, "{:?}", self.player_two.back)?;
        writeln!(f, "{:?} {:?} {:?}", self.player_two.left_deck, self.player_two.center_deck, self.player_two.right_deck)?;
        writeln!(f, "{:?}", self.player_two.destroyed)?;
        writeln!(f, "{:?}", self.player_two.hand)?;
        writeln!(f, "}}")?;
        Ok(())
    }
}
