use crate::{
    ctf_server::{ActorTask, ActorTaskTo, SendNetworkMessage},
    messages::IncomingCTFRequest,
};

use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState},
    NetworkMessage,
};

use sea_orm::DatabaseConnection;

pub async fn handle(
    tasks: &mut Vec<ActorTask>,
    msg: &IncomingCTFRequest,
    db_clone: &DatabaseConnection,
) {
    // If a client connected but isn't authenticated, send them public data
    // about the CTF
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(msg.id),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::GlobalData(CTFState::get_global_data(db_clone).await),
        )),
    }));

    // Tell every other player that this player has logged in
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::BroadcastAll,
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::GlobalData(CTFState::get_global_data(db_clone).await),
        )),
    }));
}
