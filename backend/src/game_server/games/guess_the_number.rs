use crate::{
    game_server::game_room::{Game, GameData, GameRoom, PlayerData},
    messages::WsActorMessage,
};
use actix::{prelude::Context, AsyncContext};
use common::{
    game_message::{CTFMessage, GuessTheNumber},
    ClientId, NetworkMessage,
};
use std::{collections::HashMap, time::Duration};

pub struct GuessTheNumberGame {
    pub max: u32,
    pub guesses: HashMap<ClientId, u32>,
    pub answer: Option<u32>,
}

impl GuessTheNumberGame {
    pub fn new() -> Self {
        GuessTheNumberGame {
            max: 10,
            guesses: HashMap::new(),
            answer: None,
        }
    }

    pub fn handle_client_message(
        _ctx: &mut Context<GameRoom>,
        client_id: ClientId,
        message: GuessTheNumber,
        game_room: &mut GameRoom,
    ) {
        // Get the game data as a GuessTheNumberGame
        let game_data = match &mut game_room.game {
            GameData::GuessTheNumber(guess_the_number_game) => guess_the_number_game,
            _ => panic!("Wrong game type"),
        };

        match message {
            GuessTheNumber::Guess { guess } => {
                game_data.guesses.insert(client_id, guess);
            }
            GuessTheNumber::Start { max: _ } => todo!(),
            GuessTheNumber::Stop => todo!(),
        }
    }
}

impl Game for GuessTheNumberGame {
    fn start(&mut self, ctx: &mut Context<GameRoom>) {
        self.answer = Some(rand::random::<u32>() % self.max);

        // Once a second, get each player to guess a number
        ctx.run_interval(Duration::from_millis(300), |game_room, _ctx| {
            // Extract the game data from act
            let game_data = match &mut game_room.game {
                GameData::GuessTheNumber(guess_the_number_game) => guess_the_number_game,
                _ => panic!("Wrong game type"),
            };

            // First, check if there was a previous answer to guess
            if let Some(answer) = game_data.answer {
                // If there was, check if any of the players guessed correctly
                for (client_id, guess) in game_data.guesses.iter() {
                    if *guess == answer {
                        // println!("{} guessed correctly!", client_id);
                        // If they did, add a point to that client's score
                        game_room
                            .players
                            .entry(*client_id)
                            .and_modify(|player_data| player_data.score += 1);
                    }
                }

                // Reset the guesses, set a new answer
                game_data.guesses.clear();
                game_data.answer = Some(rand::random::<u32>() % game_data.max);
            }

            // Send the max to each client
            for (_client_id, player_data) in game_room.players.iter() {
                let message =
                    CTFMessage::GuessTheNumber(GuessTheNumber::Start { max: game_data.max });
                player_data.socket.do_send(WsActorMessage::OutgoingMessage(
                    NetworkMessage::CTFMessage(message),
                ));
            }

            // Print out the current scores from most to least
            let mut scores: Vec<_> = game_room.players.iter().collect();
            scores.sort_by(|(_, a), (_, b)| b.score.cmp(&a.score));
            // for (client_id, player_data) in scores.iter() {
            //     println!("{}: {}", client_id, player_data.score);
            // }
        });
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
            game: GameData::GuessTheNumber(GuessTheNumberGame {
                max: 10,
                guesses: HashMap::new(),
                answer: None,
            }),
        }
    }
}
