use crate::messages::GameRoomMessage;
use actix::prelude::{Actor, Context, Handler};
use common::{game_message::CTFMessage, ClientId, NetworkMessage};
use std::collections::HashMap;
use uuid::Uuid;

use super::{
    games::{battlesnake::Battlesnake, guess_the_number::GuessTheNumberGame},
    WsClientSocket,
};

pub struct GameRoom {
    pub players: HashMap<ClientId, PlayerData>,
    pub game: GameData,
}

pub trait Game {
    fn start(&mut self, ctx: &mut Context<GameRoom>);
    fn new_with_clients(clients: Vec<(ClientId, WsClientSocket)>) -> GameRoom;
}

pub enum GameData {
    GuessTheNumber(GuessTheNumberGame),
    Battlesnake(Battlesnake),
}

pub struct PlayerData {
    pub socket: WsClientSocket,
    pub score: u32,
}

impl GameRoom {
    pub fn new_with_clients(clients: Vec<(ClientId, WsClientSocket)>, game: GameData) -> Self {
        // Print the number of people that are in this room
        println!("{} people in this room", clients.len());

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
            game,
        }
    }
}

impl Actor for GameRoom {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        match &mut self.game {
            GameData::GuessTheNumber(guess_the_number_game) => {
                guess_the_number_game.start(ctx);
            }
            GameData::Battlesnake(battlesnake_game) => {
                battlesnake_game.start(ctx);
            }
        }
    }
}

impl GameRoom {
    fn send_message(&self, _message: NetworkMessage, _id_to: &Uuid) {
        // if let Some(socket_recipient) = self.sessions.get(id_to) {
        //     let _ = socket_recipient.do_send(ActorMessage::NetworkMessage(message));
        // } else {
        //     println!("attempting to send message but couldn't find user id.");
        // }
    }
}

impl Handler<GameRoomMessage> for GameRoom {
    type Result = ();

    fn handle(&mut self, msg: GameRoomMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg.game_message {
            CTFMessage::GuessTheNumber(guess_the_number) => {
                GuessTheNumberGame::handle_client_message(ctx, msg.id, guess_the_number, self);
            }
        }
    }
}
