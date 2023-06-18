use crate::game_server::game_room::{Game, GameData, GameRoom, PlayerData};
use actix::{AsyncContext, Context};
use std::collections::HashMap;

mod snake;

pub type Coordinate = (u32, u32);

pub struct Battlesnake {
    pub turns: Vec<GameTurnState>,
}

pub struct GameTurnState {
    pub map: HashMap<Coordinate, MapEntities>,
    pub player_actions: HashMap<common::ClientId, BattlesnakePlayerTurnState>,
    pub snakes: HashMap<common::ClientId, snake::Snake>,
}

pub enum MapEntities {
    Food,
    Hazard,
    Empty,
}

pub struct BattlesnakePlayerTurnState {
    pub direction: BattlesnakeDirection,
}

// We need to store:
// - Player turn decision
// - Player's snake state

#[derive(Debug, Clone, Copy)]
pub enum BattlesnakeDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Game for Battlesnake {
    fn start(&mut self, ctx: &mut Context<GameRoom>) {
        // Set up the game map with the players in the four corners. The map is
        // 11x11
        // let mut map = HashMap::new();

        // Once a second, move to the next turn
        ctx.run_interval(std::time::Duration::from_secs(1), |game_room, _ctx| {
            // Start by resolving the previous turn. There will be a new game
            // state for each turn. The moves that each player makes will be
            // stored in the previous turn, so that a new turn can be fully set
            // up based on that.

            // Extract the game data as a battlesnake game
            let game_data = match &mut game_room.game {
                GameData::Battlesnake(battlesnake) => battlesnake,
                _ => panic!("Game data is not a battlesnake game"),
            };

            // Make a new turn state
            let _new_turn_state = GameTurnState {
                map: HashMap::new(),
                player_actions: HashMap::new(),
                snakes: HashMap::new(),
            };

            // If it's the first turn, then do nothing
            if game_data.turns.len() == 0 {
                return;
            }

            // Try to move all the snakes into their new positions
            let previous_turn = game_data.turns.last_mut().unwrap();

            // Get the player actions
            let player_actions = &previous_turn.player_actions;

            // Go through each player action. Move their snake in that
            // direction. If a player didn't submit a turn action, then they
            // should move in the same direction as the previous turn.
            let mut players_without_actions = game_room.players.keys().collect::<Vec<_>>();

            for (player_id, player_action) in player_actions {
                // Remove the player from the list of players without actions
                players_without_actions.retain(|id| *id != player_id);

                // Get the player's snake
                let snake = previous_turn.snakes.get_mut(player_id).unwrap();

                // Get the snake's direction
                let direction = match player_action {
                    BattlesnakePlayerTurnState { direction } => direction,
                };

                // Move the snake in that direction
                snake.move_direction(*direction);
            }

            // For each player without an action, move their snake in the same
            // as their previous turn with a valid action. If they don't have
            // any valid actions, then they should just move up.
            for player_id in players_without_actions {
                // Find the most recent turn with a valid action for this player
                let mut direction = BattlesnakeDirection::Up;

                for turn in game_data.turns.iter().rev() {
                    if let Some(player_action) = turn.player_actions.get(player_id) {
                        direction = player_action.direction;
                        break;
                    }
                }

                // Get the player's snake
                let snake = game_data
                    .turns
                    .last_mut()
                    .unwrap()
                    .snakes
                    .get_mut(player_id)
                    .unwrap();

                // Move the snake in that direction
                snake.move_direction(direction);
            }

            // Now, check if any snake heads have moved into the same space as
            // another snake head
            // let mut snake_heads = HashMap::new();

            // for (player_id, snake) in &previous_turn.snakes {
            //     // Get the snake's head
            //     let head = snake.get_head();

            //     // Check if the snake head is already in the map
            //     if let Some(player_id) = snake_heads.get(&head) {
            //         // If it is, then the game is over. Send a message to the
            //         // players
            //         game_room.players.get(player_id).unwrap().socket.do_send(
            //             crate::game_server::WsClientSocketMessage::Text(
            //                 "You lost the game!".to_string(),
            //             ),
            //         );

            //         game_room.players.get(player_id).unwrap().socket.do_send(
            //             crate::game_server::WsClientSocketMessage::Text(
            //                 "You won the game!".to_string(),
            //             ),
            //         );

            //         // Stop the game
            //         ctx.stop();
            //         return;
            //     }

            //     // If it isn't, then add it to the map
            //     snake_heads.insert(head, player_id);
            // }
        });

        // Get the players
    }

    fn new_with_clients(
        clients: Vec<(common::ClientId, crate::game_server::WsClientSocket)>,
    ) -> GameRoom {
        GameRoom {
            players: clients
                .into_iter()
                .map(|(client_id, client_socket)| {
                    (
                        client_id,
                        PlayerData {
                            socket: client_socket,
                            score: 0,
                        },
                    )
                })
                .collect(),
            game: GameData::Battlesnake(Battlesnake { turns: Vec::new() }),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_snake_move() {
        // Set up a game where snakes are near each other, with one spot between
        // them. The snakes should move into that spot.
    }
}
