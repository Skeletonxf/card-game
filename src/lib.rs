mod card_type;
mod cards;
mod state;

#[cfg(test)]
mod tests {
    use crate::cards::Cards;
    use crate::state::{Action, Card, CardInstance, CardEffect, CardStatus, GameState};

    #[test]
    fn reading_cards() {
        let cards = Cards::get();
        let card = cards.card("Staple Dragon").unwrap();
        assert_eq!(card.name, "Staple Dragon");
    }

    #[test]
    fn summon_from_hand() {
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
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::ActivateFromField(instance, CardEffect(0)));
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
    }
}
