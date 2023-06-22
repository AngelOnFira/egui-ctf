use crate::messages::{
    CTFRoomMessage, Connect, DeferredWorkResult, Disconnect, IncomingCTFRequest,
    WsActorMessage,
};
use actix::{
    prelude::{Actor, Context, Handler, Recipient},
    ActorFutureExt, AsyncContext, ResponseActFuture,
};
use common::{
    ctf_message::{CTFMessage, CTFState, ClientUpdate},
    ClientId, NetworkMessage,
};
use entity::entities::{challenge, hacker, team};
use fake::{faker::internet::en::Username, Fake};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use std::{collections::HashMap, time::Duration};


pub type WsClientSocket = Recipient<WsActorMessage>;
pub type GameRoomSocket = Recipient<CTFRoomMessage>;

pub struct CTFServer {
    db: DatabaseConnection,
    sessions: HashMap<ClientId, WsClientSocket>,
    pub ctf_state: CTFState,
}

impl CTFServer {
    pub async fn new_with_rooms() -> anyhow::Result<Self> {
        // Load the database connection with the sqlite file.db
        let db = Database::connect("sqlite://../file.db").await?;
        Ok(CTFServer {
            db,
            sessions: HashMap::new(),
            ctf_state: CTFState::default(),
        })
    }
}

impl Actor for CTFServer {
    type Context = Context<Self>;

    // We'll do a few things here. We're going to check once a second if more
    // than 2 players are in the game server without being in the room. If so,
    // we'll start a new game for them.
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(1), |act, _ctx| {
            // Print the number of players in the game server
            println!("{} players in the game server", act.sessions.len());
        });
    }
}

impl CTFServer {
    fn send_message(&self, message: NetworkMessage, id_to: &ClientId) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsActorMessage::IncomingMessage(message));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }

    fn send_message_associated(message: NetworkMessage, to: WsClientSocket) {
        to.do_send(WsActorMessage::IncomingMessage(message));
    }

    fn broadcast_message(&self, message: NetworkMessage) {
        for (_, socket_recipient) in self.sessions.iter() {
            let _ = socket_recipient.do_send(WsActorMessage::IncomingMessage(message.clone()));
        }
    }
}

impl Handler<Disconnect> for CTFServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // Remove this user from the room, and notify others
        // Find the room that the user is in

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
        println!("User disconnected: {}", msg.id);
        self.sessions.remove(&msg.id);
    }
}

impl Handler<Connect> for CTFServer {
    type Result = ResponseActFuture<Self, DeferredWorkResult>;

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        println!("User connected: {}", msg.self_id);
        self.sessions.insert(msg.self_id, msg.addr.clone());

        let db_clone = self.db.clone();
        let fut = async move {
            // Start username generation
            let username_gen = Username();

            // Add a new team to the database
            let team = team::ActiveModel {
                name: Set(username_gen.fake::<String>()),
                ..Default::default()
            };

            let team: team::Model = team.insert(&db_clone).await.expect("Failed to insert team");

            // Add a new hacker to the database
            let hacker = hacker::ActiveModel {
                username: Set(username_gen.fake::<String>()),
                fk_team_id: Set(Some(team.id)),
                ..Default::default()
            };

            let _hacker: hacker::Model = hacker
                .insert(&db_clone)
                .await
                .expect("Failed to insert hacker");

            // Get the updated state from the database
            CTFState::rebuild_state(&db_clone).await
        };

        let fut = actix::fut::wrap_future::<_, Self>(fut);

        let fut = fut.map(|result, actor, _ctx| {
            // Actor's state updated here
            actor.ctf_state = result;

            // Broadcast the state change to all players
            actor.broadcast_message(NetworkMessage::CTFMessage(CTFMessage::CTFClientState(
                actor.ctf_state.get_client_state(),
            )));

            Ok(())
        });

        // Return the future to be run
        Box::pin(fut)
    }
}

impl Handler<IncomingCTFRequest> for CTFServer {
    type Result = ResponseActFuture<Self, DeferredWorkResult>;

    fn handle(&mut self, msg: IncomingCTFRequest, _ctx: &mut Self::Context) -> Self::Result {
        let db_clone = self.db.clone();
        let recipient_clone: WsClientSocket = self.sessions.get(&msg.id).unwrap().clone();

        let fut = async move {
            match msg.ctf_message {
                CTFMessage::CTFClientState(_) => todo!(),
                CTFMessage::SubmitFlag(flag) => {
                    // Check the database to see if there are any challenges with
                    // this flag
                    let correct_flag_challenges: Vec<challenge::Model> = challenge::Entity::find()
                        .filter(challenge::Column::Flag.eq(&flag))
                        .all(&db_clone)
                        .await
                        .expect("Failed to get challenges with flag");

                    // If they solved a challenge, send them a message that they
                    // solved a challenge
                    for challenge in &correct_flag_challenges {
                        let recipient_clone = recipient_clone.clone();
                        CTFServer::send_message_associated(
                            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                ClientUpdate::ScoredPoint(format!(
                                    "You solved {} for {} points!",
                                    challenge.title, challenge.points
                                )),
                            )),
                            recipient_clone,
                        )
                    }

                    // Otherwise, tell them they didn't solve a challenge
                    if correct_flag_challenges.is_empty() {
                        let recipient_clone = recipient_clone.clone();
                        CTFServer::send_message_associated(
                            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                ClientUpdate::ScoredPoint(
                                    "That flag didn't solve any challenges.".to_string(),
                                ),
                            )),
                            recipient_clone,
                        )
                    }
                }
                CTFMessage::ClientUpdate(_) => todo!(),
            }
        };

        let fut = actix::fut::wrap_future::<_, Self>(fut);

        let fut = fut.map(|_result, _actor, _ctx| Ok(()));

        // Return the future to be run
        Box::pin(fut)
    }
}
