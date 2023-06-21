use actix::{
    prelude::{Message, Recipient},
    ResponseActFuture,
};

use common::{ctf_message::CTFMessage, NetworkMessage};
use uuid::Uuid;

// The response type returned by the actor future
pub type OriginalActorResponse = ();
// The error type returned by the actor future
pub type MessageError = ();
// This is the needed result for the DeferredWork message
// It's a result that combine both Response and Error from the future response.
pub type DeferredWorkResult = Result<OriginalActorResponse, MessageError>;

#[derive(Message)]
#[rtype(result = "Result<(), MessageError>")]
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
