mod card_type;
mod cards;
mod state;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use crate::cards::Cards;
    use crate::state::{Action, Activation, ActivationData, ActivationStatus, Card, CardInstance, CardEffect, CardStatus, GameState};

    #[test]
    fn reading_cards() {
        let cards = Cards::get();
        let card = cards.card("Staple Dragon").unwrap();
        assert_eq!(card.name, "Staple Dragon");
    }

    #[test]
    fn draw_from_deck() {
        let card_pool = Cards::from_test(vec![
        r#"
        name = "Staple Mirage"
        defense = 2
        attack = 0
        [[effects]]
            type = "OnDraw"
            mandatory = false
            [effects.trigger]
                type = "SwapHandWithField"
        "#,
        r#"
        name = "Dummy"
        defense = 0
        attack = 0
        "#,
        ]).expect("Parsing card types should not fail");
        let mut player = GameState {
            field: {
                let mut map = BTreeMap::new();
                map.insert(0, Card {
                    card_type: card_pool.card("Dummy").unwrap().id,
                    state: CardStatus::None,
                    instance: CardInstance(0),
                });
                map
            },
            left_deck: vec![Card {
                card_type: card_pool.card("Staple Mirage").unwrap().id,
                state: CardStatus::None,
                instance: CardInstance(1),
            }],
            remaining_draws: 1,
            ..Default::default()
        };
        let instance = CardInstance(1);

        let actions = player.actions(&card_pool);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::DrawFromLeftDeck);
        let result = player.take_action(&card_pool, actions[0]);
        assert!(result.is_ok());
        assert!(player
            .hand
            .iter()
            .any(|card| card.instance == instance));
        assert!(!player
            .left_deck
            .iter()
            .any(|card| card.instance == instance));

        let actions = player.actions(&card_pool);
        let activate_from_hand = Action::ActivateFromHand(instance, CardEffect(0), Activation {
            status: ActivationStatus::Can,
            data: ActivationData {
                slot: Some(0),
            }
        });
        assert!(actions.contains(&activate_from_hand));
        let result = player.take_action(&card_pool, activate_from_hand);
        assert!(result.is_ok());
        assert!(player
            .hand
            .iter()
            .any(|card| card.instance == CardInstance(0)));
        assert!(player
            .field
            .get(&0)
            .map(|card| card.instance == instance)
            .unwrap());
    }

    #[test]
    fn summon_from_hand_mandatory_response_window() {
        let card_pool = Cards::from_test(vec![
        r#"
        name = "Staple Dragon"
        defense = 5
        attack = 6
        [[effects]]
            type = "OnSummon"
            mandatory = true
            [effects.trigger]
                type = "DestroySelfUnless"
                [effects.trigger.condition]
                    type = "NamedCardOnField"
                    name = "Dragonification"
        "#,
        ]).expect("Parsing card types should not fail");
        let mut player = GameState {
            hand: vec![Card {
                card_type: card_pool.card("Staple Dragon").unwrap().id,
                state: CardStatus::None,
                instance: CardInstance(0),
            }],
            ..Default::default()
        };
        let instance = CardInstance(0);
        assert_eq!(player.phase, 0);

        let actions = player.actions(&card_pool);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::SummonFromHand(instance));
        let result = player.take_action(&card_pool, actions[0]);
        assert!(result.is_ok());
        assert!(player
            .field
            .values()
            .any(|card| card.instance == instance));
        assert!(!player
            .hand
            .iter()
            .any(|card| card.instance == instance));

        let actions = player.actions(&card_pool);
        assert_eq!(player.response_window, true);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::ActivateFromField(instance, CardEffect(0), ActivationStatus::Mandatory.into()));
        let result = player.take_action(&card_pool, actions[0]);
        assert!(result.is_ok());
        assert!(!player
            .field
            .values()
            .any(|card| card.instance == instance));
        assert!(player
            .graveyard
            .values()
            .any(|cards| cards.iter().any(|card| card.instance == instance)));

        let actions = player.actions(&card_pool);
        assert_eq!(player.response_window, true);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::YieldResponseWindow);
        let result = player.take_action(&card_pool, actions[0]);
        assert!(result.is_ok());
        assert_eq!(player.response_window, false);
        assert_eq!(player.phase, 1);
    }

    #[test]
    fn summon_from_hand_optional_response_window() {
        let card_pool = Cards::from_test(vec![
        r#"
        name = "Staple Dragon"
        defense = 5
        attack = 6
        [[effects]]
            type = "OnSummon"
            mandatory = false
            [effects.trigger]
                type = "DestroySelfUnless"
                [effects.trigger.condition]
                    type = "NamedCardOnField"
                    name = "Dragonification"
        "#,
        ]).expect("Parsing card types should not fail");
        let mut player = GameState {
            hand: vec![Card {
                card_type: card_pool.card("Staple Dragon").unwrap().id,
                state: CardStatus::None,
                instance: CardInstance(0),
            }],
            ..Default::default()
        };
        let instance = CardInstance(0);
        assert_eq!(player.phase, 0);

        let actions = player.actions(&card_pool);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::SummonFromHand(instance));
        let result = player.take_action(&card_pool, actions[0]);
        assert!(result.is_ok());
        assert!(player
            .field
            .values()
            .any(|card| card.instance == instance));
        assert!(!player
            .hand
            .iter()
            .any(|card| card.instance == instance));

        let actions = player.actions(&card_pool);
        assert_eq!(player.response_window, true);
        assert_eq!(actions.len(), 2);
        assert!(actions.contains(&Action::ActivateFromField(instance, CardEffect(0), ActivationStatus::Can.into())));
        assert!(actions.contains(&Action::YieldResponseWindow));
        let result = player.take_action(&card_pool, Action::YieldResponseWindow);
        assert!(result.is_ok());
        assert!(player
            .field
            .values()
            .any(|card| card.instance == instance));
        assert!(!player
            .graveyard
            .values()
            .any(|cards| cards.iter().any(|card| card.instance == instance)));
        assert_eq!(player.response_window, false);
        assert_eq!(player.phase, 1);
    }
}
