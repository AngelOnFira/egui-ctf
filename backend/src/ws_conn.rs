use crate::{
    game_server::{GameRoomSocket, GameServer},
    messages::{Connect, Disconnect, GameRoomMessage, WsActorMessage},
};
use actix::{
    fut, Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler, WrapFuture,
};
use actix_web_actors::ws::{self, Message};

use common::NetworkMessage;
use std::time::{Duration, Instant};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(3);

pub struct WsConn {
    room: Option<GameRoomSocket>,
    game_server_addr: Addr<GameServer>,
    hb: Instant,
    id: Uuid,
}

impl WsConn {
    pub fn new(game_server: Addr<GameServer>) -> WsConn {
        WsConn {
            id: Uuid::new_v4(),
            room: None,
            hb: Instant::now(),
            game_server_addr: game_server,
        }
    }
}

impl Actor for WsConn {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.game_server_addr
            .send(Connect {
                addr: addr.recipient(),
                self_id: self.id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.game_server_addr.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl WsConn {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                act.game_server_addr.do_send(Disconnect { id: act.id });
                ctx.stop();
                return;
            }

            // Encode a heartbeat and send it to the client
            ctx.ping(
                serde_json::to_string(&NetworkMessage::Heartbeat)
                    .unwrap()
                    .as_bytes(),
            );
        });

        // Once a second, send the elapsed time that this client has been
        // connected
        ctx.run_interval(Duration::from_secs(1), |act, ctx| {
            let msg = NetworkMessage::Time(act.hb.elapsed().as_secs());
            ctx.text(serde_json::to_string(&msg).unwrap());
        });
    }
}

impl StreamHandler<Result<Message, ws::ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(msg) => {
                match msg {
                    // Any message we get from the client should be encoded as
                    // a NetworkMessage. It will then get passed on to either
                    // the game server, or a game room.
                    Message::Text(text) => {
                        // Deserialize as a NetworkMessage
                        let message: NetworkMessage = serde_json::from_str(&text).unwrap();

                        // If we're in a room and the message is a GameMessage,
                        // pass it to the room to be handled.
                        if let Some(room) = &self.room {
                            if let NetworkMessage::CTFMessage(message) = message {
                                room.do_send(GameRoomMessage {
                                    id: self.id,
                                    game_message: message,
                                });
                            }
                        }
                    }
                    // If we get a pong back, update the heartbeat
                    Message::Pong(_) => {
                        self.hb = Instant::now();
                    }
                    _ => (),
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                ctx.stop();
            }
        }

        // if let Ok(Message::Binary(bytes)) = msg {
        //     match from_reader::<ClientAction, _>(Cursor::new(bytes)) {
        //         Ok(message) => {
        //             // Send the message to be handled by the room
        //             // self.game_server_addr.do_send(ActorMessage::NetworkMessage(
        //             //     NetworkMessage::ClientAction(_message),
        //             // ));
        //         }
        //         Err(e) => {
        //             dbg!(e);
        //         }
        //     }
        // }
    }
}

impl Handler<WsActorMessage> for WsConn {
    type Result = ();

    fn handle(&mut self, msg: WsActorMessage, ctx: &mut Self::Context) {
        match msg {
            // Pass all network messages right to the game server
            WsActorMessage::IncomingMessage(network_message) => {
                send_client_message(network_message, ctx);
            }
            WsActorMessage::ActorRequest(actor_request) => match actor_request {
                crate::messages::ActorRequest::UpdateRoom(room_id) => {
                    self.room = Some(room_id);

                    // Pass this change along
                    // let msg = NetworkMessage::RoomId(room_id);
                    // send_client_message(msg, ctx);
                }
            },
            WsActorMessage::OutgoingMessage(network_message) => {
                send_client_message(network_message, ctx);
            }
        }
    }
}

fn send_client_message(network_message: NetworkMessage, ctx: &mut ws::WebsocketContext<WsConn>) {
    ctx.text(serde_json::to_string(&network_message).unwrap());
}
