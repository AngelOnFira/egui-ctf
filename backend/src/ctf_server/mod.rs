use crate::messages::{
    CTFRoomMessage, Connect, DeferredWorkResult, Disconnect, IncomingCTFRequest, WsActorMessage,
};
use actix::{
    prelude::{Actor, Context, Handler, Recipient},
    ActorFutureExt, AsyncContext, ResponseActFuture,
};
use common::{
    ctf_message::{CTFClientState, CTFMessage, CTFState, ClientUpdate},
    ClientId, NetworkMessage,
};
use entity::entities::{challenge, hacker, team, token};
use fake::{faker::internet::en::Username, Fake};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use std::{collections::HashMap, time::Duration};

pub type WsClientSocket = Recipient<WsActorMessage>;
pub type GameRoomSocket = Recipient<CTFRoomMessage>;

pub struct CTFServer {
    db: DatabaseConnection,
    sessions: HashMap<ClientId, Session>,
    pub ctf_state: CTFState,
}

pub struct Session {
    auth: Auth,
    pub socket: WsClientSocket,
}

impl Session {
    pub fn new(socket: WsClientSocket) -> Self {
        Session {
            auth: Auth::Unauthenticated,
            socket,
        }
    }
}

#[derive(Debug, Clone)]
enum Auth {
    Unauthenticated,
    Hacker { discord_id: String },
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
            let _ = socket_recipient
                .socket
                .do_send(WsActorMessage::IncomingMessage(message));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }

    fn send_message_associated(message: NetworkMessage, to: WsClientSocket) {
        to.do_send(WsActorMessage::IncomingMessage(message));
    }

    fn broadcast_message(&self, message: NetworkMessage) {
        for (_, socket_recipient) in self.sessions.iter() {
            let _ = socket_recipient
                .socket
                .do_send(WsActorMessage::IncomingMessage(message.clone()));
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
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        println!("User connected: {}", msg.self_id);
        self.sessions
            .insert(msg.self_id, Session::new(msg.addr.clone()));
    }
}

enum CTFServerStateChange {
    Authenticated {
        discord_id: String,
        ctf_client_state: CTFClientState,
    },
}

impl Handler<IncomingCTFRequest> for CTFServer {
    type Result = ResponseActFuture<Self, DeferredWorkResult>;

    fn handle(&mut self, msg: IncomingCTFRequest, _ctx: &mut Self::Context) -> Self::Result {
        let db_clone = self.db.clone();
        let recipient_clone: WsClientSocket = self.sessions.get(&msg.id).unwrap().socket.clone();
        let auth = self.sessions.get(&msg.id).unwrap().auth.clone();

        let fut = async move {
            // Check if this client is authenticated
            match auth {
                // If they are unauthenticated, the only message we'll take from
                // them is a login message.
                // TODO: Should this also allow public data to be seen?
                Auth::Unauthenticated => {
                    if let CTFMessage::Login(token) = &msg.ctf_message {
                        // Find any tokens in the database that match this token
                        let token = token::Entity::find()
                            .filter(token::Column::Token.eq(token))
                            // Token is a primary key, so only getting one is fine
                            .one(&db_clone)
                            .await
                            .expect("Failed to get token");

                        // If we have that token, then we can authenticate this
                        // websocket connection as the user they say they are
                        match token {
                            Some(token) => {
                                // Get the hacker associated with this token
                                let hacker =
                                    hacker::Entity::find_by_id(token.fk_hacker_id.unwrap())
                                        .one(&db_clone)
                                        .await
                                        .expect("Failed to get hacker");

                                // If we have a hacker, then we can authenticate
                                // this websocket connection as the user they say
                                // they are
                                match hacker {
                                    Some(hacker) => {
                                        // Tell the client they are authenticated
                                        CTFServer::send_message_associated(
                                            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                                ClientUpdate::Authenticated {
                                                    discord_username: hacker.username.clone(),
                                                    valid_token: token.token.clone(),
                                                },
                                            )),
                                            recipient_clone.clone(),
                                        );

                                        // Get the updated state from the
                                        // database.
                                        CTFState::rebuild_state(&db_clone).await;

                                        // Update the session to be authenticated
                                        return Some(CTFServerStateChange::Authenticated(
                                            hacker.discord_id.clone(),
                                        ));
                                    }
                                    // If this token doesn't have a hacker
                                    // associated with it, something is wrong.
                                    // This is unreachable.
                                    None => {
                                        panic!("Token has no hacker associated with it");
                                    }
                                }
                            }
                            None => {
                                // If we don't have that token, then we can't
                                // authenticate this websocket connection
                                CTFServer::send_message_associated(
                                    NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                        ClientUpdate::IncorrectToken,
                                    )),
                                    recipient_clone,
                                );
                            }
                        }
                    }
                }
                Auth::Hacker { discord_id } => {
                    match msg.ctf_message {
                        CTFMessage::CTFClientState(_) => todo!(),
                        CTFMessage::SubmitFlag(flag) => {
                            // Check the database to see if there are any challenges with
                            // this flag
                            let correct_flag_challenges: Vec<challenge::Model> =
                                challenge::Entity::find()
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
                        CTFMessage::Login(_) => todo!(),
                    }
                }
            }

            return None;
        };

        let fut = actix::fut::wrap_future::<_, Self>(fut);

        let fut = fut.map(move |result, actor, _ctx| {
            // Run any updates of state change if needed
            if let Some(state_change) = result {
                match state_change {
                    CTFServerStateChange::Authenticated {
                        discord_id,
                        ctf_client_state,
                    } => {
                        // Update the session to be authenticated
                        let session = actor.sessions.get_mut(&msg.id).unwrap();
                        session.auth = Auth::Hacker { discord_id };

                        // Broadcast this state update to all connected hackers.
                        // This needs to be done here, since we don't have
                        // access to the actor in the `fut` block above.
                        actor.broadcast_message(NetworkMessage::CTFMessage(
                            CTFMessage::CTFClientState(actor.ctf_state.get_client_state()),
                        ));
                    }
                }
            }

            Ok(())
        });

        // Return the future to be run
        Box::pin(fut)
    }
}
