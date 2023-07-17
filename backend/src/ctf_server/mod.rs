use crate::messages::{
    AnonymousCTFRequest, CTFRoomMessage, Connect, DeferredWorkResult, Disconnect,
    IncomingCTFRequest, WsActorMessage,
};
use actix::prelude::*;
use common::{
    ctf_message::{CTFMessage, CTFState, ClientData, DiscordClientId, GameData, TeamData},
    ClientId, NetworkMessage,
};

use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

use self::{ai_teams::AITeams, handlers::handle_request};

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
        ctx.run_interval(Duration::from_secs(5), move |_act, _ctx| {
            let ai_teams = ai_teams.clone();
            let database_clone = database_clone.clone();
            arbiter.spawn(async move {
                ai_teams.run(&database_clone).await;
            });
        });
    }
}

impl CTFServer {
    fn send_message(&self, message: NetworkMessage, id_to: &RequestID) {
        let id_to = match id_to {
            RequestID::Actix(id) => id,
            RequestID::Anonymous => return,
        };

        if let Some(socket_recipient) = self.sessions.get(id_to) {
            socket_recipient
                .socket
                .do_send(WsActorMessage::IncomingMessage(message));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }

    fn send_message_associated(message: NetworkMessage, to: ActixRecipient) {
        if let ActixRecipient::Actix(to) = to {
            to.do_send(WsActorMessage::IncomingMessage(message));
        }
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
    /// Send to a certain session
    Session(RequestID),
    /// Send to a certain team
    Team(Vec<Uuid>),
    /// Send to all authenticated clients
    BroadcastAuthenticated,
    /// Send to all connected clients
    BroadcastAll,
}

pub struct HandleData<'a> {
    pub db_clone: DatabaseConnection,
    pub tasks: &'a mut Vec<ActorTask>,
    pub request: ActixRequest,
    pub recipient: ActixRecipient,
}

#[derive(Debug, Clone)]
pub enum ActixRecipient {
    Actix(Recipient<WsActorMessage>),
    Anonymous,
}

#[derive(Debug, Clone)]
pub struct ActixRequest {
    pub id: RequestID,
    pub ctf_message: CTFMessage,
}

#[derive(Debug, Clone, Copy)]
pub enum RequestID {
    Actix(ClientId),
    Anonymous,
}

impl Handler<IncomingCTFRequest> for CTFServer {
    type Result = ResponseActFuture<Self, DeferredWorkResult>;

    fn handle(&mut self, msg: IncomingCTFRequest, _ctx: &mut Self::Context) -> Self::Result {
        // Items to be moved into closure
        let db_clone_1 = self.db.clone();
        let recipient_clone: WsClientSocket = self.sessions.get(&msg.id).unwrap().socket.clone();
        let auth = self.sessions.get(&msg.id).unwrap().auth.clone();

        let msg_clone_1 = msg.clone();
        let msg_clone_2 = msg;

        let fut = async move {
            // Queue of tasks for the actor to take
            let mut tasks: Vec<ActorTask> = Vec::new();

            let handle_data: HandleData<'_> = HandleData {
                db_clone: db_clone_1.clone(),
                tasks: &mut tasks,
                request: ActixRequest {
                    id: RequestID::Actix(msg_clone_1.id),
                    ctf_message: msg_clone_1.ctf_message,
                },
                recipient: ActixRecipient::Actix(recipient_clone),
            };

            handle_request(auth, handle_data).await;

            tasks
        };

        let fut = actix::fut::wrap_future::<_, Self>(fut);

        let fut = fut.map(move |result: Vec<ActorTask>, actor, _ctx| {
            resolve_actor_state(result, actor, RequestID::Actix(msg_clone_2.id))
        });

        // Return the future to be run
        Box::pin(fut)
    }
}

impl Handler<AnonymousCTFRequest> for CTFServer {
    type Result = ResponseActFuture<Self, DeferredWorkResult>;

    fn handle(&mut self, msg: AnonymousCTFRequest, _ctx: &mut Self::Context) -> Self::Result {
        // Items to be moved into closure
        let db_clone_1 = self.db.clone();
        // let recipient_clone: WsClientSocket = self.sessions.get(&msg.id).unwrap().socket.clone();
        let auth = Auth::Hacker {
            discord_id: msg.discord_id,
        };

        let msg_clone_1 = msg.clone();

        let fut = async move {
            // Queue of tasks for the actor to take
            let mut tasks: Vec<ActorTask> = Vec::new();

            let handle_data: HandleData<'_> = HandleData {
                db_clone: db_clone_1.clone(),
                tasks: &mut tasks,
                request: ActixRequest {
                    id: RequestID::Anonymous,
                    ctf_message: msg_clone_1.ctf_message,
                },
                recipient: ActixRecipient::Anonymous,
            };

            handle_request(auth, handle_data).await;

            tasks
        };

        let fut = actix::fut::wrap_future::<_, Self>(fut);

        let fut = fut.map(move |result: Vec<ActorTask>, actor, _ctx| {
            resolve_actor_state(result, actor, RequestID::Anonymous)
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
    request_id: RequestID,
    // _recipient_clone: Recipient<WsActorMessage>,
) -> Result<(), ()> {
    for task in result {
        match task {
            ActorTask::UpdateState(update_state) => {
                // Make sure we have a valid actix id
                match request_id {
                    RequestID::Actix(id) => {
                        match update_state {
                            UpdateState::SessionAuth { auth } => {
                                // Update the session to be authenticated. If the server
                                // restarted and a client is still trying to connect,
                                // then it might be in a bad state here.
                                if let Some(session) = actor.sessions.get_mut(&id) {
                                    session.auth = auth;
                                } else {
                                    // TODO: Do some error thing here
                                }
                            }
                            UpdateState::Logout => {
                                // Update the session to be unauthenticated
                                if let Some(session) = actor.sessions.get_mut(&id) {
                                    session.auth = Auth::Unauthenticated;
                                } else {
                                    // TODO: Do some error thing here
                                }
                            }
                        }
                    }
                    RequestID::Anonymous => todo!(),
                }
            }
            ActorTask::SendNetworkMessage(send_network_message) => match send_network_message.to {
                ActorTaskTo::Session(session) => {
                    actor.send_message(send_network_message.message, &session);
                }
                ActorTaskTo::Team(team_members) => {
                    for member_id in team_members {
                        actor.send_message(
                            send_network_message.message.clone(),
                            &RequestID::Actix(member_id),
                        );
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
