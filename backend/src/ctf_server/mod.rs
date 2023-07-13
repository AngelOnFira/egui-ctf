use crate::messages::{
    CTFRoomMessage, Connect, DeferredWorkResult, Disconnect, IncomingCTFRequest, WsActorMessage,
};
use actix::prelude::*;
use common::{
    ctf_message::{CTFMessage, CTFState, ClientData, DiscordClientId, GameData, TeamData},
    ClientId, NetworkMessage,
};

use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

use self::ai_teams::AITeams;

pub mod ai_teams;
pub mod handlers;

pub type WsClientSocket = Recipient<WsActorMessage>;
pub type GameRoomSocket = Recipient<CTFRoomMessage>;

pub struct CTFServer {
    pub db: DatabaseConnection,
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
pub enum Auth {
    Unauthenticated,
    Hacker { discord_id: DiscordClientId },
}

impl CTFServer {
    pub async fn new_with_rooms() -> anyhow::Result<Self> {
        // Load the database connection with the sqlite file.db
        let db = Database::connect("postgres://postgres:postgres@localhost:5432/postgres").await?;

        Ok(CTFServer {
            db,
            sessions: HashMap::new(),
            ctf_state: CTFState::default(),
        })
    }
}

impl Actor for CTFServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Once a second, print the number of players in the game server
        ctx.run_interval(Duration::from_secs(1), |act, _ctx| {
            // Print the number of players in the game server
            println!("{} players in the game server", act.sessions.len());
        });

        // We'll also start a thread to randomly get teams to solve challenges
        let ai_teams = AITeams::new();
        let arbiter = Arbiter::new();
        let database_clone = self.db.clone();
        ctx.run_interval(Duration::from_secs(1), move |_act, _ctx| {
            let ai_teams = ai_teams.clone();
            let database_clone = database_clone.clone();
            arbiter.spawn(async move {
                ai_teams.run(&database_clone).await;
            });
        });
    }
}

impl CTFServer {
    fn send_message(&self, message: NetworkMessage, id_to: &ClientId) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            socket_recipient
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
            socket_recipient
                .socket
                .do_send(WsActorMessage::IncomingMessage(message.clone()));
        }
    }

    fn broadcast_message_authenticated(&self, message: NetworkMessage) {
        for (_id, socket_recipient) in self.sessions.iter() {
            if let Auth::Hacker { .. } = socket_recipient.auth {
                socket_recipient
                    .socket
                    .do_send(WsActorMessage::IncomingMessage(message.clone()));
            }
        }
    }
}

impl Handler<Disconnect> for CTFServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // Remove this user from the room, and notify others Find the room that
        // the user is in

        // // TODO: Improve this so we don't have to iterate over the whole
        // hashmap // a second time. Right now it's so it can send messages in
        // the first, // then mutate it in the second. for (_, room) in
        // self.rooms.iter_mut() { if room.users.contains(&msg.id) { // Remove
        //     the user from the room room.users.remove(&msg.id); } }

        // Remove this user's session
        println!("User disconnected: {}", msg.id);
        self.sessions.remove(&msg.id);
    }
}

impl Handler<Connect> for CTFServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        println!("User connected: {}", msg.self_id);
        self.sessions.insert(msg.self_id, Session::new(msg.addr));
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

#[derive(Debug, Clone)]
pub enum ActorTask {
    UpdateState(UpdateState),
    SendNetworkMessage(SendNetworkMessage),
}

#[derive(Debug, Clone)]
pub enum UpdateState {
    SessionAuth { auth: Auth },
    Logout,
}

#[derive(Debug, Clone)]
pub struct SendNetworkMessage {
    pub to: ActorTaskTo,
    pub message: NetworkMessage,
}

#[derive(Debug, Clone)]
pub enum ActorTaskTo {
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

        let msg_clone = msg.clone();

        let fut = async move {
            // Queue of tasks for the actor to take
            let mut tasks: Vec<ActorTask> = Vec::new();

            // Check if this client is authenticated
            match auth {
                // If they are unauthenticated, the only message we'll take from
                // them is a login message.and TODO: Should this also allow
                // public data to be seen? TODO: What happens if you try to log
                // in after you
                Auth::Unauthenticated => match ctf_message {
                    CTFMessage::Login(token) => {
                        handlers::unauthenticated_login::handle(
                            token,
                            &db_clone,
                            &mut tasks,
                            &msg,
                            &recipient_clone,
                        )
                        .await;
                    }
                    CTFMessage::Connect => {
                        handlers::unauthenticated_connect::handle(&mut tasks, &msg, &db_clone)
                            .await;
                    }
                    _ => (),
                },
                Auth::Hacker { discord_id } => {
                    match ctf_message {
                        CTFMessage::CTFClientStateComponent(_) => todo!(),
                        CTFMessage::SubmitFlag {
                            challenge_name,
                            flag,
                        } => {
                            if let Some(value) =
                                handlers::authenticated_submit_flag::auth_submit_flag(
                                    challenge_name,
                                    &db_clone,
                                    discord_id,
                                    &recipient_clone,
                                    &mut tasks,
                                    flag,
                                )
                                .await
                            {
                                return value;
                            }
                        }
                        CTFMessage::ClientUpdate(_) => todo!(),
                        // TODO: This can be hit after logout for some reason
                        CTFMessage::Login(_) => todo!(),
                        CTFMessage::Logout => {
                            // If a client wants to log out, deauthenticate
                            // their stream
                            tasks.push(ActorTask::UpdateState(UpdateState::Logout));

                            // TODO: update everyone that this player has gone
                            // offline

                            // return vec![ActorTask::SendNetworkMessage(
                            //     SendNetworkMessage { to:
                            //         ActorTaskTo::Session(msg.id), message:
                            //         NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                            //             ClientUpdate::Logout, )), }, )]
                        }
                        CTFMessage::JoinTeam(token) => {
                            if let Some(value) = handlers::authenticated_join_team::handle(
                                token,
                                &recipient_clone,
                                &mut tasks,
                                &db_clone,
                                discord_id,
                                &msg,
                            )
                            .await
                            {
                                return value;
                            }
                        }
                        CTFMessage::CreateTeam(team_name) => {
                            if let Some(value) = handlers::authenticated_create_team::handle(
                                team_name,
                                recipient_clone,
                                &mut tasks,
                                &db_clone,
                                discord_id,
                                &msg,
                            )
                            .await
                            {
                                return value;
                            }
                        }
                        CTFMessage::LeaveTeam => {
                            handlers::authenticated_leave_team::handle(
                                discord_id, db_clone, &mut tasks, &msg,
                            )
                            .await;
                        }
                        CTFMessage::Connect => todo!(),
                    }
                }
            }

            tasks
        };

        let fut = actix::fut::wrap_future::<_, Self>(fut);

        // Items to be moved into closure
        let recipient_clone: WsClientSocket =
            self.sessions.get(&msg_clone.id).unwrap().socket.clone();

        let fut = fut.map(move |result: Vec<ActorTask>, actor, _ctx| {
            resolve_actor_state(result, actor, msg_clone, recipient_clone)
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
                        // Update the session to be authenticated. If the server
                        // restarted and a client is still trying to connect,
                        // then it might be in a bad state here.
                        if let Some(session) = actor.sessions.get_mut(&msg.id) {
                            session.auth = auth;
                        } else {
                            // TODO: Do some error thing here
                        }
                    }
                    UpdateState::Logout => {
                        // Update the session to be unauthenticated
                        if let Some(session) = actor.sessions.get_mut(&msg.id) {
                            session.auth = Auth::Unauthenticated;
                        } else {
                            // TODO: Do some error thing here
                        }
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
