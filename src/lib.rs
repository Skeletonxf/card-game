mod card_type;
mod cards;
mod state;

#[cfg(test)]
mod tests {
    use crate::cards::Cards;
    use crate::state::{Action, ActionType, Card, CardInstance, GameState, PlayerOption, FaceDownDeck, FieldSlot, InvalidAction};

    fn same_set(one: Vec<PlayerOption>, two: Vec<PlayerOption>) -> bool {
        one.iter().all(|option| two.contains(option)) && one.len() == two.len()
    }

    #[test]
    fn reading_cards() {
        let cards = Cards::get();
        let card = cards.card("Staple Dragon").unwrap();
        assert_eq!(card.name, "Staple Dragon");
    }

    #[test]
    fn starting_game_state() {
        let cards = Cards::get();
        let card = cards.card("Staple Dragon").unwrap();
        let player_one = (
            vec![Card::instantiate(card), Card::instantiate(card), Card::instantiate(card)],
            vec![],
            vec![Card::instantiate(card), Card::instantiate(card)],
            vec![
                Card::instantiate(card), Card::instantiate(card), Card::instantiate(card),
                Card::instantiate(card), Card::instantiate(card)
            ]
        );
        let player_two = (
            vec![Card::instantiate(card), Card::instantiate(card), Card::instantiate(card)],
            vec![],
            vec![Card::instantiate(card), Card::instantiate(card)],
            vec![
                Card::instantiate(card), Card::instantiate(card), Card::instantiate(card),
                Card::instantiate(card), Card::instantiate(card)
            ]
        );
        let game = GameState::start(player_one, player_two);
        println!("{:?}", game);
        let actions = game.priority_player_options();
        assert!(
            same_set(actions, vec![
                PlayerOption::Draw(FaceDownDeck::Left),
                PlayerOption::Draw(FaceDownDeck::Right),
                PlayerOption::SkipDraw,
            ])
        );
    }

    #[test]
    fn test_summoning() -> Result<(), InvalidAction> {
        let cards = Cards::get();
        let card = cards.card("Staple Dragon").unwrap();
        let player_one = (
            vec![],
            vec![],
            vec![],
            vec![Card::instantiate(card)]
        );
        let player_two = (vec![], vec![], vec![], vec![]);
        let mut game = GameState::start(player_one, player_two);
        game.priorty_player_take_option(PlayerOption::SkipDraw)?;
        game.priorty_player_take_option(PlayerOption::Action(Action {
            action_type: ActionType::Summon,
            instance: CardInstance(0),
            slot: Some(FieldSlot::F4),
        }))?;
        println!("{:?}", game);
        Ok(())
    }
}
