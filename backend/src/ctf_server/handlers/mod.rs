use crate::repo::Repo;
use actix::prelude::*;
use common::ctf_message::CTFMessage;

use entity::entities::team;
use migration::{Migrator, MigratorTrait};
use sea_orm::{EntityTrait, Set};

use super::{ActorTask, Auth, HandleData, UpdateState};

pub mod authenticated_create_team;
pub mod authenticated_join_team;
pub mod authenticated_leave_team;
pub mod authenticated_submit_flag;
pub mod unauthenticated_connect;
pub mod unauthenticated_login;

pub async fn handle_request(auth: Auth, mut handle_data: HandleData<'_>) {
    let db_clone = handle_data.db_clone.clone();

    let ctf_message_clone_1 = handle_data.request.ctf_message.clone();
    let ctf_message_clone_2 = handle_data.request.ctf_message.clone();

    match auth {
        // If they are unauthenticated, the only message we'll take from
        // them is a login message.and TODO: Should this also allow
        // public data to be seen? TODO: What happens if you try to log
        // in after you
        Auth::Unauthenticated => match ctf_message_clone_1 {
            CTFMessage::Login(token) => {
                unauthenticated_login::handle(&mut handle_data, token).await;
            }
            CTFMessage::Connect => {
                unauthenticated_connect::handle(&mut handle_data).await;
            }
            _ => (),
        },
        Auth::Hacker { discord_id } => {
            match ctf_message_clone_1 {
                CTFMessage::CTFClientStateComponent(_) => todo!(),
                CTFMessage::SubmitFlag {
                    challenge_name,
                    flag,
                } => {
                    authenticated_submit_flag::handle(
                        &mut handle_data,
                        challenge_name,
                        discord_id,
                        flag,
                    )
                    .await
                }
                CTFMessage::ClientUpdate(_) => todo!(),
                // TODO: This can be hit after logout for some reason
                CTFMessage::Login(_) => todo!(),
                CTFMessage::Logout => {
                    // If a client wants to log out, deauthenticate
                    // their stream
                    handle_data
                        .tasks
                        .push(ActorTask::UpdateState(UpdateState::Logout));

                    // TODO: update everyone that this player has gone
                    // offline

                    // return vec![ActorTask::SendNetworkMessage(
                    //     SendNetworkMessage { to:
                    //         ActorTaskTo::Session(msg.id), message:
                    //         NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(
                    //             ClientUpdate::Logout, )), }, )]
                }
                CTFMessage::JoinTeam(token) => {
                    authenticated_join_team::handle(&mut handle_data, token, discord_id).await
                }
                CTFMessage::CreateTeam(team_name) => {
                    authenticated_create_team::handle(&mut handle_data, team_name, discord_id).await
                }
                CTFMessage::LeaveTeam => {
                    authenticated_leave_team::handle(&mut handle_data, discord_id).await;
                }
                CTFMessage::Connect => todo!(),
                CTFMessage::ResetDB => (),
                CTFMessage::SpawnTeams => (),
                CTFMessage::CloneRepo => (),
            }
        }
    }

    match ctf_message_clone_2 {
        CTFMessage::ResetDB => {
            println!("Resetting database");
            // Rerun the migrations on the database
            Migrator::fresh(&db_clone).await.unwrap();

            // Download the repo
            Repo::clone_repo();

            // Load the repo from the repository
            let repo = Repo::parse_repo();

            // Load all the challenges found into the database
            repo.update_database().await;

            println!("Spawn 10 teams");
            // Spawn 1000 teams
            team::Entity::insert_many((0..10).map(|i| team::ActiveModel {
                name: Set(format!("Team {}", i)),
                join_token: Set("".to_string()),
                ..Default::default()
            }))
            .exec(&db_clone)
            .await
            .unwrap();
        }
        CTFMessage::SpawnTeams => {
            println!("Spawning teams");
            // Spawn 1000 teams
            team::Entity::insert_many((0..10).map(|i| team::ActiveModel {
                name: Set(format!("Team {}", i)),
                join_token: Set("".to_string()),
                ..Default::default()
            }))
            .exec(&db_clone)
            .await
            .unwrap();
        }
        CTFMessage::CloneRepo => {
            // Download the repo
            Repo::clone_repo();

            // Load the repo from the repository
            let repo = Repo::parse_repo();

            // Load all the challenges found into the database
            repo.update_database().await;
        }
        _ => (),
    }
}
