use actix::prelude::{Message, Recipient};

use common::{ctf_message::CTFMessage, NetworkMessage};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsActorMessage>,
    pub self_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum WsActorMessage {
    /// Messages incoming from the client
    IncomingMessage(NetworkMessage),
    /// Messages that should get passed right along to the client
    OutgoingMessage(NetworkMessage),
    /// Messages that should be handled by the client's websocket actor
    ActorRequest(ActorRequest),
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameRoomMessage {
    pub id: Uuid,
    pub ctf_message: CTFMessage,
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum ActorRequest {}
