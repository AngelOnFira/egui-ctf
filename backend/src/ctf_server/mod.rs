use crate::messages::{
    CTFRoomMessage, Connect, DeferredWorkResult, Disconnect, IncomingCTFRequest, WsActorMessage,
};
use actix::{
    prelude::{Actor, Context, Handler, Recipient},
    ActorFutureExt, AsyncContext, ResponseActFuture,
};
use common::{
    ctf_message::{
        self, CTFClientStateComponent, CTFMessage, CTFState, ClientData, ClientUpdate, GameData,
        TeamData,
    },
    ClientId, NetworkMessage,
};
use entity::entities::{challenge, hacker, team, token};

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

    fn broadcast_message(&self, message_authed: NetworkMessage, message_unauthed: NetworkMessage) {
        for (_, socket_recipient) in self.sessions.iter() {
            match socket_recipient.auth {
                Auth::Unauthenticated => socket_recipient
                    .socket
                    .do_send(WsActorMessage::IncomingMessage(message_unauthed.clone())),
                Auth::Hacker { discord_id } => socket_recipient
                    .socket
                    .do_send(WsActorMessage::IncomingMessage(message_authed.clone())),
            }
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

struct CTFServerStateChange {
    discord_id: String,
    hacker_client_data: ClientData,
    hacker_team_data: TeamData,

    game_data_update: GameData,

    task: CTFServerStateChangeTask,
}

enum CTFServerStateChangeTask {
    Authenticated,
    TeamCreated,
    TeamJoined,
}

impl Handler<IncomingCTFRequest> for CTFServer {
    type Result = ResponseActFuture<Self, DeferredWorkResult>;

    fn handle(&mut self, msg: IncomingCTFRequest, _ctx: &mut Self::Context) -> Self::Result {
        // Items to be moved into closure
        let db_clone = self.db.clone();
        let recipient_clone: WsClientSocket = self.sessions.get(&msg.id).unwrap().socket.clone();
        let auth = self.sessions.get(&msg.id).unwrap().auth.clone();
        let ctf_message = msg.ctf_message.clone();

        let fut = async move {
            // Check if this client is authenticated
            match auth {
                // If they are unauthenticated, the only message we'll take from
                // them is a login message.
                // TODO: Should this also allow public data to be seen?
                // TODO: What happens if you try to log in after you
                Auth::Unauthenticated => {
                    if let CTFMessage::Login(token) = ctf_message {
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
                                        return Some(CTFServerStateChange {
                                            discord_id: hacker.discord_id.clone(),
                                            game_data_update: GameData::LoggedOut,
                                            hacker_team_data: {
                                                CTFState::get_hacker_team_data(
                                                    &hacker.discord_id,
                                                    &db_clone,
                                                )
                                                .await
                                            },
                                            hacker_client_data: {
                                                CTFState::get_hacker_client_data(
                                                    &hacker.discord_id,
                                                    &db_clone,
                                                )
                                                .await
                                            },
                                            task: CTFServerStateChangeTask::Authenticated,
                                        });
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
                    match ctf_message {
                        CTFMessage::CTFClientStateComponent(_) => todo!(),
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
                        CTFMessage::JoinTeam(_) => todo!(),
                        CTFMessage::CreateTeam(team_name) => {
                            // TODO: Check if this user is already on a team

                            // If the team name is empty, return an error
                            // message
                            if team_name.is_empty() {
                                CTFServer::send_message_associated(
                                    NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                        ClientUpdate::Notification(
                                            "Team name cannot be empty".to_string(),
                                        ),
                                    )),
                                    recipient_clone,
                                );

                                return None;
                            }

                            // Create a new team in the database
                            let team = team::ActiveModel {
                                name: Set(team_name),
                                ..Default::default()
                            }
                            .insert(&db_clone)
                            .await
                            .unwrap();

                            // Set this team as the hacker's team
                            let mut hacker: hacker::ActiveModel =
                                hacker::Entity::find_by_id(&discord_id)
                                    .one(&db_clone)
                                    .await
                                    .expect("Failed to get hacker")
                                    .unwrap()
                                    .into();

                            // Set the hacker's team
                            hacker.fk_team_id = Set(Some(team.id));

                            // Save the hacker in the database
                            hacker.update(&db_clone).await.unwrap();

                            // Get the updated state from the
                            // database.
                            CTFState::rebuild_state(&db_clone).await;

                            // Update the session to be authenticated
                            return Some(CTFServerStateChange {
                                discord_id: discord_id.clone(),
                                game_data_update: GameData::LoggedOut,
                                hacker_team_data: {
                                    CTFState::get_hacker_team_data(&discord_id, &db_clone).await
                                },
                                hacker_client_data: {
                                    CTFState::get_hacker_client_data(&discord_id, &db_clone).await
                                },
                                task: CTFServerStateChangeTask::TeamCreated,
                            });
                        }
                    }
                }
            }

            return None;
        };

        let fut = actix::fut::wrap_future::<_, Self>(fut);

        // Items to be moved into closure
        let recipient_clone: WsClientSocket = self.sessions.get(&msg.id).unwrap().socket.clone();

        let fut = fut.map(move |result: Option<CTFServerStateChange>, actor, _ctx| {
            resolve_actor_state(result, actor, msg, recipient_clone)
        });

        // Return the future to be run
        Box::pin(fut)
    }
}

/// Run any updates of state change if needed Any message sending needs to be
/// done here, since we don't have access to the actor in the `fut` block above.
///
/// This section should take in a list of tasks to be sent to a list of clients.
/// There are several types of messages
/// - To a single client
///     - What team they're on
///     - Login verification
/// - To a team
///     - Team additions
///     - Team solves
///     - Challenge note updates
/// - Broadcast to all connected hackers
///     - Challenge updates
/// - Broadcast to all connected web clients
///     - Scoreboard updates
///
/// All of this needs to be passed into this function since we can't run async
/// code from here. Ideally a list should be passed in, and then we can run all
/// of the commands in it to update the clients that need updating.
fn resolve_actor_state(
    result: Option<CTFServerStateChange>,
    actor: &mut CTFServer,
    msg: IncomingCTFRequest,
    recipient_clone: Recipient<WsActorMessage>,
) -> Result<(), ()> {
    if let Some(state_change) = result {
        match state_change.task {
            CTFServerStateChangeTask::Authenticated {} => {
                // Update the session to be authenticated
                let session = actor.sessions.get_mut(&msg.id).unwrap();
                session.auth = Auth::Hacker {
                    discord_id: state_change.discord_id,
                };

                // Broadcast this GameData update to all connected hackers.
                actor.broadcast_message(NetworkMessage::CTFMessage(
                    CTFMessage::CTFClientStateComponent(CTFClientStateComponent::GameData(
                        state_change.game_data_update,
                    )),
                ));

                // Sent the client their client data
                CTFServer::send_message_associated(
                    NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                        CTFClientStateComponent::ClientData(state_change.hacker_client_data),
                    )),
                    recipient_clone.clone(),
                );

                // Send the client their team data
                CTFServer::send_message_associated(
                    NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                        CTFClientStateComponent::TeamData(state_change.hacker_team_data),
                    )),
                    recipient_clone.clone(),
                );
            }
            CTFServerStateChangeTask::TeamCreated => {
                // Broadcast this GameData update to all connected hackers.
                actor.broadcast_message(NetworkMessage::CTFMessage(
                    CTFMessage::CTFClientStateComponent(CTFClientStateComponent::GameData(
                        state_change.game_data_update,
                    )),
                ));

                // Sent the client their client data
                CTFServer::send_message_associated(
                    NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                        CTFClientStateComponent::ClientData(state_change.hacker_client_data),
                    )),
                    recipient_clone.clone(),
                );

                // Send the client their team data
                CTFServer::send_message_associated(
                    NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                        CTFClientStateComponent::TeamData(state_change.hacker_team_data),
                    )),
                    recipient_clone.clone(),
                );
            }
            CTFServerStateChangeTask::TeamJoined => todo!(),
        }
    }

    Ok(())
}
