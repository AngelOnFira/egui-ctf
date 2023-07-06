use actix::Actor;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};

use ctf_server::CTFServer;

use repo::Repo;
use start_connection::start_connection_route;

mod ctf_server;
mod messages;
mod repo;
mod start_connection;
mod ws_conn;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Try connecting to the database again once every 5 seconds if it fails
    let ctf_server = {
        loop {
            let result = CTFServer::new_with_rooms().await;

            match result {
                Ok(ctf_server) => break ctf_server,
                Err(e) => {
                    println!("Failed to connect to database: {}", e);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        }
    };
    let ctf_server = Data::new(ctf_server.start()); //create and spin up a lobby

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    // Download the repo
    // Repo::clone_repo();

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
    .bind("0.0.0.0:4040")?
    .run()
    .await
}
