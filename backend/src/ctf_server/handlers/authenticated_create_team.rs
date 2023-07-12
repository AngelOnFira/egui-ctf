use crate::{
    ctf_server::{ActorTask, ActorTaskTo, CTFServer, SendNetworkMessage},
    messages::{IncomingCTFRequest, WsActorMessage},
};
use actix::prelude::Recipient;
use common::{
    ctf_message::{CTFClientStateComponent, CTFMessage, CTFState, ClientUpdate},
    NetworkMessage,
};
use entity::entities::{hacker, team};

use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use uuid::Uuid;

pub async fn handle(
    team_name: String,
    recipient_clone: Recipient<WsActorMessage>,
    tasks: &mut Vec<ActorTask>,
    db_clone: &DatabaseConnection,
    discord_id: i64,
    msg: &IncomingCTFRequest,
) -> Option<Vec<ActorTask>> {
    if team_name.is_empty() {
        CTFServer::send_message_associated(
            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                "Team name cannot be empty".to_string(),
            ))),
            recipient_clone,
        );

        // Return tasks
        return Some(tasks.clone());
    }
    let team_exists: bool = team::Entity::find()
        .filter(team::Column::Name.eq(&team_name))
        .one(db_clone)
        .await
        .expect("Failed to check if team exists")
        .is_some();
    if team_exists {
        CTFServer::send_message_associated(
            NetworkMessage::CTFMessage(CTFMessage::ClientUpdate(ClientUpdate::Notification(
                format!("Team '{}' already exists", team_name),
            ))),
            recipient_clone,
        );

        // Return tasks
        return Some(tasks.clone());
    }
    let team = team::ActiveModel {
        name: Set(team_name),
        join_token: Set(Uuid::new_v4().as_simple().to_string()),
        ..Default::default()
    }
    .insert(db_clone)
    .await
    .unwrap();
    let mut hacker: hacker::ActiveModel = hacker::Entity::find_by_id(discord_id)
        .one(db_clone)
        .await
        .expect("Failed to get hacker")
        .unwrap()
        .into();
    hacker.fk_team_id = Set(Some(team.id));
    hacker.update(db_clone).await.unwrap();
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Team(Vec::new()),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::GlobalData(CTFState::get_global_data(db_clone).await),
        )),
    }));
    tasks.push(ActorTask::SendNetworkMessage(SendNetworkMessage {
        to: ActorTaskTo::Session(msg.id),
        message: NetworkMessage::CTFMessage(CTFMessage::CTFClientStateComponent(
            CTFClientStateComponent::TeamData(
                CTFState::get_hacker_team_data(discord_id, db_clone).await,
            ),
        )),
    }));
    // TODO: Check if this user is already on a team

    // If the team name is empty, return an error message

    // Check if a team by this name already exists in the database

    // If a team by this name already exists, return an error message

    // Create a new team in the database

    // Set this team as the hacker's team

    // Set the hacker's team

    // Save the hacker in the database

    // Broadcast this new GlobalData to every client

    // Update the client's TeamData on their hacker joining a team
    None
}
