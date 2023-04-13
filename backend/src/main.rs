use actix::Actor;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};

use env_logger;
use game_server::GameServer;
use start_connection::start_connection_route;

mod game_server;
mod messages;
mod start_connection;
mod ws_conn;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game_server = Data::new(GameServer::new_with_rooms().start()); //create and spin up a lobby

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route)
            .app_data(game_server.clone())
            .wrap(Logger::default())
    })
    .bind("127.0.0.1:4040")?
    .run()
    .await
}
