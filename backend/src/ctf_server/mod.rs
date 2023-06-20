use crate::messages::{ActorRequest, Connect, Disconnect, GameRoomMessage, WsActorMessage};
use actix::{
    prelude::{Actor, Context, Handler, Recipient},
    ActorFutureExt, AsyncContext, ResponseActFuture,
};
use common::{
    ctf_message::{self, CTFMessage, CTFState, Hacker, HackerTeam},
    ClientId, NetworkMessage, RoomId,
};
use entity::entities::{hacker, team};
use fake::{faker::internet::en::Username, Fake};
use itertools::Itertools;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

pub type WsClientSocket = Recipient<WsActorMessage>;
pub type GameRoomSocket = Recipient<GameRoomMessage>;

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
    fn send_message(&self, message: NetworkMessage, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsActorMessage::IncomingMessage(message));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
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

// The response type returned by the actor future
type OriginalActorResponse = ();
// The error type returned by the actor future
type MessageError = ();
// This is the needed result for the DeferredWork message
// It's a result that combine both Response and Error from the future response.
type DeferredWorkResult = Result<OriginalActorResponse, MessageError>;

impl Handler<Connect> for CTFServer {
    // type Result = ();
    type Result = ResponseActFuture<Self, DeferredWorkResult>;

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) -> Self::Result {
        println!("User connected: {}", msg.self_id);
        self.sessions.insert(msg.self_id, msg.addr.clone());

        // Add them to a team
        self.ctf_state.hacker_teams.push(HackerTeam {
            name: Username().fake::<String>(),
            hackers: vec![Hacker {
                name: Username().fake::<String>(),
            }],
        });

        let db_clone = self.db.clone();
        let fut = async move {
            // Add a new team to the database
            let team = team::ActiveModel {
                name: Set(Username().fake::<String>()),
                ..Default::default()
            };

            let team: team::Model = team.insert(&db_clone).await.expect("Failed to insert team");

            // Add a new hacker to the database
            let hacker = hacker::ActiveModel {
                username: Set(Username().fake::<String>()),
                fk_team_id: Set(Some(team.id)),
                ..Default::default()
            };

            let hacker: hacker::Model = hacker
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
            Ok(())
        });

        Box::pin(fut)

        // Broadcast the state change to all players
        // self.broadcase_message(NetworkMessage::CTFMessage(CTFMessage::CTFClientState(
        //     self.ctf_state.get_client_state(),
        // )));

        // Ok(())
    }
}
