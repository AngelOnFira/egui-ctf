use ctf_message::CTFMessage;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod ctf_message;
pub mod room;

pub type ClientId = Uuid;
pub type RoomId = Uuid;

/// This message represents anything that can be sent over the network
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkMessage {
    Heartbeat,
    Time(u64),
    UserDisconnected(Uuid),
    AvailableLobbies(Vec<Uuid>),
    RoomId(Uuid),
    CTFMessage(CTFMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerData {
    pub sessions: Vec<RoomData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomData {
    pub room_name: String,
    pub user_count: u32,
}
