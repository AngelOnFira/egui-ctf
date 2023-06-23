use actix::prelude::{Message, Recipient};

use common::{ctf_message::CTFMessage, ClientId, NetworkMessage};

// The response type returned by the actor future
pub type OriginalActorResponse = ();
// The error type returned by the actor future
pub type MessageError = ();
// This is the needed result for the DeferredWork message
// It's a result that combine both Response and Error from the future response.
pub type DeferredWorkResult = Result<OriginalActorResponse, MessageError>;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<WsActorMessage>,
    pub self_id: ClientId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: ClientId,
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
pub struct CTFRoomMessage {
    pub id: ClientId,
    pub ctf_message: CTFMessage,
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum ActorRequest {}

#[derive(Message)]
#[rtype(result = "Result<(), MessageError>")]
pub struct IncomingCTFRequest {
    pub id: ClientId,
    pub ctf_message: CTFMessage,
}
