use crate::ctf_server::{ActorTask, ActorTaskTo, HandleData, SendNetworkMessage};

use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState},
    NetworkMessage,
};

pub async fn handle<'a>(handle_data: &'a mut HandleData<'a>) {
    // If a client connected but isn't authenticated, send them public data
    // about the CTF
    handle_data
        .tasks
        .push(ActorTask::SendNetworkMessage(SendNetworkMessage {
            to: ActorTaskTo::Session(handle_data.msg.id),
            message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                CTFClientStateComponent::GlobalData(
                    CTFState::get_global_data(&handle_data.db_clone).await,
                ),
            )),
        }));

    // Tell every other player that this player has logged in
    handle_data
        .tasks
        .push(ActorTask::SendNetworkMessage(SendNetworkMessage {
            to: ActorTaskTo::BroadcastAll,
            message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
                CTFClientStateComponent::GlobalData(
                    CTFState::get_global_data(&handle_data.db_clone).await,
                ),
            )),
        }));
}
