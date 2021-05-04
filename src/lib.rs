mod card_type;
mod cards;
mod state;

#[cfg(test)]
mod tests {
    use crate::card_type::{CardType, OnSummon, DestroySelfUnless, NamedCardOnField};
    use crate::cards::Cards;
    use crate::state::{Action, Card, CardInstance, CardStatus, GameState};

    #[test]
    fn reading_cards() {
        let cards = Cards::get();
        let card = cards.card("Staple Dragon").unwrap();
        assert_eq!(card.name, "Staple Dragon");
    }

    #[test]
    fn summon_from_hand() {
        let card = CardType {
            id: 0,
            name: "Staple Dragon".to_owned(),
            effects: vec![Box::new(OnSummon {
                mandatory: false,
                trigger: Box::new(DestroySelfUnless {
                    condition: Box::new(NamedCardOnField {
                        name: "Dragonification".to_owned(),
                    })
                })
            })],
            defense: 5,
            attack: 6,
        };
        let mut player = GameState {
            hand: vec![Card {
                card_type: &card,
                state: CardStatus::None,
                instance: CardInstance(0),
            }],
            ..Default::default()
        };
        let actions = player.actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::SummonFromHand(CardInstance(0)));
        let result = player.take_action(actions[0]);
        assert!(result.is_ok());
        assert!(player
            .field
            .values()
            .any(|card| card.instance == CardInstance(0)));
        assert!(!player
            .hand
            .iter()
            .any(|card| card.instance == CardInstance(0)));
        let actions = player.actions();
        println!("Actions: {:?}", actions);
    }
}
