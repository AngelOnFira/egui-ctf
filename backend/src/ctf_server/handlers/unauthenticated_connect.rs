use crate::{
    ctf_server::{ActorTask, ActorTaskTo, CTFServer, SendNetworkMessage},
    messages::{
        CTFRoomMessage, Connect, DeferredWorkResult, Disconnect, IncomingCTFRequest, WsActorMessage,
    },
};
use actix::{
    prelude::{Actor, Context, Handler, Recipient},
    ActorFutureExt, AsyncContext, ResponseActFuture,
};
use common::{
    ctf_message::{
        CTFClientStateComponent, CTFMessage, CTFState, ClientData, ClientUpdate, DiscordClientId,
        GameData, TeamData,
    },
    ClientId, NetworkMessage,
};
use entity::entities::{challenge, hacker, submission, team, token};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub async fn handle(
    tasks: &mut Vec<ActorTask>,
    msg: &IncomingCTFRequest,
    db_clone: &DatabaseConnection,
) {
    // If a client connected but isn't authenticated,
    // send them public data about the CTF
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(msg.id),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::GlobalData(CTFState::get_global_data(db_clone).await),
        )),
    }));
}
