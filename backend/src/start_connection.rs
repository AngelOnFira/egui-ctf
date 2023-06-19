use crate::{game_server::CTFServer, ws_conn::WsConn};
use actix::Addr;
use actix_web::{
    get,
    web::{Data, Payload},
    Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;

#[get("/ws")]
pub async fn start_connection_route(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<CTFServer>>,
) -> Result<HttpResponse, Error> {
    let ws = WsConn::new(srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream);

    resp
}
