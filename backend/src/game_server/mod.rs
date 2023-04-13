use crate::{
    game_server::{game_room::GameData, games::guess_the_number::GuessTheNumberGame},
    messages::{ActorRequest, Connect, Disconnect, GameRoomMessage, WsActorMessage},
};
use actix::{
    prelude::{Actor, Context, Handler, Recipient},
    AsyncContext,
};
use common::{ClientId, NetworkMessage, RoomId};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

mod game_room;
pub mod games;

pub type WsClientSocket = Recipient<WsActorMessage>;
pub type GameRoomSocket = Recipient<GameRoomMessage>;

pub struct GameServer {
    sessions: HashMap<ClientId, WsClientSocket>,
    rooms: HashMap<RoomId, GameRoomSocket>,
    client_room_map: HashMap<ClientId, Option<RoomId>>,
}

impl GameServer {
    pub fn new_with_rooms() -> Self {
        GameServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
            client_room_map: HashMap::new(),
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;

    // We'll do a few things here. We're going to check once a second if more
    // than 2 players are in the game server without being in the room. If so,
    // we'll start a new game for them.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(1), |act, _ctx| {
            // Print the number of players in the game server
            println!("{} players in the game server", act.sessions.len());

            // Check if there are more than 4 players in the game server
            // without being in a room
            let players_in_lobby = act
                .client_room_map
                .iter_mut()
                .filter(|(_, room_id)| room_id.is_none())
                .collect::<Vec<_>>();

            if players_in_lobby.len() >= 2 {
                // Create a new GameRoom actor
                let game_room = game_room::GameRoom::new_with_clients(
                    players_in_lobby
                        .iter()
                        .map(|(client_id, _)| {
                            (
                                *client_id.clone(),
                                act.sessions.get(client_id).unwrap().clone(),
                            )
                        })
                        .collect(),
                    GameData::GuessTheNumber(GuessTheNumberGame::new()),
                )
                .start();

                let room_id = Uuid::new_v4();

                let game_room_recipient = game_room.recipient();

                act.rooms.insert(room_id, game_room_recipient.clone());

                // Send the room id to all the players in the lobby
                for (client_id, client_room) in players_in_lobby {
                    act.sessions
                        .get(client_id)
                        .unwrap()
                        .do_send(WsActorMessage::ActorRequest(ActorRequest::UpdateRoom(
                            game_room_recipient.clone(),
                        )));

                    *client_room = Some(room_id.clone());
                }
            }
        });
    }
}

impl GameServer {
    fn send_message(&self, message: NetworkMessage, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsActorMessage::IncomingMessage(message));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // Remove this user from the room, and notify others
        // Find the room that the user is in
        for (_, _room) in self.rooms.iter() {
            //     if room.users.contains(&msg.id) {
            //         // Notify the other users in the room
            //         for user_id in room.users.iter() {
            //             self.send_message(NetworkMessage::UserDisconnected(msg.id), user_id);
            //         }
            //     }
        }

        // // TODO: Improve this so we don't have to iterate over the whole hashmap
        // // a second time. Right now it's so it can send messages in the first,
        // // then mutate it in the second.
        // for (_, room) in self.rooms.iter_mut() {
        //     if room.users.contains(&msg.id) {
        //         // Remove the user from the room
        //         room.users.remove(&msg.id);
        //     }
        // }

        // Remove this user's session
        self.sessions.remove(&msg.id);
    }
}

impl Handler<Connect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(msg.self_id, msg.addr.clone());
        self.client_room_map.insert(msg.self_id, None);
    }
}

// impl GameServer {
//     pub fn sync_state(&self, _action: ClientAction) {}
// }
