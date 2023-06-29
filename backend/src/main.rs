use actix::Actor;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};

use ctf_server::CTFServer;
use env_logger;
use git2::Repository;
use repo::Repo;
use start_connection::start_connection_route;

mod ctf_server;
mod messages;
mod start_connection;
mod ws_conn;
mod repo;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ctf_server = CTFServer::new_with_rooms().await.unwrap();
    let ctf_server = Data::new(ctf_server.start()); //create and spin up a lobby

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Load the repo from the repository
    let repo = Repo::parse_repo();

    // Load all the challenges found into the database
    repo.update_database().await;

    HttpServer::new(move || {
        App::new()
            .service(start_connection_route)
            .app_data(ctf_server.clone())
            .wrap(Logger::default())
    })
    .bind("127.0.0.1:4040")?
    .run()
    .await
}
