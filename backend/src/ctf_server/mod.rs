use crate::messages::{
    CTFRoomMessage, Connect, DeferredWorkResult, Disconnect, IncomingCTFRequest, WsActorMessage,
};
use actix::{
    prelude::{Actor, Context, Handler, Recipient},
    ActorFutureExt, AsyncContext, ResponseActFuture,
};
use common::{
    ctf_message::{
        CTFClientStateComponent, CTFMessage, CTFState, ClientData, ClientUpdate, GameData, TeamData,
    },
    ClientId, NetworkMessage,
};
use entity::entities::{challenge, hacker, team, token};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

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

    fn broadcast_message_authenticated(&self, message: NetworkMessage) {
        for (_id, socket_recipient) in self.sessions.iter() {
            if let Auth::Hacker { .. } = socket_recipient.auth {
                let _ = socket_recipient
                    .socket
                    .do_send(WsActorMessage::IncomingMessage(message.clone()));
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

enum ActorTask {
    UpdateState(UpdateState),
    SendNetworkMessage(SendNetworkMessage),
}

enum UpdateState {
    SessionAuth { auth: Auth },
}

struct SendNetworkMessage {
    to: ActorTaskTo,
    message: NetworkMessage,
}

enum ActorTaskTo {
    Session(Uuid),
    Team(Vec<Uuid>),
    BroadcastAuthenticated,
    BroadcastAll,
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
            // Queue of tasks for the actor to take
            let mut tasks: Vec<ActorTask> = Vec::new();

            // Check if this client is authenticated
            match auth {
                // If they are unauthenticated, the only message we'll take from
                // them is a login message.and
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
                                        tasks.push(ActorTask::SendNetworkMessage(
                                            SendNetworkMessage {
                                                to: ActorTaskTo::Session(msg.id),
                                                message: NetworkMessage::CTFMessage(
                                                    CTFMessage::ClientUpdate(
                                                        ClientUpdate::Authenticated {
                                                            discord_username: hacker
                                                                .username
                                                                .clone(),
                                                            valid_token: token.token.clone(),
                                                        },
                                                    ),
                                                ),
                                            },
                                        ));

                                        // Get the updated state from the
                                        // database.
                                        CTFState::get_global_data(&db_clone).await;

                                        // TODO:
                                        // // Tell every other player that this
                                        // // player has logged in
                                        // tasks.push(ActorTask::SendNetworkMessage(
                                        //     SendNetworkMessage {
                                        //         to: ActorTaskTo::BroadcastAuthenticated,
                                        //         message: NetworkMessage::CTFMessage(
                                        //             CTFMessage::ServerUpdate(
                                        //                 ServerUpdate::PlayerLogin {
                                        //                     discord_username: hacker
                                        //                         .username
                                        //                         .clone(),
                                        //                 },
                                        //             ),
                                        //         ),
                                        //     },
                                        // ));

                                        // Update this session's auth state
                                        tasks.push(ActorTask::UpdateState(
                                            UpdateState::SessionAuth {
                                                auth: Auth::Hacker {
                                                    discord_id: hacker.discord_id.clone(),
                                                },
                                            },
                                        ));

                                        // Update the team on their hacker
                                        // coming online
                                        tasks.push(ActorTask::SendNetworkMessage(
                                            SendNetworkMessage {
                                                to: ActorTaskTo::Session(msg.id),
                                                message: NetworkMessage::CTFMessage(
                                                    CTFMessage::CTFClientStateComponent(
                                                        CTFClientStateComponent::TeamData(
                                                            CTFState::get_hacker_team_data(
                                                                &hacker.discord_id,
                                                                &db_clone,
                                                            )
                                                            .await,
                                                        ),
                                                    ),
                                                ),
                                            },
                                        ));

                                        // Update the client on their hacker
                                        // coming online
                                        tasks.push(ActorTask::SendNetworkMessage(
                                            SendNetworkMessage {
                                                to: ActorTaskTo::Session(msg.id),
                                                message: NetworkMessage::CTFMessage(
                                                    CTFMessage::CTFClientStateComponent(
                                                        CTFClientStateComponent::ClientData(
                                                            CTFState::get_hacker_client_data(
                                                                &hacker.discord_id,
                                                                &db_clone,
                                                            )
                                                            .await,
                                                        ),
                                                    ),
                                                ),
                                            },
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
                        CTFMessage::JoinTeam(token) => {
                            // Make sure the token isn't empty
                            if token.is_empty() {
                                CTFServer::send_message_associated(
                                    NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                        ClientUpdate::Notification(
                                            "Token cannot be empty".to_string(),
                                        ),
                                    )),
                                    recipient_clone,
                                );

                                // Return tasks
                                return tasks;
                            }

                            // See if there is a team with this token
                            let team: Option<team::Model> = team::Entity::find()
                                .filter(team::Column::JoinToken.eq(&token))
                                .one(&db_clone)
                                .await
                                .expect("Failed to check if team exists");

                            match team {
                                // If no team exists with this token, return an
                                // error message
                                None => {
                                    CTFServer::send_message_associated(
                                        NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                            ClientUpdate::Notification(
                                                "No team exists with this token".to_string(),
                                            ),
                                        )),
                                        recipient_clone,
                                    );

                                    // Return tasks
                                    return tasks;
                                }
                                Some(team) => {
                                    // Get the hacker associated with this
                                    // request
                                    let hacker: hacker::Model = hacker::Entity::find()
                                        .filter(hacker::Column::DiscordId.eq(&discord_id))
                                        .one(&db_clone)
                                        .await
                                        .expect("Failed to get hacker")
                                        .unwrap();

                                    // If this hacker is already on a team,
                                    // return an error message
                                    if hacker.fk_team_id.is_some() {
                                        CTFServer::send_message_associated(
                                            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                                ClientUpdate::Notification(
                                                    "You are already on a team".to_string(),
                                                ),
                                            )),
                                            recipient_clone,
                                        );

                                        // Return tasks
                                        return tasks;
                                    }

                                    // Update the hacker's team id
                                    let mut hacker: hacker::ActiveModel = hacker.into();
                                    hacker.fk_team_id = Set(Some(team.id));
                                    let hacker_id = hacker.clone().discord_id.unwrap();
                                    hacker
                                        .save(&db_clone)
                                        .await
                                        .expect("Failed to update hacker");

                                    // Send the hacker a message that they
                                    // joined a team
                                    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                                        to: ActorTaskTo::Session(msg.id),
                                        message: NetworkMessage::CTFMessage(
                                            CTFMessage::CTFClientStateComponent(
                                                CTFClientStateComponent::ClientData(
                                                    CTFState::get_hacker_client_data(
                                                        &hacker_id, &db_clone,
                                                    )
                                                    .await,
                                                ),
                                            ),
                                        ),
                                    }));

                                    // Send the hacker their team data
                                    tasks.push(ActorTask::SendNetworkMessage(
                                        SendNetworkMessage {
                                            to: ActorTaskTo::Session(msg.id),
                                            message: NetworkMessage::CTFMessage(
                                                CTFMessage::CTFClientStateComponent(
                                                    CTFClientStateComponent::TeamData(
                                                        CTFState::get_hacker_team_data(
                                                            &hacker_id, &db_clone,
                                                        )
                                                        .await,
                                                    ),
                                                ),
                                            ),
                                        },
                                    ));

                                    // Send the hacker a notification that they
                                    // joined a team
                                    CTFServer::send_message_associated(
                                        NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                            ClientUpdate::Notification(format!(
                                                "You joined team {}",
                                                team.name
                                            )),
                                        )),
                                        recipient_clone,
                                    );
                                }
                            }
                        }
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

                                // Return tasks
                                return tasks;
                            }

                            // Check if a team by this name already exists in
                            // the database
                            let team_exists: bool = team::Entity::find()
                                .filter(team::Column::Name.eq(&team_name))
                                .one(&db_clone)
                                .await
                                .expect("Failed to check if team exists")
                                .is_some();

                            // If a team by this name already exists, return an
                            // error message
                            if team_exists {
                                CTFServer::send_message_associated(
                                    NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                                        ClientUpdate::Notification(format!(
                                            "Team '{}' already exists",
                                            team_name
                                        )),
                                    )),
                                    recipient_clone,
                                );

                                // Return tasks
                                return tasks;
                            }

                            // Create a new team in the database
                            let team = team::ActiveModel {
                                name: Set(team_name),
                                join_token: Set(Uuid::new_v4().as_simple().to_string()),
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

                            // Broadcast this new GlobalData to every client
                            tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                                to: ActorTaskTo::Team(Vec::new()),
                                message: NetworkMessage::CTFMessage(
                                    CTFMessage::CTFClientStateComponent(
                                        CTFClientStateComponent::GlobalData(
                                            CTFState::get_global_data(&db_clone).await,
                                        ),
                                    ),
                                ),
                            }));

                            // Update the client's TeamData on their hacker
                            // joining a team
                            tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                                to: ActorTaskTo::Session(msg.id),
                                message: NetworkMessage::CTFMessage(
                                    CTFMessage::CTFClientStateComponent(
                                        CTFClientStateComponent::TeamData(
                                            CTFState::get_hacker_team_data(&discord_id, &db_clone)
                                                .await,
                                        ),
                                    ),
                                ),
                            }));
                        }
                        CTFMessage::LeaveTeam => {
                            // check that this hacker is on a team

                            let mut hacker: hacker::ActiveModel =
                                hacker::Entity::find_by_id(&discord_id)
                                    .one(&db_clone)
                                    .await
                                    .expect("Failed to get hacker")
                                    .unwrap()
                                    .into();

                            // Set the hacker's team to empty
                            hacker.fk_team_id = Set(None);

                            // Save the hacker in the database
                            hacker.update(&db_clone).await.unwrap();

                            // Broadcast this new GlobalData to every client
                            tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                                to: ActorTaskTo::Team(Vec::new()),
                                message: NetworkMessage::CTFMessage(
                                    CTFMessage::CTFClientStateComponent(
                                        CTFClientStateComponent::GlobalData(
                                            CTFState::get_global_data(&db_clone).await,
                                        ),
                                    ),
                                ),
                            }));

                            // Update the client's TeamData on their hacker
                            // leaving a team
                            tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
                                to: ActorTaskTo::Session(msg.id),
                                message: NetworkMessage::CTFMessage(
                                    CTFMessage::CTFClientStateComponent(
                                        CTFClientStateComponent::TeamData(
                                            CTFState::get_hacker_team_data(&discord_id, &db_clone)
                                                .await,
                                        ),
                                    ),
                                ),
                            }));
                        }
                    }
                }
            }

            return tasks;
        };

        let fut = actix::fut::wrap_future::<_, Self>(fut);

        // Items to be moved into closure
        let recipient_clone: WsClientSocket = self.sessions.get(&msg.id).unwrap().socket.clone();

        let fut = fut.map(move |result: Vec<ActorTask>, actor, _ctx| {
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
    result: Vec<ActorTask>,
    actor: &mut CTFServer,
    msg: IncomingCTFRequest,
    _recipient_clone: Recipient<WsActorMessage>,
) -> Result<(), ()> {
    for task in result {
        match task {
            ActorTask::UpdateState(update_state) => {
                match update_state {
                    UpdateState::SessionAuth { auth } => {
                        // Update the session to be authenticated
                        let session = actor.sessions.get_mut(&msg.id).unwrap();
                        session.auth = auth;
                    }
                }
            }
            ActorTask::SendNetworkMessage(send_network_message) => match send_network_message.to {
                ActorTaskTo::Session(session) => {
                    actor.send_message(send_network_message.message, &session);
                }
                ActorTaskTo::Team(team_members) => {
                    for member_id in team_members {
                        actor.send_message(send_network_message.message.clone(), &member_id);
                    }
                }
                ActorTaskTo::BroadcastAuthenticated => {
                    actor.broadcast_message_authenticated(send_network_message.message);
                }
                ActorTaskTo::BroadcastAll => {
                    actor.broadcast_message(send_network_message.message);
                }
            },
        }
    }

    Ok(())
}
